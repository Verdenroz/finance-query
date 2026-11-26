//! Per-symbol simulation state and shared account arithmetic.

use std::collections::HashMap;

use crate::backtesting::engine::SizingSeries;
use crate::backtesting::position::{Position, Trade};
use crate::backtesting::result::{EquityPoint, SignalRecord};
use crate::backtesting::strategy::Strategy;
use crate::models::chart::{Candle, Dividend};

/// Per-symbol simulation state (private to the engine).
pub(super) struct SymbolState<S: Strategy> {
    pub(super) candles: Vec<Candle>,
    pub(super) dividends: Vec<Dividend>,
    pub(super) ts_index: HashMap<i64, usize>,
    pub(super) indicators: HashMap<String, Vec<Option<f64>>>,
    pub(super) sizing_series: SizingSeries,
    pub(super) strategy: S,
    pub(super) warmup: usize,
    pub(super) position: Option<Position>,
    pub(super) hwm: Option<f64>,
    pub(super) extremes: Option<crate::backtesting::strategy::PositionExtremes>,
    pub(super) track_extremes: bool,
    pub(super) div_idx: usize,
    pub(super) trades: Vec<Trade>,
    pub(super) signals: Vec<SignalRecord>,
    /// Cumulative realized P&L (net of commissions and dividends) from closed trades.
    pub(super) realized_pnl: f64,
    /// Per-symbol equity curve: sym_initial_capital + realized_pnl + open position unrealized P&L.
    pub(super) equity_curve: Vec<EquityPoint>,
    /// Running peak equity for per-symbol drawdown calculation.
    pub(super) sym_peak: f64,
    /// Peak gross exposure over portfolio equity, not over `sym_initial_capital`:
    /// entries draw on the shared cash pool, which the static sleeve baseline
    /// does not track.
    pub(super) sym_max_leverage: f64,
    /// Expected per-symbol capital allocation (derived from portfolio config at setup time).
    ///
    /// Used as the baseline for per-symbol equity, total_return_pct, Sharpe, etc.
    /// so that metrics reflect the actual allocation rather than the full portfolio
    /// initial_capital.
    pub(super) sym_initial_capital: f64,
    /// Name of the strategy used for this symbol (for reporting).
    pub(super) strategy_name: String,
}

/// Notional available to a new entry: equity times `max_leverage` minus
/// marked gross exposure, so aggregate exposure stays within the ceiling.
///
/// Equity includes unreinvested dividends, matching
/// [`compute_portfolio_equity`] and the single-symbol `add_buying_power`. At
/// 1x with only longs this is raw cash plus those dividends (value and
/// exposure cancel); a short reserves twice its marked notional, since its
/// sale proceeds sit in cash while its exposure consumes the same headroom.
pub(super) fn compute_buying_power<S: Strategy>(
    cash: f64,
    states: &HashMap<String, SymbolState<S>>,
    timestamp: i64,
    max_leverage: f64,
) -> f64 {
    let (value, gross) = states
        .values()
        .filter_map(|s| {
            s.position.as_ref().and_then(|pos| {
                close_at_or_before(s, timestamp).map(|close| {
                    (
                        pos.current_value(close) + pos.unreinvested_dividends,
                        pos.quantity * close,
                    )
                })
            })
        })
        .fold((0.0, 0.0), |(v, g), (pv, pg)| (v + pv, g + pg));
    (cash + value) * max_leverage - gross
}

/// Compute total portfolio equity: cash + sum of all open position values.
pub(super) fn compute_portfolio_equity<S: Strategy>(
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

pub(super) fn close_at_or_before<S: Strategy>(
    state: &SymbolState<S>,
    timestamp: i64,
) -> Option<f64> {
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
