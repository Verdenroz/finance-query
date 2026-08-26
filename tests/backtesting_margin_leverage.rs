//! Leverage must be paid for. These run through the public API only, so the
//! guarantees they pin are the ones a caller actually gets.

#![cfg(feature = "backtesting")]

use finance_query::Candle;
use finance_query::backtesting::{
    BacktestConfig, BacktestEngine, BacktestResult, Signal, Strategy, StrategyContext,
};
use finance_query::indicators::Indicator;

struct EnterLongHold;

impl Strategy for EnterLongHold {
    fn name(&self) -> &str {
        "Enter Long Hold"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        if ctx.index == 0 && !ctx.has_position() {
            Signal::long(ctx.timestamp(), ctx.close())
        } else {
            Signal::hold()
        }
    }
}

fn candles(prices: &[f64]) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            serde_json::from_value(serde_json::json!({
                "timestamp": i as i64,
                "open": p,
                "high": p,
                "low": p,
                "close": p,
                "volume": 1000,
                "adj_close": p,
            }))
            .unwrap()
        })
        .collect()
}

/// Total P&L before the costs of trading it.
fn gross_pnl(result: &BacktestResult) -> f64 {
    result
        .trades
        .iter()
        .map(|t| t.pnl + t.commission + t.transaction_tax + t.financing_cost)
        .sum()
}

#[test]
fn leverage_lifts_gross_return_and_sinks_net_return() {
    let prices: Vec<f64> = (0..100).map(|i| 100.0 + i as f64 * 0.02).collect();
    let series = candles(&prices);

    let run = |max_leverage: f64| {
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .position_size_pct(1.0)
            .commission_pct(0.001)
            .slippage_pct(0.0)
            .margin_interest_rate(0.10)
            .bars_per_year(252.0)
            .max_leverage(max_leverage)
            .close_at_end(true)
            .build()
            .unwrap();
        BacktestEngine::new(config)
            .run("TEST", &series, EnterLongHold)
            .unwrap()
    };

    let unlevered = run(1.0);
    let levered = run(2.0);

    assert!(
        gross_pnl(&levered) > gross_pnl(&unlevered),
        "leverage should raise gross P&L: {} vs {}",
        gross_pnl(&levered),
        gross_pnl(&unlevered),
    );
    assert!(
        levered.metrics.total_return_pct < unlevered.metrics.total_return_pct,
        "financing should sink net return: {} vs {}",
        levered.metrics.total_return_pct,
        unlevered.metrics.total_return_pct,
    );
    assert!(levered.metrics.total_financing_cost > 0.0);
    assert_eq!(unlevered.metrics.total_financing_cost, 0.0);
}

#[test]
fn a_margin_call_liquidates_the_position() {
    // 3x on 10_000 buys 300 shares and leaves cash at -20_000. Maintenance bites
    // once -20_000 + 300p < 0.25 * 300p, i.e. below 88.89.
    let series = candles(&[100.0, 100.0, 100.0, 85.0, 85.0]);

    let run = |max_leverage: f64| {
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .position_size_pct(1.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .max_leverage(max_leverage)
            .maintenance_margin_pct(0.25)
            .close_at_end(false)
            .build()
            .unwrap();
        BacktestEngine::new(config)
            .run("TEST", &series, EnterLongHold)
            .unwrap()
    };

    let called = run(3.0);
    assert_eq!(called.trades.len(), 1);
    assert_eq!(called.trades[0].exit_timestamp, 3);
    assert!(
        called.trades[0]
            .exit_signal
            .reason
            .as_deref()
            .is_some_and(|r| r.contains("Margin call"))
    );
    assert!(called.open_position.is_none());
    assert!(called.signals.iter().any(|s| {
        s.executed
            && s.reason
                .as_deref()
                .is_some_and(|r| r.contains("Margin call"))
    }));

    let unlevered = run(1.0);
    assert!(unlevered.trades.is_empty());
    assert!(unlevered.open_position.is_some());
}
