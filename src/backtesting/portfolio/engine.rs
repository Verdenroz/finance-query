//! Multi-symbol portfolio backtesting engine.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use crate::backtesting::config::BacktestConfig;
use crate::backtesting::engine::BacktestEngine;
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

                // Update HWM for trailing stop
                if let Some(ref pos) = state.position {
                    state.hwm = Some(match state.hwm {
                        None => candle.close,
                        Some(prev) => {
                            if pos.is_long() {
                                prev.max(candle.close)
                            } else {
                                prev.min(candle.close)
                            }
                        }
                    });
                } else {
                    state.hwm = None;
                }

                // Credit dividends ex-dated on or before this bar
                while state.div_idx < state.dividends.len()
                    && state.dividends[state.div_idx].timestamp <= timestamp
                {
                    if let Some(ref mut pos) = state.position {
                        let income = state.dividends[state.div_idx].amount * pos.quantity;
                        if self.config.base.reinvest_dividends && candle.close > 0.0 {
                            pos.quantity += income / candle.close;
                        }
                        pos.dividend_income += income;
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

            // Process auto-exits (SL/TP/trailing) — these bypass strategy signals
            let mut exited_this_bar: Vec<String> = Vec::new();
            for (sym, exit_signal) in auto_exits {
                let state = states.get_mut(&sym).unwrap();
                let candle_idx = state.ts_index[&timestamp];
                let candle = &state.candles[candle_idx];

                let Some(pos) = state.position.take() else {
                    continue;
                };
                let exit_price = self
                    .config
                    .base
                    .apply_exit_slippage(candle.close, pos.is_long());
                let exit_comm = self
                    .config
                    .base
                    .calculate_commission(exit_price * pos.quantity);
                let trade = pos.close(timestamp, exit_price, exit_comm, exit_signal);
                cash += trade.entry_value() + trade.pnl;
                state.realized_pnl += trade.pnl;
                state.trades.push(trade);
                state.hwm = None;
                state.signals.push(SignalRecord {
                    timestamp,
                    price: candle.close,
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                    reason: Some("SL/TP/Trailing stop".to_string()),
                    executed: true,
                });
                exited_this_bar.push(sym);
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
                let candle = &state.candles[candle_idx];

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
                    });
                    continue;
                }

                match signal.direction {
                    SignalDirection::Exit => {
                        // Immediate exit: close open position
                        if let Some(pos) = state.position.take() {
                            let exit_price = self
                                .config
                                .base
                                .apply_exit_slippage(candle.close, pos.is_long());
                            let exit_comm = self
                                .config
                                .base
                                .calculate_commission(exit_price * pos.quantity);
                            let trade = pos.close(timestamp, exit_price, exit_comm, signal.clone());
                            cash += trade.entry_value() + trade.pnl;
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
                            });
                        }
                    }
                    SignalDirection::Long | SignalDirection::Short => {
                        // Queue for priority-ordered entry
                        pending_entries.push((sym.clone(), signal));
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
                let (has_position, close) = {
                    let state = states.get(&sym).unwrap();
                    let idx = state.ts_index[&timestamp];
                    (state.position.is_some(), state.candles[idx].close)
                }; // immutable borrow on `states` ends here

                if has_position {
                    continue;
                }

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

                let entry_price = self.config.base.apply_entry_slippage(close, is_long);
                // Reserve the flat fee before computing quantity so that
                // entry_cost == target_capital when both flat and pct fees are
                // set. Without this, entry_cost > target_capital and the entry
                // is silently rejected whenever a flat commission is configured.
                let effective_target = (target_capital - self.config.base.commission).max(0.0);
                let quantity =
                    effective_target / (entry_price * (1.0 + self.config.base.commission_pct));
                let entry_comm = self
                    .config
                    .base
                    .calculate_commission(entry_price * quantity);
                let entry_cost = entry_price * quantity + entry_comm;

                if entry_cost > cash {
                    continue;
                }

                // ── Write phase: all immutable borrows of `states` are gone ────
                cash -= entry_cost;
                let side = if is_long {
                    PositionSide::Long
                } else {
                    PositionSide::Short
                };

                let state = states.get_mut(&sym).unwrap();
                state.position = Some(Position::new(
                    side,
                    timestamp,
                    entry_price,
                    quantity,
                    entry_comm,
                    signal.clone(),
                ));
                state.hwm = Some(close);
                state.signals.push(SignalRecord {
                    timestamp: signal.timestamp,
                    price: signal.price,
                    direction: signal.direction,
                    strength: signal.strength.value(),
                    reason: signal.reason,
                    executed: true,
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
                        s.ts_index
                            .get(&timestamp)
                            .map(|&idx| (sym.clone(), pos.current_value(s.candles[idx].close)))
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
                    let exit_price = self
                        .config
                        .base
                        .apply_exit_slippage(last_candle.close, pos.is_long());
                    let exit_comm = self
                        .config
                        .base
                        .calculate_commission(exit_price * pos.quantity);
                    let exit_signal = Signal::exit(last_candle.timestamp, last_candle.close)
                        .with_reason("End of backtest");
                    let trade =
                        pos.close(last_candle.timestamp, exit_price, exit_comm, exit_signal);
                    cash += trade.entry_value() + trade.pnl;
                    state.realized_pnl += trade.pnl;
                    state.trades.push(trade);
                    state.hwm = None;
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
                        .map(|(pos, c)| pos.current_value(c.close))
                        .unwrap_or(0.0)
                })
                .sum::<f64>();

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

        let portfolio_metrics = PerformanceMetrics::calculate(
            &all_trades,
            &portfolio_equity_curve,
            initial_capital,
            total_signals,
            executed_signals,
            self.config.base.risk_free_rate,
            self.config.base.bars_per_year,
        );

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
                s.ts_index
                    .get(&timestamp)
                    .map(|&idx| pos.current_value(s.candles[idx].close))
            })
        })
        .sum::<f64>()
}

/// Check stop-loss, take-profit, and trailing stop for an open position.
///
/// Returns an exit signal if any trigger fires; `None` otherwise.
fn check_sl_tp(
    pos: &Position,
    candle: &Candle,
    hwm: Option<f64>,
    config: &BacktestConfig,
) -> Option<Signal> {
    let entry = pos.entry_price;
    let price = candle.close;

    // Stop-loss
    if let Some(sl) = config.stop_loss_pct {
        let triggered = if pos.is_long() {
            price <= entry * (1.0 - sl)
        } else {
            price >= entry * (1.0 + sl)
        };
        if triggered {
            return Some(Signal::exit(candle.timestamp, price).with_reason("Stop-loss triggered"));
        }
    }

    // Take-profit
    if let Some(tp) = config.take_profit_pct {
        let triggered = if pos.is_long() {
            price >= entry * (1.0 + tp)
        } else {
            price <= entry * (1.0 - tp)
        };
        if triggered {
            return Some(
                Signal::exit(candle.timestamp, price).with_reason("Take-profit triggered"),
            );
        }
    }

    // Trailing stop
    if let (Some(trail), Some(peak)) = (config.trailing_stop_pct, hwm) {
        let triggered = if pos.is_long() {
            price <= peak * (1.0 - trail)
        } else {
            price >= peak * (1.0 + trail)
        };
        if triggered {
            return Some(
                Signal::exit(candle.timestamp, price).with_reason("Trailing stop triggered"),
            );
        }
    }

    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::portfolio::config::{PortfolioConfig, RebalanceMode};
    use crate::backtesting::{BacktestConfig, SmaCrossover};

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
}
