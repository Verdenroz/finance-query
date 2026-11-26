//! Multi-symbol portfolio backtesting engine.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::backtesting::engine::{BacktestEngine, update_position_extremes, update_trailing_hwm};
use crate::backtesting::error::{BacktestError, Result};
use crate::backtesting::result::EquityPoint;
use crate::backtesting::signal::Signal;
use crate::backtesting::strategy::Strategy;
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
            let warmup = strategy
                .warmup_period()
                .max(self.config.base.sizing_warmup());
            let track_extremes = strategy.tracks_position_extremes();
            if data.candles.len() < warmup {
                return Err(BacktestError::insufficient_data(warmup, data.candles.len()));
            }
            let strategy_name = strategy.name().to_string();
            let indicators = helper_engine.compute_indicators(&data.candles, &strategy)?;
            let sizing_series = helper_engine.compute_sizing_series(&data.candles);
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
                self.config.base.position_size_pct * self.config.base.max_leverage,
            );

            states.insert(
                data.symbol.clone(),
                SymbolState {
                    candles: data.candles.clone(),
                    dividends: data.dividends.clone(),
                    ts_index,
                    indicators,
                    sizing_series,
                    strategy,
                    warmup,
                    position: None,
                    hwm: None,
                    extremes: None,
                    track_extremes,
                    div_idx: 0,
                    trades: vec![],
                    signals: vec![],
                    realized_pnl: 0.0,
                    equity_curve: vec![],
                    sym_peak: sym_initial_capital,
                    sym_max_leverage: 0.0,
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
        let financing_enabled =
            self.config.base.short_borrow_rate > 0.0 || self.config.base.margin_interest_rate > 0.0;
        let margin_enabled = self.config.base.max_leverage > 1.0 || self.config.base.allow_short;
        let per_bar = 1.0 / self.config.base.bars_per_year;

        // ── Main simulation loop ───────────────────────────────────────────────
        for &timestamp in &master_timeline {
            // Collect present symbols for this bar (parallel mutable iteration
            // is not possible, so we collect keys then iterate)
            let mut active_symbols: Vec<String> = states
                .keys()
                .filter(|sym| states[*sym].ts_index.contains_key(&timestamp))
                .cloned()
                .collect();
            // HashMap iteration order is unspecified; sort so ScaleIn/ScaleOut
            // cash contention resolves the same way on every run.
            active_symbols.sort();

            // Margin interest on a debit cash balance, split across open
            // positions by gross exposure so it exits through their trades.
            // Skipped when flat, matching the single-symbol engine.
            if financing_enabled && self.config.base.margin_interest_rate > 0.0 {
                let interest = (-cash).max(0.0) * self.config.base.margin_interest_rate * per_bar;
                if interest > 0.0 {
                    let grosses: Vec<(String, f64)> = states
                        .iter()
                        .filter_map(|(sym, s)| {
                            s.position.as_ref().and_then(|pos| {
                                close_at_or_before(s, timestamp)
                                    .map(|close| (sym.clone(), pos.quantity * close))
                            })
                        })
                        .collect();
                    let gross_total: f64 = grosses.iter().map(|(_, g)| g).sum();
                    if gross_total > 0.0 {
                        cash -= interest;
                        for (sym, gross) in grosses {
                            if let Some(pos) = states.get_mut(&sym).unwrap().position.as_mut() {
                                pos.accrue_financing_cost(interest * gross / gross_total);
                            }
                        }
                    }
                }
            }

            // --- Step 1: Update position values, dividends, trailing stops ----
            let mut auto_exits: Vec<(String, Signal)> = Vec::new();

            for sym in &active_symbols {
                let state = states.get_mut(sym).unwrap();
                let candle_idx = state.ts_index[&timestamp];
                let close = state.candles[candle_idx].close;

                if financing_enabled
                    && let Some(pos) = state.position.as_mut()
                    && pos.is_short()
                {
                    let fee = pos.quantity * close * self.config.base.short_borrow_rate * per_bar;
                    if fee > 0.0 {
                        cash -= fee;
                        pos.accrue_financing_cost(fee);
                    }
                }

                let candle = &state.candles[candle_idx];

                if state.track_extremes {
                    update_position_extremes(state.position.as_ref(), &mut state.extremes, candle);
                }

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

                // Check SL/TP/trailing stop against the hwm as of the prior bar,
                // before this bar's own high/low is folded in below.
                if let Some(ref pos) = state.position
                    && let Some(exit_signal) =
                        check_sl_tp(pos, candle, state.hwm, &self.config.base)
                {
                    auto_exits.push((sym.clone(), exit_signal));
                }

                update_trailing_hwm(state.position.as_ref(), &mut state.hwm, candle);
            }

            // Process auto-exits (SL/TP/trailing) — execute on the current bar at the
            // fill price embedded in the signal (stop/TP level with gap guard).
            let mut exited_this_bar: HashSet<String> = HashSet::new();
            for (sym, exit_signal) in auto_exits {
                let state = states.get_mut(&sym).unwrap();
                let fill_price = exit_signal.price;
                if execute_forced_exit(
                    &self.config.base,
                    state,
                    &mut cash,
                    timestamp,
                    fill_price,
                    exit_signal,
                ) {
                    exited_this_bar.insert(sym);
                }
            }

            // Account-level maintenance check, after the stops so an intrabar
            // stop on the same bar outranks the liquidation. Positions close
            // at the bar's close, largest exposure first, until equity covers
            // the requirement. An unlevered long-only book is never checked.
            if margin_enabled {
                loop {
                    let equity = compute_portfolio_equity(cash, &states, timestamp);
                    let mut gross_total = 0.0;
                    let mut any_short = false;
                    let mut largest: Option<(String, f64, f64)> = None;
                    for (sym, s) in &states {
                        if let Some(pos) = s.position.as_ref()
                            && let Some(close) = close_at_or_before(s, timestamp)
                        {
                            let gross = pos.quantity * close;
                            gross_total += gross;
                            any_short |= pos.is_short();
                            let replace = match &largest {
                                Some((lsym, lgross, _)) => {
                                    gross > *lgross || (gross == *lgross && sym < lsym)
                                }
                                None => true,
                            };
                            if replace {
                                largest = Some((sym.clone(), gross, close));
                            }
                        }
                    }
                    let checked = self.config.base.max_leverage > 1.0 || any_short;
                    if !checked
                        || gross_total <= 0.0
                        || equity >= self.config.base.maintenance_margin_pct * gross_total
                    {
                        break;
                    }
                    let Some((sym, _, close)) = largest else {
                        break;
                    };
                    let exit_signal = Signal::exit(timestamp, close)
                        .with_reason("Margin call: equity below maintenance margin requirement");
                    let state = states.get_mut(&sym).unwrap();
                    execute_forced_exit(
                        &self.config.base,
                        state,
                        &mut cash,
                        timestamp,
                        close,
                        exit_signal,
                    );
                    exited_this_bar.insert(sym);
                }
            }

            // --- Step 2: strategy signals; exits/scales execute, entries queue ---
            let pending_entries = dispatch_bar_signals(
                &self.config,
                &mut states,
                &active_symbols,
                &exited_this_bar,
                timestamp,
                &mut cash,
            );

            // --- Step 3: Open entry positions (highest strength first) ----------
            open_pending_entries(
                &self.config,
                &helper_engine,
                &mut states,
                pending_entries,
                timestamp,
                initial_capital,
                n_symbols,
                &mut cash,
            );

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
                if portfolio_equity > 0.0
                    && let Some(pos) = state.position.as_ref()
                {
                    let leverage = pos.quantity * close / portfolio_equity;
                    state.sym_max_leverage = state.sym_max_leverage.max(leverage);
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
                    state.extremes = None;

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

        Ok(build_portfolio_result(
            &self.config,
            states,
            portfolio_equity_curve,
            allocation_history,
            initial_capital,
            final_equity,
        ))
    }
}

mod entries;
mod exits;
mod report;
mod signals;
mod state;

use self::entries::open_pending_entries;
use self::exits::execute_forced_exit;
use self::report::{build_portfolio_result, sync_terminal_equity_point};
use self::signals::dispatch_bar_signals;
use self::state::{SymbolState, close_at_or_before, compute_portfolio_equity};
use crate::backtesting::engine::check_sl_tp;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_margin;
