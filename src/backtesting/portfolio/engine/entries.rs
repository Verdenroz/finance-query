//! Priority-ordered entry fills (Step 3).

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::backtesting::engine::BacktestEngine;
use crate::backtesting::position::{Position, PositionSide};
use crate::backtesting::result::SignalRecord;
use crate::backtesting::signal::{Signal, SignalDirection};
use crate::backtesting::strategy::Strategy;

use super::super::config::PortfolioConfig;
use super::state::{SymbolState, compute_buying_power};

/// Open queued entries strongest-first against the shared buying power.
#[allow(clippy::too_many_arguments)]
pub(super) fn open_pending_entries<S: Strategy>(
    config: &PortfolioConfig,
    helper_engine: &BacktestEngine,
    states: &mut HashMap<String, SymbolState<S>>,
    mut pending_entries: Vec<(String, Signal)>,
    timestamp: i64,
    initial_capital: f64,
    n_symbols: usize,
    cash: &mut f64,
) {
    // --- Step 3: Open entry positions (highest strength first) ----------
    // Sort: strength desc, then symbol asc for determinism
    pending_entries.sort_by(|(sym_a, sig_a), (sym_b, sig_b)| {
        sig_b
            .strength
            .value()
            .partial_cmp(&sig_a.strength.value())
            .unwrap_or(Ordering::Equal)
            .then_with(|| sym_a.cmp(sym_b))
    });

    let open_positions_count: usize = states.values().filter(|s| s.position.is_some()).count();
    let mut positions_open = open_positions_count;

    for (sym, signal) in pending_entries {
        // ── Read phase ──────────────────────────────────────────────────
        // Scope the immutable borrow so it ends before the mutable write
        // phase. All values needed downstream are moved into owned bindings.
        //
        // Capture next bar's open to fill at next-bar open, avoiding
        // same-bar close-fill bias (mirrors single-symbol engine).
        let (has_position, signal_price, fill_open, fill_ts) = {
            let state = states.get(&sym).unwrap();
            let idx = state.ts_index[&timestamp];
            let signal_price = state.candles[idx].close;
            let next = state.candles.get(idx + 1).map(|c| (c.open, c.timestamp));
            (
                state.position.is_some(),
                signal_price,
                next.map(|(o, _)| o),
                next.map(|(_, t)| t),
            )
        }; // immutable borrow on `states` ends here

        if has_position {
            continue;
        }

        // No next bar — signal unexecuted (last candle in series).
        let (Some(fill_open), Some(fill_ts)) = (fill_open, fill_ts) else {
            states.get_mut(&sym).unwrap().signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed: false,
                tags: signal.tags,
            });
            continue;
        };

        // Capacity check — safe to mutate now that the immutable borrow is gone
        if let Some(max) = config.max_total_positions
            && positions_open >= max
        {
            states.get_mut(&sym).unwrap().signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed: false,
                tags: signal.tags,
            });
            continue;
        }

        if signal.direction == SignalDirection::Short && !config.base.allow_short {
            states.get_mut(&sym).unwrap().signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed: false,
                tags: signal.tags,
            });
            continue;
        }

        let is_long = signal.direction == SignalDirection::Long;
        let entry_price_slipped = config.base.apply_entry_slippage(fill_open, is_long);
        // Spread is applied after slippage so that entry_price already
        // embeds the half-spread cost; no extra spread term is needed in
        // the denominator below.
        let entry_price = config.base.apply_entry_spread(entry_price_slipped, is_long);

        // The active sizing scheme's fraction for this entry, from the
        // symbol's own series and closed trades as of the signal bar.
        let fraction = {
            let state = states.get(&sym).unwrap();
            let idx = state.ts_index[&timestamp];
            let ctx = helper_engine.build_sizing_context(idx, &state.sizing_series, &state.trades);
            config.base.sizing_fraction(entry_price, &ctx)
        };

        let buying_power = compute_buying_power(*cash, states, timestamp, config.base.max_leverage);
        let target_capital =
            config.allocation_target(&sym, buying_power, initial_capital, n_symbols, fraction);

        if target_capital <= 0.0 {
            states.get_mut(&sym).unwrap().signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed: false,
                tags: signal.tags,
            });
            continue;
        }

        // Compute a target quantity that is guaranteed to fit within
        // `target_capital` after all entry-side frictions are paid.
        //
        // Entry-side frictions:
        //   • flat commission  — reserved upfront from effective_target
        //   • % commission     — folded into denominator (entry only; exit
        //                        commission is paid from close proceeds)
        //   • half spread      — already embedded in entry_price above
        //   • transaction tax  — buy orders only (long entries); folded
        //                        into denominator because it scales with
        //                        quantity and cannot be subtracted upfront
        //
        // When commission_fn is set we cannot analytically invert it, so
        // we omit the % commission term and rely on the fill-rejection
        // guard (`entry_cost > buying_power`) to catch any over-allocation.
        let (flat_reserve, pct_friction) = if config.base.commission_fn.is_some() {
            (0.0, 0.0)
        } else {
            (config.base.commission, config.base.commission_pct)
        };
        let tax_friction = if is_long {
            config.base.transaction_tax_pct
        } else {
            0.0
        };
        let effective_target = (target_capital - flat_reserve).max(0.0);
        let quantity = effective_target / (entry_price * (1.0 + pct_friction + tax_friction));
        let entry_comm = config.base.calculate_commission(quantity, entry_price);
        let entry_tax = config
            .base
            .calculate_transaction_tax(entry_price * quantity, is_long);
        let entry_cost = entry_price * quantity + entry_comm + entry_tax;

        // A short consumes buying power by its notional just like a
        // long, even though it credits cash instead of debiting it.
        if entry_cost > buying_power {
            states.get_mut(&sym).unwrap().signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed: false,
                tags: signal.tags,
            });
            continue;
        }

        // ── Write phase: all immutable borrows of `states` are gone ────
        if is_long {
            *cash -= entry_cost;
        } else {
            *cash += entry_price * quantity - entry_comm;
        }
        let side = if is_long {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        let state = states.get_mut(&sym).unwrap();
        state.position = Some(Position::new_with_tax(
            side,
            fill_ts,
            entry_price,
            quantity,
            entry_comm,
            entry_tax,
            signal.clone(),
        ));
        state.hwm = Some(entry_price);
        state.extremes = None;
        state.signals.push(SignalRecord {
            timestamp: signal.timestamp,
            price: signal_price,
            direction: signal.direction,
            strength: signal.strength.value(),
            reason: signal.reason,
            executed: true,
            tags: signal.tags,
        });
        positions_open += 1;
    }
}
