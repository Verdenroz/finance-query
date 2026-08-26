use chrono::NaiveDate;

use super::{BacktestResult, EquityPoint, PerformanceMetrics};
use crate::backtesting::config::BacktestConfig;
use crate::backtesting::position::{Position, PositionSide, Trade};
use crate::backtesting::signal::Signal;

pub(super) fn make_trade(pnl: f64, return_pct: f64, is_long: bool) -> Trade {
    Trade {
        side: if is_long {
            PositionSide::Long
        } else {
            PositionSide::Short
        },
        entry_timestamp: 0,
        exit_timestamp: 100,
        entry_price: 100.0,
        exit_price: 100.0 + pnl / 10.0,
        quantity: 10.0,
        entry_quantity: 10.0,
        commission: 0.0,
        transaction_tax: 0.0,
        pnl,
        return_pct,
        dividend_income: 0.0,
        unreinvested_dividends: 0.0,
        financing_cost: 0.0,
        tags: Vec::new(),
        is_partial: false,
        scale_sequence: 0,
        entry_signal: Signal::long(0, 100.0),
        exit_signal: Signal::exit(100, 110.0),
    }
}

pub(super) fn make_trade_timed(pnl: f64, return_pct: f64, entry_ts: i64, exit_ts: i64) -> Trade {
    Trade {
        side: PositionSide::Long,
        entry_timestamp: entry_ts,
        exit_timestamp: exit_ts,
        entry_price: 100.0,
        exit_price: 100.0 + pnl / 10.0,
        quantity: 10.0,
        entry_quantity: 10.0,
        commission: 0.0,
        transaction_tax: 0.0,
        pnl,
        return_pct,
        dividend_income: 0.0,
        unreinvested_dividends: 0.0,
        financing_cost: 0.0,
        tags: Vec::new(),
        is_partial: false,
        scale_sequence: 0,
        entry_signal: Signal::long(entry_ts, 100.0),
        exit_signal: Signal::exit(exit_ts, 100.0 + pnl / 10.0),
    }
}

/// Minimal `BacktestResult` fixture using the default `BacktestConfig`
/// (risk_free_rate=0.0, bars_per_year=252.0).
pub(super) fn make_result(trades: Vec<Trade>, equity_curve: Vec<EquityPoint>) -> BacktestResult {
    let metrics = PerformanceMetrics::calculate(
        &trades,
        &equity_curve,
        10000.0,
        trades.len(),
        trades.len(),
        0.0,
        252.0,
    );
    BacktestResult {
        symbol: "TEST".to_string(),
        strategy_name: "TestStrategy".to_string(),
        config: BacktestConfig::default(),
        start_timestamp: equity_curve.first().map(|e| e.timestamp).unwrap_or(0),
        end_timestamp: equity_curve.last().map(|e| e.timestamp).unwrap_or(0),
        initial_capital: 10000.0,
        final_equity: equity_curve.last().map(|e| e.equity).unwrap_or(10000.0),
        metrics,
        trades,
        equity_curve,
        signals: vec![],
        open_position: None::<Position>,
        benchmark: None,
        diagnostics: vec![],
    }
}

pub(super) fn ts(date: &str) -> i64 {
    let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
    d.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp()
}

pub(super) fn equity_point(timestamp: i64, equity: f64, drawdown_pct: f64) -> EquityPoint {
    EquityPoint {
        timestamp,
        equity,
        drawdown_pct,
    }
}
