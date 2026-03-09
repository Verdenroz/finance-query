//! Multi-symbol portfolio backtesting engine.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::backtesting::config::BacktestConfig;
use crate::backtesting::engine::{BacktestEngine, update_trailing_hwm};
use crate::backtesting::error::{BacktestError, Result};
use crate::backtesting::position::{Position, PositionSide, Trade};
use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics, SignalRecord};
use crate::backtesting::signal::{Signal, SignalDirection};
use crate::backtesting::strategy::{Strategy, StrategyContext};
use crate::models::chart::{Candle, Dividend};

use super::config::PortfolioConfig;
use super::result::{AllocationSnapshot, PortfolioResult};

// ── Public types ──────────────────────────────────────────────────────────────

/// Input data for a single symbol in the portfolio backtest.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct SymbolData {
    /// Ticker symbol (e.g. `"AAPL"`)
    pub symbol: String,

    /// OHLCV candles sorted by timestamp ascending.
    pub candles: Vec<Candle>,

    /// Dividend history sorted by timestamp ascending.
    ///
    /// An empty vec disables dividend processing for this symbol.
    pub dividends: Vec<Dividend>,
}

impl SymbolData {
    /// Convenience constructor with no dividends.
    pub fn new(symbol: impl Into<String>, candles: Vec<Candle>) -> Self {
        Self {
            symbol: symbol.into(),
            candles,
            dividends: vec![],
        }
    }

    /// Attach dividends (sorted ascending by timestamp).
    pub fn with_dividends(mut self, dividends: Vec<Dividend>) -> Self {
        self.dividends = dividends;
        self
    }
}

/// Multi-symbol portfolio backtesting engine.
///
/// Runs all symbols on a shared capital pool, applying the configured
/// allocation strategy and position constraints simultaneously.
pub struct PortfolioEngine {
    config: PortfolioConfig,
}

impl PortfolioEngine {
    /// Create a new portfolio engine.
    pub fn new(config: PortfolioConfig) -> Self {
        Self { config }
    }

