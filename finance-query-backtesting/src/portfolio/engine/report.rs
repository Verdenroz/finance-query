//! Portfolio-level curve, metric, and result-assembly helpers.

use std::collections::HashMap;

use crate::backtesting::position::Trade;
use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics};
use crate::backtesting::strategy::Strategy;

use super::super::config::PortfolioConfig;
use super::super::result::{AllocationSnapshot, PortfolioResult};
use super::state::SymbolState;

/// Fraction of portfolio backtest time with at least one open position.
///
/// Uses allocation snapshots and timestamp deltas so overlapping symbol
/// positions count once (union exposure), not once per symbol/trade.
pub(super) fn compute_portfolio_time_in_market(allocation_history: &[AllocationSnapshot]) -> f64 {
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

pub(super) fn sync_terminal_equity_point(
    equity_curve: &mut Vec<EquityPoint>,
    timestamp: i64,
    equity: f64,
) {
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

/// Assemble per-symbol results and aggregate portfolio metrics.
pub(super) fn build_portfolio_result<S: Strategy>(
    config: &PortfolioConfig,
    states: HashMap<String, SymbolState<S>>,
    portfolio_equity_curve: Vec<EquityPoint>,
    allocation_history: Vec<AllocationSnapshot>,
    initial_capital: f64,
    final_equity: f64,
) -> PortfolioResult {
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
                config.base.risk_free_rate,
                config.base.bars_per_year,
            );

            let start_ts = state.candles.first().map(|c| c.timestamp).unwrap_or(0);
            let end_ts = state.candles.last().map(|c| c.timestamp).unwrap_or(0);

            let result = BacktestResult {
                symbol: sym.clone(),
                strategy_name: state.strategy_name.clone(),
                config: config.base.clone(),
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
                max_leverage_used: state.sym_max_leverage,
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
        config.base.risk_free_rate,
        config.base.bars_per_year,
    );
    portfolio_metrics.time_in_market_pct = compute_portfolio_time_in_market(&allocation_history);

    PortfolioResult {
        symbols: symbol_results,
        portfolio_equity_curve,
        portfolio_metrics,
        initial_capital,
        final_equity,
        allocation_history,
    }
}
