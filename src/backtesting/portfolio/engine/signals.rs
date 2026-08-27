//! Per-bar strategy evaluation and signal dispatch (Step 2).

use std::collections::{HashMap, HashSet};

use crate::backtesting::result::SignalRecord;
use crate::backtesting::signal::{Signal, SignalDirection};
use crate::backtesting::strategy::{Strategy, StrategyContext};

use super::super::config::PortfolioConfig;
use super::state::{SymbolState, compute_buying_power, compute_portfolio_equity};

/// Evaluate each active strategy on the current bar and dispatch its signal:
/// exits and scale signals execute against the next bar, entries are returned
/// for the priority-ordered open pass.
pub(super) fn dispatch_bar_signals<S: Strategy>(
    config: &PortfolioConfig,
    states: &mut HashMap<String, SymbolState<S>>,
    active_symbols: &[String],
    exited_this_bar: &HashSet<String>,
    timestamp: i64,
    cash: &mut f64,
) -> Vec<(String, Signal)> {
    // --- Step 2: Collect strategy signals --------------------------------
    let mut pending_entries: Vec<(String, Signal)> = Vec::new();

    for sym in active_symbols {
        // Skip symbols that already auto-exited this bar
        if exited_this_bar.contains(sym) {
            continue;
        }

        // Scope the mutable borrow to extract candle_idx; `continue` propagates
        // to the enclosing `for` loop even from inside a plain block.
        let candle_idx = {
            let state = states.get_mut(sym).unwrap();
            let idx = state.ts_index[&timestamp];
            if idx < state.warmup.saturating_sub(1) {
                continue;
            }
            idx
        }; // mutable borrow on `states` released here

        // Compute portfolio equity with an immutable borrow (no conflict now)
        let portfolio_equity = compute_portfolio_equity(*cash, states, timestamp);
        let buying_power = compute_buying_power(*cash, states, timestamp, config.base.max_leverage);

        // Re-acquire the mutable borrow for strategy evaluation and signal dispatch
        let state = states.get_mut(sym).unwrap();

        let ctx = StrategyContext {
            candles: &state.candles[..=candle_idx],
            index: candle_idx,
            position: state.position.as_ref(),
            equity: portfolio_equity,
            indicators: &state.indicators,
            extremes: state.position.as_ref().and(state.extremes.as_ref()),
            indicator_index: None,
        };

        let signal = state.strategy.on_candle(&ctx);

        if signal.is_hold() {
            continue;
        }
        if signal.strength.value() < config.base.min_signal_strength {
            state.signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason.clone(),
                executed: false,
                tags: signal.tags.clone(),
            });
            continue;
        }

        match signal.direction {
            SignalDirection::Exit => {
                // Execute on next bar open to avoid same-bar close-fill bias.
                if let Some(pos) = state.position.take() {
                    if let Some(fill_candle) = state.candles.get(candle_idx + 1) {
                        let exit_price_slipped = config
                            .base
                            .apply_exit_slippage(fill_candle.open, pos.is_long());
                        let exit_price = config
                            .base
                            .apply_exit_spread(exit_price_slipped, pos.is_long());
                        let exit_comm = config.base.calculate_commission(pos.quantity, exit_price);
                        let exit_tax = config
                            .base
                            .calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());
                        let trade = pos.close_with_tax(
                            fill_candle.timestamp,
                            exit_price,
                            exit_comm,
                            exit_tax,
                            signal.clone(),
                        );
                        if trade.is_long() {
                            *cash += trade.exit_value() - exit_comm + trade.unreinvested_dividends;
                        } else {
                            *cash -= trade.exit_value() + exit_comm + exit_tax
                                - trade.unreinvested_dividends;
                        }
                        state.realized_pnl += trade.pnl;
                        state.trades.push(trade);
                        state.hwm = None;
                        state.extremes = None;
                        state.signals.push(SignalRecord {
                            timestamp: signal.timestamp,
                            price: signal.price,
                            direction: signal.direction,
                            strength: signal.strength.value(),
                            reason: signal.reason,
                            executed: true,
                            tags: signal.tags,
                        });
                    } else {
                        // No next bar — put position back, record as unexecuted.
                        state.position = Some(pos);
                        state.signals.push(SignalRecord {
                            timestamp: signal.timestamp,
                            price: signal.price,
                            direction: signal.direction,
                            strength: signal.strength.value(),
                            reason: signal.reason,
                            executed: false,
                            tags: signal.tags,
                        });
                    }
                }
            }
            SignalDirection::Long | SignalDirection::Short => {
                // Queue for priority-ordered entry
                pending_entries.push((sym.clone(), signal));
            }
            SignalDirection::ScaleIn => {
                let fraction = signal.scale_fraction.unwrap_or(0.0).clamp(0.0, 1.0);
                let executed = fraction > 0.0
                    && state.position.is_some()
                    && state
                        .candles
                        .get(candle_idx + 1)
                        .is_some_and(|fill_candle| {
                            let pos = state.position.as_mut().unwrap();
                            let is_long = pos.is_long();
                            let fill_price = config.base.apply_entry_spread(
                                config.base.apply_entry_slippage(fill_candle.open, is_long),
                                is_long,
                            );
                            if fill_price <= 0.0 {
                                return false;
                            }
                            let add_value = portfolio_equity * fraction;
                            let add_qty = add_value / fill_price;
                            let commission = config.base.calculate_commission(add_qty, fill_price);
                            let entry_tax =
                                config.base.calculate_transaction_tax(add_value, is_long);
                            // Both directions consume buying power by
                            // the added notional plus costs.
                            let total_cost = add_value + commission + entry_tax;
                            if add_qty <= 0.0 || total_cost > buying_power {
                                return false;
                            }
                            if is_long {
                                *cash -= add_value + commission + entry_tax;
                            } else {
                                *cash += add_value - commission;
                            }
                            pos.scale_in(fill_price, add_qty, commission, entry_tax);
                            true
                        });
                state.signals.push(SignalRecord {
                    timestamp: signal.timestamp,
                    price: signal.price,
                    direction: signal.direction,
                    strength: signal.strength.value(),
                    reason: signal.reason,
                    executed,
                    tags: signal.tags,
                });
            }
            SignalDirection::ScaleOut => {
                let fraction = signal.scale_fraction.unwrap_or(0.0).clamp(0.0, 1.0);
                let executed = fraction > 0.0 && {
                    // Extract position metadata before any mutable borrow.
                    let pos_meta = state.position.as_ref().map(|p| (p.is_long(), p.quantity));
                    match (state.candles.get(candle_idx + 1), pos_meta) {
                        (Some(fill_candle), Some((is_long, qty_full))) => {
                            let exit_price = config.base.apply_exit_spread(
                                config.base.apply_exit_slippage(fill_candle.open, is_long),
                                is_long,
                            );
                            let qty_to_close = if fraction >= 1.0 {
                                qty_full
                            } else {
                                qty_full * fraction
                            };
                            let commission =
                                config.base.calculate_commission(qty_to_close, exit_price);
                            let exit_tax = config
                                .base
                                .calculate_transaction_tax(exit_price * qty_to_close, !is_long);
                            let trade = if fraction >= 1.0 {
                                let pos = state.position.take().unwrap();
                                state.hwm = None;
                                state.extremes = None;
                                pos.close_with_tax(
                                    fill_candle.timestamp,
                                    exit_price,
                                    commission,
                                    exit_tax,
                                    signal.clone(),
                                )
                            } else {
                                state.position.as_mut().unwrap().partial_close(
                                    fraction,
                                    fill_candle.timestamp,
                                    exit_price,
                                    commission,
                                    exit_tax,
                                    signal.clone(),
                                )
                            };
                            if trade.is_long() {
                                *cash +=
                                    trade.exit_value() - commission + trade.unreinvested_dividends;
                            } else {
                                *cash -= trade.exit_value() + commission + exit_tax
                                    - trade.unreinvested_dividends;
                            }
                            state.realized_pnl += trade.pnl;
                            state.trades.push(trade);
                            true
                        }
                        _ => false,
                    }
                };
                state.signals.push(SignalRecord {
                    timestamp: signal.timestamp,
                    price: signal.price,
                    direction: signal.direction,
                    strength: signal.strength.value(),
                    reason: signal.reason,
                    executed,
                    tags: signal.tags,
                });
            }
            SignalDirection::Hold => {}
        }
    }

    pending_entries
}