    /// Run a portfolio backtest.
    ///
    /// `factory` is called once per symbol to create an independent strategy
    /// instance for that symbol. Use a closure that captures any shared
    /// parameters:
    ///
    /// ```ignore
    /// engine.run(&symbol_data, |sym| SmaCrossover::new(10, 50))
    /// ```
    ///
    /// Entry signals across symbols are ranked by strength (descending); ties
    /// broken alphabetically, giving deterministic results.
    pub fn run<S, F>(&self, symbol_data: &[SymbolData], factory: F) -> Result<PortfolioResult>
    where
        S: Strategy,
        F: Fn(&str) -> S,
    {
        let n_symbols = symbol_data.len();
        self.config.validate(n_symbols)?;

        let initial_capital = self.config.base.initial_capital;

        // ── Build per-symbol state ─────────────────────────────────────────────
        let helper_engine = BacktestEngine::new(self.config.base.clone());

        let mut states: HashMap<String, SymbolState<S>> = HashMap::with_capacity(n_symbols);
        for data in symbol_data {
            let strategy = factory(&data.symbol);
            let warmup = strategy.warmup_period();
            if data.candles.len() < warmup {
                return Err(BacktestError::insufficient_data(warmup, data.candles.len()));
            }
            let strategy_name = strategy.name().to_string();
            let indicators = helper_engine.compute_indicators(&data.candles, &strategy)?;
            let ts_index: HashMap<i64, usize> = data
                .candles
                .iter()
                .enumerate()
                .map(|(i, c)| (c.timestamp, i))
                .collect();

            // Pre-compute the expected per-symbol capital allocation so that
            // per-symbol equity, return %, and Sharpe are relative to the
            // actual amount deployed — not the full portfolio initial_capital.
            let sym_initial_capital = self.config.allocation_target(
                &data.symbol,
                initial_capital,
                initial_capital,
                n_symbols,
            );

            states.insert(
                data.symbol.clone(),
                SymbolState {
                    candles: data.candles.clone(),
                    dividends: data.dividends.clone(),
                    ts_index,
                    indicators,
                    strategy,
                    warmup,
                    position: None,
                    hwm: None,
                    div_idx: 0,
                    trades: vec![],
                    signals: vec![],
                    realized_pnl: 0.0,
                    equity_curve: vec![],
                    sym_peak: sym_initial_capital,
                    sym_initial_capital,
                    strategy_name,
                },
            );
        }

        // ── Build master timeline (union of all symbol timestamps) ─────────────
        let master_timeline: BTreeSet<i64> = states
            .values()
            .flat_map(|s| s.candles.iter().map(|c| c.timestamp))
            .collect();

        // ── Shared portfolio state ─────────────────────────────────────────────
        let mut cash = initial_capital;
        let mut portfolio_equity_curve: Vec<EquityPoint> = Vec::new();
        let mut allocation_history: Vec<AllocationSnapshot> = Vec::new();
        let mut portfolio_peak = initial_capital;

        // ── Main simulation loop ───────────────────────────────────────────────
        for &timestamp in &master_timeline {
            // Collect present symbols for this bar (parallel mutable iteration
            // is not possible, so we collect keys then iterate)
            let active_symbols: Vec<String> = states
                .keys()
                .filter(|sym| states[*sym].ts_index.contains_key(&timestamp))
                .cloned()
                .collect();

            // --- Step 1: Update position values, dividends, trailing stops ----
            let mut auto_exits: Vec<(String, Signal)> = Vec::new();

            for sym in &active_symbols {
                let state = states.get_mut(sym).unwrap();
                let candle_idx = state.ts_index[&timestamp];
                let candle = &state.candles[candle_idx];

                // Update HWM for trailing stop using the intrabar extreme so the
                // trailing stop correctly reflects the best price reached during the bar.
                update_trailing_hwm(state.position.as_ref(), &mut state.hwm, candle);

                // Credit dividends ex-dated on or before this bar
                while state.div_idx < state.dividends.len()
                    && state.dividends[state.div_idx].timestamp <= timestamp
                {
                    if let Some(ref mut pos) = state.position {
                        let per_share = state.dividends[state.div_idx].amount;
                        let income = if pos.is_long() {
                            per_share * pos.quantity
                        } else {
                            -(per_share * pos.quantity)
                        };
                        pos.credit_dividend(
                            income,
                            candle.close,
                            self.config.base.reinvest_dividends,
                        );
                    }
                    state.div_idx += 1;
                }

                // Check SL/TP/trailing stop
                if let Some(ref pos) = state.position
                    && let Some(exit_signal) =
                        check_sl_tp(pos, candle, state.hwm, &self.config.base)
                {
                    auto_exits.push((sym.clone(), exit_signal));
                }
            }

            // Process auto-exits (SL/TP/trailing) — execute on the current bar at the
            // fill price embedded in the signal (stop/TP level with gap guard).
            let mut exited_this_bar: HashSet<String> = HashSet::new();
            for (sym, exit_signal) in auto_exits {
                let state = states.get_mut(&sym).unwrap();
                let fill_price = exit_signal.price;

                let Some(pos) = state.position.take() else {
                    continue;
                };
                let exit_price_slipped = self
                    .config
                    .base
                    .apply_exit_slippage(fill_price, pos.is_long());
                let exit_price = self
                    .config
                    .base
                    .apply_exit_spread(exit_price_slipped, pos.is_long());
                let exit_comm = self
                    .config
                    .base
                    .calculate_commission(pos.quantity, exit_price);
                let exit_tax = self
                    .config
                    .base
                    .calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());
                let exit_reason = exit_signal.reason.clone();
                let exit_tags = exit_signal.tags.clone();
                let trade =
                    pos.close_with_tax(timestamp, exit_price, exit_comm, exit_tax, exit_signal);
                if trade.is_long() {
                    cash += trade.exit_value() - exit_comm + trade.unreinvested_dividends;
                } else {
                    cash -=
                        trade.exit_value() + exit_comm + exit_tax - trade.unreinvested_dividends;
                }
                state.realized_pnl += trade.pnl;
                state.trades.push(trade);
                state.hwm = None;
                state.signals.push(SignalRecord {
                    timestamp,
                    price: fill_price,
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                    reason: exit_reason,
                    executed: true,
                    tags: exit_tags,
                });
                exited_this_bar.insert(sym);
            }

            // --- Step 2: Collect strategy signals --------------------------------
            let mut pending_entries: Vec<(String, Signal)> = Vec::new();

            for sym in &active_symbols {
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
                let portfolio_equity = compute_portfolio_equity(cash, &states, timestamp);

                // Re-acquire the mutable borrow for strategy evaluation and signal dispatch
                let state = states.get_mut(sym).unwrap();

                let ctx = StrategyContext {
                    candles: &state.candles[..=candle_idx],
                    index: candle_idx,
                    position: state.position.as_ref(),
                    equity: portfolio_equity,
                    indicators: &state.indicators,
                };

                let signal = state.strategy.on_candle(&ctx);

                if signal.is_hold() {
                    continue;
                }
                if signal.strength.value() < self.config.base.min_signal_strength {
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
                                let exit_price_slipped = self
                                    .config
                                    .base
                                    .apply_exit_slippage(fill_candle.open, pos.is_long());
                                let exit_price = self
                                    .config
                                    .base
                                    .apply_exit_spread(exit_price_slipped, pos.is_long());
                                let exit_comm = self
                                    .config
                                    .base
                                    .calculate_commission(pos.quantity, exit_price);
                                let exit_tax = self.config.base.calculate_transaction_tax(
                                    exit_price * pos.quantity,
                                    !pos.is_long(),
                                );
                                let trade = pos.close_with_tax(
                                    fill_candle.timestamp,
                                    exit_price,
                                    exit_comm,
                                    exit_tax,
                                    signal.clone(),
                                );
                                if trade.is_long() {
                                    cash += trade.exit_value() - exit_comm
                                        + trade.unreinvested_dividends;
                                } else {
                                    cash -= trade.exit_value() + exit_comm + exit_tax
                                        - trade.unreinvested_dividends;
                                }
                                state.realized_pnl += trade.pnl;
                                state.trades.push(trade);
                                state.hwm = None;
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
                                    let fill_price = self.config.base.apply_entry_spread(
                                        self.config
                                            .base
                                            .apply_entry_slippage(fill_candle.open, is_long),
                                        is_long,
                                    );
                                    if fill_price <= 0.0 {
                                        return false;
                                    }
                                    let add_value = portfolio_equity * fraction;
                                    let add_qty = add_value / fill_price;
                                    let commission =
                                        self.config.base.calculate_commission(add_qty, fill_price);
                                    let entry_tax = self
                                        .config
                                        .base
                                        .calculate_transaction_tax(add_value, is_long);
                                    let total_cost = if is_long {
                                        add_value + commission + entry_tax
                                    } else {
                                        commission
                                    };
                                    if add_qty <= 0.0 || total_cost > cash {
                                        return false;
                                    }
                                    if is_long {
                                        cash -= add_value + commission + entry_tax;
                                    } else {
                                        cash += add_value - commission;
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
                            let pos_meta =
                                state.position.as_ref().map(|p| (p.is_long(), p.quantity));
                            match (state.candles.get(candle_idx + 1), pos_meta) {
                                (Some(fill_candle), Some((is_long, qty_full))) => {
                                    let exit_price = self.config.base.apply_exit_spread(
                                        self.config
                                            .base
                                            .apply_exit_slippage(fill_candle.open, is_long),
                                        is_long,
                                    );
                                    let qty_to_close = if fraction >= 1.0 {
                                        qty_full
                                    } else {
                                        qty_full * fraction
                                    };
                                    let commission = self
                                        .config
                                        .base
                                        .calculate_commission(qty_to_close, exit_price);
                                    let exit_tax = self.config.base.calculate_transaction_tax(
                                        exit_price * qty_to_close,
                                        !is_long,
                                    );
                                    let trade = if fraction >= 1.0 {
                                        let pos = state.position.take().unwrap();
                                        state.hwm = None;
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
                                        cash += trade.exit_value() - commission
                                            + trade.unreinvested_dividends;
                                    } else {
                                        cash -= trade.exit_value() + commission + exit_tax
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

            let open_positions_count: usize =
                states.values().filter(|s| s.position.is_some()).count();
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
                if let Some(max) = self.config.max_total_positions
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

                if signal.direction == SignalDirection::Short && !self.config.base.allow_short {
                    continue;
                }

                let is_long = signal.direction == SignalDirection::Long;
                let target_capital =
                    self.config
                        .allocation_target(&sym, cash, initial_capital, n_symbols);

                if target_capital <= 0.0 {
                    continue;
                }

                let entry_price_slipped = self.config.base.apply_entry_slippage(fill_open, is_long);
                // Spread is applied after slippage so that entry_price already
                // embeds the half-spread cost; no extra spread term is needed in
                // the denominator below.
                let entry_price = self
                    .config
                    .base
                    .apply_entry_spread(entry_price_slipped, is_long);

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
                // guard (`entry_cost > cash`) to catch any over-allocation.
                let (flat_reserve, pct_friction) = if self.config.base.commission_fn.is_some() {
                    (0.0, 0.0)
                } else {
                    (self.config.base.commission, self.config.base.commission_pct)
                };
                let tax_friction = if is_long {
                    self.config.base.transaction_tax_pct
                } else {
                    0.0
                };
                let effective_target = (target_capital - flat_reserve).max(0.0);
                let quantity =
                    effective_target / (entry_price * (1.0 + pct_friction + tax_friction));
                let entry_comm = self.config.base.calculate_commission(quantity, entry_price);
                let entry_tax = self
                    .config
                    .base
                    .calculate_transaction_tax(entry_price * quantity, is_long);
                let entry_cost = entry_price * quantity + entry_comm + entry_tax;

                if is_long {
                    if entry_cost > cash {
                        continue;
                    }
                } else if entry_comm > cash {
                    continue;
                }

                // ── Write phase: all immutable borrows of `states` are gone ────
                if is_long {
                    cash -= entry_cost;
                } else {
                    cash += entry_price * quantity - entry_comm;
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

            // --- Step 4: Record portfolio equity and allocation snapshot --------
            let portfolio_equity = compute_portfolio_equity(cash, &states, timestamp);

            if portfolio_equity > portfolio_peak {
                portfolio_peak = portfolio_equity;
            }
            let drawdown_pct = if portfolio_peak > 0.0 {
                (portfolio_peak - portfolio_equity) / portfolio_peak
            } else {
                0.0
            };

            portfolio_equity_curve.push(EquityPoint {
                timestamp,
                equity: portfolio_equity,
                drawdown_pct,
            });

            // Record per-symbol equity curves for symbols active this bar
            for sym in &active_symbols {
                let state = states.get_mut(sym).unwrap();
                let candle_idx = state.ts_index[&timestamp];
                let close = state.candles[candle_idx].close;
                let unrealized = state
                    .position
                    .as_ref()
                    .map(|pos| pos.unrealized_pnl(close))
                    .unwrap_or(0.0);
                let sym_equity = state.sym_initial_capital + state.realized_pnl + unrealized;
                if sym_equity > state.sym_peak {
                    state.sym_peak = sym_equity;
                }
                let sym_drawdown = if state.sym_peak > 0.0 {
                    (state.sym_peak - sym_equity) / state.sym_peak
                } else {
                    0.0
                };
                state.equity_curve.push(EquityPoint {
                    timestamp,
                    equity: sym_equity,
                    drawdown_pct: sym_drawdown,
                });
            }

            // Record allocation snapshot
            let position_values: HashMap<String, f64> = states
                .iter()
                .filter_map(|(sym, s)| {
                    s.position.as_ref().and_then(|pos| {
                        close_at_or_before(s, timestamp).map(|close| {
                            (
                                sym.clone(),
                                pos.current_value(close) + pos.unreinvested_dividends,
                            )
                        })
                    })
                })
                .collect();

            allocation_history.push(AllocationSnapshot {
                timestamp,
                cash,
                positions: position_values,
            });
        }

        // ── Close any remaining open positions at end ──────────────────────────
        if self.config.base.close_at_end {
            for state in states.values_mut() {
                if let Some(pos) = state.position.take() {
                    let last_candle = state.candles.last().unwrap();
                    let exit_price_slipped = self
                        .config
                        .base
                        .apply_exit_slippage(last_candle.close, pos.is_long());
                    let exit_price = self
                        .config
                        .base
                        .apply_exit_spread(exit_price_slipped, pos.is_long());
                    let exit_comm = self
                        .config
                        .base
                        .calculate_commission(pos.quantity, exit_price);
                    let exit_tax = self
                        .config
                        .base
                        .calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());
                    let exit_signal = Signal::exit(last_candle.timestamp, last_candle.close)
                        .with_reason("End of backtest");
                    let trade = pos.close_with_tax(
                        last_candle.timestamp,
                        exit_price,
                        exit_comm,
                        exit_tax,
                        exit_signal,
                    );
                    if trade.is_long() {
                        cash += trade.exit_value() - exit_comm + trade.unreinvested_dividends;
                    } else {
                        cash -= trade.exit_value() + exit_comm + exit_tax
                            - trade.unreinvested_dividends;
                    }
                    state.realized_pnl += trade.pnl;
                    state.trades.push(trade);
                    state.hwm = None;

                    let sym_equity = state.sym_initial_capital + state.realized_pnl;
                    sync_terminal_equity_point(
                        &mut state.equity_curve,
                        last_candle.timestamp,
                        sym_equity,
                    );
                }
            }
        }

        // ── Final equity ───────────────────────────────────────────────────────
        let final_equity: f64 = cash
            + states
                .values()
                .map(|s| {
                    s.position
                        .as_ref()
                        .zip(s.candles.last())
                        .map(|(pos, c)| pos.current_value(c.close) + pos.unreinvested_dividends)
                        .unwrap_or(0.0)
                })
                .sum::<f64>();

        if let Some(last_ts) = master_timeline.last().copied() {
            sync_terminal_equity_point(&mut portfolio_equity_curve, last_ts, final_equity);
        }

        // ── Build per-symbol BacktestResult ────────────────────────────────────
        let symbol_results: HashMap<String, BacktestResult> = states
            .into_iter()
            .map(|(sym, state)| {
                // Per-symbol final equity: sym_initial_capital + all realized P&L + open position value.
                // sym_initial_capital is the expected allocation (not the full portfolio capital),
                // so return %, Sharpe, etc. correctly reflect per-symbol performance.
                let sym_final_equity = state
                    .equity_curve
                    .last()
                    .map(|ep| ep.equity)
                    .unwrap_or(state.sym_initial_capital);

                let exec_count = state.signals.iter().filter(|s| s.executed).count();
                let metrics = PerformanceMetrics::calculate(
                    &state.trades,
                    &state.equity_curve,
                    state.sym_initial_capital,
                    state.signals.len(),
                    exec_count,
                    self.config.base.risk_free_rate,
                    self.config.base.bars_per_year,
                );

                let start_ts = state.candles.first().map(|c| c.timestamp).unwrap_or(0);
                let end_ts = state.candles.last().map(|c| c.timestamp).unwrap_or(0);

                let result = BacktestResult {
                    symbol: sym.clone(),
                    strategy_name: state.strategy_name.clone(),
                    config: self.config.base.clone(),
                    start_timestamp: start_ts,
                    end_timestamp: end_ts,
                    initial_capital: state.sym_initial_capital,
                    final_equity: sym_final_equity,
                    metrics,
                    trades: state.trades,
                    equity_curve: state.equity_curve,
                    signals: state.signals,
                    open_position: state.position,
                    benchmark: None,
                    diagnostics: vec![],
                };

                (sym, result)
            })
            .collect();

        // ── Aggregate portfolio metrics ────────────────────────────────────────
        let all_trades: Vec<Trade> = symbol_results
            .values()
            .flat_map(|r| r.trades.iter().cloned())
            .collect();

        let total_signals: usize = symbol_results.values().map(|r| r.signals.len()).sum();
        let executed_signals: usize = symbol_results
            .values()
            .flat_map(|r| r.signals.iter())
            .filter(|s| s.executed)
            .count();

        let mut portfolio_metrics = PerformanceMetrics::calculate(
            &all_trades,
            &portfolio_equity_curve,
            initial_capital,
            total_signals,
            executed_signals,
            self.config.base.risk_free_rate,
            self.config.base.bars_per_year,
        );
        portfolio_metrics.time_in_market_pct =
            compute_portfolio_time_in_market(&allocation_history);

        Ok(PortfolioResult {
            symbols: symbol_results,
            portfolio_equity_curve,
            portfolio_metrics,
            initial_capital,
            final_equity,
            allocation_history,
        })
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Per-symbol simulation state (private to the engine).
struct SymbolState<S: Strategy> {
    candles: Vec<Candle>,
    dividends: Vec<Dividend>,
    ts_index: HashMap<i64, usize>,
    indicators: HashMap<String, Vec<Option<f64>>>,
    strategy: S,
    warmup: usize,
    position: Option<Position>,
    hwm: Option<f64>,
    div_idx: usize,
    trades: Vec<Trade>,
    signals: Vec<SignalRecord>,
    /// Cumulative realized P&L (net of commissions and dividends) from closed trades.
    realized_pnl: f64,
    /// Per-symbol equity curve: sym_initial_capital + realized_pnl + open position unrealized P&L.
    equity_curve: Vec<EquityPoint>,
    /// Running peak equity for per-symbol drawdown calculation.
    sym_peak: f64,
    /// Expected per-symbol capital allocation (derived from portfolio config at setup time).
    ///
    /// Used as the baseline for per-symbol equity, total_return_pct, Sharpe, etc.
    /// so that metrics reflect the actual allocation rather than the full portfolio
    /// initial_capital.
    sym_initial_capital: f64,
    /// Name of the strategy used for this symbol (for reporting).
    strategy_name: String,
}

/// Compute total portfolio equity: cash + sum of all open position values.
fn compute_portfolio_equity<S: Strategy>(
    cash: f64,
    states: &HashMap<String, SymbolState<S>>,
    timestamp: i64,
) -> f64 {
    cash + states
        .values()
        .filter_map(|s| {
            s.position.as_ref().and_then(|pos| {
                close_at_or_before(s, timestamp)
                    .map(|close| pos.current_value(close) + pos.unreinvested_dividends)
            })
        })
        .sum::<f64>()
}

fn close_at_or_before<S: Strategy>(state: &SymbolState<S>, timestamp: i64) -> Option<f64> {
    // Fast path: ts_index covers all candle timestamps.
    if let Some(&idx) = state.ts_index.get(&timestamp) {
        return Some(state.candles[idx].close);
    }
    // Slow path: timestamp falls between candle bars (e.g. portfolio timeline
    // has a bar this symbol does not trade on); return the most recent prior close.
    match state
        .candles
        .binary_search_by_key(&timestamp, |c| c.timestamp)
    {
        Ok(idx) | Err(idx) if idx > 0 => Some(state.candles[idx.saturating_sub(1)].close),
        _ => None,
    }
}

/// Fraction of portfolio backtest time with at least one open position.
///
/// Uses allocation snapshots and timestamp deltas so overlapping symbol
/// positions count once (union exposure), not once per symbol/trade.
fn compute_portfolio_time_in_market(allocation_history: &[AllocationSnapshot]) -> f64 {
    if allocation_history.len() < 2 {
        return 0.0;
    }

    let total_span = allocation_history.last().map(|s| s.timestamp).unwrap_or(0)
        - allocation_history.first().map(|s| s.timestamp).unwrap_or(0);

    if total_span <= 0 {
        return 0.0;
    }

    let mut exposed_secs: i64 = 0;
    for window in allocation_history.windows(2) {
        let current = &window[0];
        let next = &window[1];
        if !current.positions.is_empty() {
            exposed_secs += (next.timestamp - current.timestamp).max(0);
        }
    }

    (exposed_secs as f64 / total_span as f64).clamp(0.0, 1.0)
}

/// Check stop-loss, take-profit, and trailing stop for an open position.
///
/// Uses `candle.low` / `candle.high` to detect intrabar breaches.  Returns an
/// exit [`Signal`] whose `price` is the computed fill price (stop/TP level with a
/// gap-guard so the fill is never better than what the market provided).
///
/// # Exit Priority
///
/// When multiple conditions breach on the same bar the evaluation order is
/// **stop-loss → take-profit → trailing stop**.  The intrabar sequence is
/// unknowable from OHLCV bars alone, so stop-loss is given priority for
/// conservative simulation.  Strategies with SL and TP both active should
/// keep those levels well separated relative to typical bar ranges.
fn check_sl_tp(
    pos: &Position,
    candle: &Candle,
    hwm: Option<f64>,
    config: &BacktestConfig,
) -> Option<Signal> {
    // Stop-loss
    if let Some(sl_pct) = config.stop_loss_pct {
        let stop_price = if pos.is_long() {
            pos.entry_price * (1.0 - sl_pct)
        } else {
            pos.entry_price * (1.0 + sl_pct)
        };
        let triggered = if pos.is_long() {
            candle.low <= stop_price
        } else {
            candle.high >= stop_price
        };
        if triggered {
            let fill_price = if pos.is_long() {
                candle.open.min(stop_price)
            } else {
                candle.open.max(stop_price)
            };
            let return_pct = pos.unrealized_return_pct(fill_price);
            return Some(
                Signal::exit(candle.timestamp, fill_price)
                    .with_reason(format!("Stop-loss triggered ({:.1}%)", return_pct)),
            );
        }
    }

    // Take-profit
    if let Some(tp_pct) = config.take_profit_pct {
        let tp_price = if pos.is_long() {
            pos.entry_price * (1.0 + tp_pct)
        } else {
            pos.entry_price * (1.0 - tp_pct)
        };
        let triggered = if pos.is_long() {
            candle.high >= tp_price
        } else {
            candle.low <= tp_price
        };
        if triggered {
            let fill_price = if pos.is_long() {
                candle.open.max(tp_price)
            } else {
                candle.open.min(tp_price)
            };
            let return_pct = pos.unrealized_return_pct(fill_price);
            return Some(
                Signal::exit(candle.timestamp, fill_price)
                    .with_reason(format!("Take-profit triggered ({:.1}%)", return_pct)),
            );
        }
    }

    // Trailing stop — `hwm` is already updated to the intrabar extreme before this call.
    if let Some(trail_pct) = config.trailing_stop_pct
        && let Some(extreme) = hwm
        && extreme > 0.0
    {
        let trail_stop_price = if pos.is_long() {
            extreme * (1.0 - trail_pct)
        } else {
            extreme * (1.0 + trail_pct)
        };
        let triggered = if pos.is_long() {
            candle.low <= trail_stop_price
        } else {
            candle.high >= trail_stop_price
        };
        if triggered {
            let fill_price = if pos.is_long() {
                candle.open.min(trail_stop_price)
            } else {
                candle.open.max(trail_stop_price)
            };
            let adverse_move_pct = if pos.is_long() {
                (extreme - fill_price) / extreme
            } else {
                (fill_price - extreme) / extreme
            };
            return Some(
                Signal::exit(candle.timestamp, fill_price).with_reason(format!(
                    "Trailing stop triggered ({:.1}% adverse move)",
                    adverse_move_pct * 100.0
                )),
            );
        }
    }

    None
}

fn sync_terminal_equity_point(equity_curve: &mut Vec<EquityPoint>, timestamp: i64, equity: f64) {
    if let Some(last) = equity_curve.last_mut()
        && last.timestamp == timestamp
    {
        last.equity = equity;
    } else {
        equity_curve.push(EquityPoint {
            timestamp,
            equity,
            drawdown_pct: 0.0,
        });
    }

    let peak = equity_curve
        .iter()
        .map(|point| point.equity)
        .fold(f64::NEG_INFINITY, f64::max);
    let drawdown = if peak.is_finite() && peak > 0.0 {
        (peak - equity) / peak
    } else {
        0.0
    };

    if let Some(last) = equity_curve.last_mut() {
        last.drawdown_pct = drawdown;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::portfolio::config::{PortfolioConfig, RebalanceMode};
    use crate::backtesting::strategy::{Strategy, StrategyContext};
    use crate::backtesting::{BacktestConfig, SmaCrossover};
    use crate::indicators::Indicator;

    #[derive(Clone)]
    struct EnterShortHold;

    impl Strategy for EnterShortHold {
        fn name(&self) -> &str {
            "Enter Short Hold"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    #[derive(Clone)]
    struct TimedLongStrategy {
        entry_idx: usize,
        exit_idx: usize,
    }

    impl Strategy for TimedLongStrategy {
        fn name(&self) -> &str {
            "Timed Long"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == self.entry_idx && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else if ctx.index == self.exit_idx && ctx.has_position() {
                Signal::exit(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| Candle {
                timestamp: i as i64 * 86400,
                open: p,
                high: p * 1.005,
                low: p * 0.995,
                close: p,
                volume: 1_000,
                adj_close: Some(p),
            })
            .collect()
    }

    fn make_candles_with_timestamps(prices: &[f64], timestamps: &[i64]) -> Vec<Candle> {
        prices
            .iter()
            .zip(timestamps.iter())
            .map(|(&p, &ts)| Candle {
                timestamp: ts,
                open: p,
                high: p * 1.005,
                low: p * 0.995,
                close: p,
                volume: 1_000,
                adj_close: Some(p),
            })
            .collect()
    }

    #[derive(Clone)]
    struct FirstBarLongElseHold {
        enabled: bool,
    }

    impl Strategy for FirstBarLongElseHold {
        fn name(&self) -> &str {
            "First Bar Long"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if self.enabled && ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    fn trending_prices(n: usize, start: f64, rate: f64) -> Vec<f64> {
        (0..n).map(|i| start + i as f64 * rate).collect()
    }

    #[test]
    fn test_two_symbol_basic() {
        let prices_a = trending_prices(100, 100.0, 0.5);
        let prices_b = trending_prices(100, 50.0, 0.25);

        let symbol_data = vec![
            SymbolData::new("AAPL", make_candles(&prices_a)),
            SymbolData::new("MSFT", make_candles(&prices_b)),
        ];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(20_000.0)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .build()
                .unwrap(),
        )
        .max_total_positions(2);

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |_| SmaCrossover::new(5, 20))
            .unwrap();

        assert!(result.symbols.contains_key("AAPL"));
        assert!(result.symbols.contains_key("MSFT"));
        assert!(result.final_equity > 0.0);
        assert!(!result.portfolio_equity_curve.is_empty());
    }

    #[test]
    fn test_max_total_positions_respected() {
        // Two strongly trending symbols; with max_positions=1 only one should trade
        let prices = trending_prices(100, 100.0, 1.0);
        let symbol_data = vec![
            SymbolData::new("A", make_candles(&prices)),
            SymbolData::new("B", make_candles(&prices)),
        ];

        let config = PortfolioConfig::new(BacktestConfig::default()).max_total_positions(1);

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |_| SmaCrossover::new(5, 20))
            .unwrap();

        // At any time only one symbol should be open — total concurrent positions ≤ 1
        for snapshot in &result.allocation_history {
            assert!(
                snapshot.positions.len() <= 1,
                "more than 1 position open at timestamp {}",
                snapshot.timestamp
            );
        }
    }

    #[test]
    fn test_equal_weight_allocation() {
        let prices = trending_prices(100, 100.0, 0.5);
        let symbol_data = vec![
            SymbolData::new("X", make_candles(&prices)),
            SymbolData::new("Y", make_candles(&prices)),
        ];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(10_000.0)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .build()
                .unwrap(),
        )
        .rebalance(RebalanceMode::EqualWeight)
        .max_total_positions(2);

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |_| SmaCrossover::new(5, 20))
            .unwrap();

        // Portfolio should have run without error
        assert!(result.final_equity > 0.0);
    }

    #[test]
    fn test_dividend_credited() {
        let prices = trending_prices(50, 100.0, 0.2);
        let dividends = vec![
            Dividend {
                timestamp: 20 * 86400,
                amount: 1.0,
            },
            Dividend {
                timestamp: 40 * 86400,
                amount: 1.0,
            },
        ];
        let symbol_data =
            vec![SymbolData::new("DIV", make_candles(&prices)).with_dividends(dividends)];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .build()
                .unwrap(),
        );

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |_| SmaCrossover::new(5, 20))
            .unwrap();

        // Total dividend income across all trades should be non-negative
        let total_div: f64 = result.symbols["DIV"]
            .trades
            .iter()
            .map(|t| t.dividend_income)
            .sum();
        assert!(total_div >= 0.0);
    }

    #[test]
    fn test_empty_symbol_data_fails() {
        let config = PortfolioConfig::default();
        let engine = PortfolioEngine::new(config);
        assert!(
            engine
                .run::<SmaCrossover, _>(&[], |_| SmaCrossover::new(5, 20))
                .is_err()
        );
    }

    #[test]
    fn test_short_dividend_is_liability() {
        let prices = vec![100.0, 100.0, 100.0];
        let candles = make_candles(&prices);
        let dividends = vec![Dividend {
            timestamp: candles[1].timestamp,
            amount: 1.0,
        }];
        let symbol_data = vec![SymbolData::new("DIVS", candles).with_dividends(dividends)];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(10_000.0)
                .allow_short(true)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .build()
                .unwrap(),
        );

        let engine = PortfolioEngine::new(config);
        let result = engine.run(&symbol_data, |_| EnterShortHold).unwrap();

        let trades = &result.symbols["DIVS"].trades;
        assert_eq!(trades.len(), 1);
        assert!(trades[0].dividend_income < 0.0);
        assert!(result.final_equity < 10_000.0);
    }

    #[test]
    fn test_portfolio_time_in_market_uses_union_exposure() {
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];

        let symbol_data = vec![
            SymbolData::new("A", make_candles(&prices)),
            SymbolData::new("B", make_candles(&prices)),
        ];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(10_000.0)
                .position_size_pct(0.5)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .close_at_end(false)
                .build()
                .unwrap(),
        )
        .max_total_positions(2);

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |sym| {
                if sym == "A" {
                    TimedLongStrategy {
                        entry_idx: 0,
                        exit_idx: 2,
                    }
                } else {
                    TimedLongStrategy {
                        entry_idx: 1,
                        exit_idx: 3,
                    }
                }
            })
            .unwrap();

        // Union exposure spans [t0, t3] over total [t0, t4] => 3/4 = 0.75.
        // A per-trade sum approach would overstate this to 1.0 (clipped).
        let actual = result.portfolio_metrics.time_in_market_pct;
        assert!(
            (actual - 0.75).abs() < 1e-9,
            "expected 0.75 union exposure, got {actual}"
        );
    }

    #[test]
    fn test_portfolio_marks_open_positions_on_sparse_timestamps() {
        let symbol_data = vec![
            SymbolData::new("A", make_candles_with_timestamps(&[100.0, 110.0], &[0, 2])),
            SymbolData::new(
                "B",
                make_candles_with_timestamps(&[50.0, 50.0, 50.0], &[0, 1, 2]),
            ),
        ];

        let config = PortfolioConfig::new(
            BacktestConfig::builder()
                .initial_capital(10_000.0)
                .position_size_pct(1.0)
                .commission_pct(0.0)
                .slippage_pct(0.0)
                .close_at_end(false)
                .build()
                .unwrap(),
        )
        .max_total_positions(2);

        let engine = PortfolioEngine::new(config);
        let result = engine
            .run(&symbol_data, |sym| FirstBarLongElseHold {
                enabled: sym == "A",
            })
            .unwrap();

        let snapshot_t1 = result
            .allocation_history
            .iter()
            .find(|s| s.timestamp == 1)
            .expect("snapshot at timestamp 1");
        assert!(
            snapshot_t1.positions.contains_key("A"),
            "open A position should be valued at t=1"
        );
        // Entry fills at next-bar open (110), valued at close_at_or_before(t=1)=100.
        // Equity ≈ 9091 (not ~10000): the key property is the position is included,
        // not that there's no slippage from fill-bar price differences.
        assert!(
            snapshot_t1.total_equity() > 8_000.0,
            "equity should include carried-forward A valuation, got {}",
            snapshot_t1.total_equity()
        );
    }
}
