use super::*;
use crate::backtesting::config::BacktestConfig;
use crate::backtesting::engine::fixtures::*;
use crate::backtesting::strategy::SmaCrossover;

#[test]
fn test_insufficient_data() {
    let candles = make_candles(&[100.0, 101.0, 102.0]); // Only 3 candles
    let config = BacktestConfig::default();
    let engine = BacktestEngine::new(config);
    let strategy = SmaCrossover::new(10, 20); // Needs at least 21 candles

    let result = engine.run("TEST", &candles, strategy);
    assert!(result.is_err());
}

/// The fundamental invariant: final cash (when no position is open) must equal
/// initial_capital plus the sum of all realized trade P&Ls.  This guards against
/// the double-counting of commissions that existed before the fix.
#[test]
fn test_commission_accounting_invariant() {
    // Steadily rising prices so SmaCrossover(3,6) will definitely enter and exit.
    let prices: Vec<f64> = (0..40)
        .map(|i| {
            if i < 30 {
                100.0 + i as f64
            } else {
                129.0 - (i - 30) as f64 * 5.0
            }
        })
        .collect();
    let candles = make_candles(&prices);

    // Use both flat AND percentage commission to expose any double-counting.
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission(5.0) // $5 flat fee per trade
        .commission_pct(0.001) // + 0.1% per trade
        .slippage_pct(0.0)
        .close_at_end(true)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config.clone());
    let result = engine
        .run("TEST", &candles, SmaCrossover::new(3, 6))
        .unwrap();

    // When all positions are closed, cash == initial_capital + sum(trade pnls)
    let sum_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
    let expected = config.initial_capital + sum_pnl;
    let actual = result.final_equity;
    assert!(
        (actual - expected).abs() < 1e-6,
        "Commission accounting: final_equity {actual:.6} != initial_capital + sum(pnl) {expected:.6}",
    );
}

#[test]
fn test_unsorted_dividends_returns_error() {
    use crate::models::chart::Dividend;

    let prices: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
    let candles = make_candles(&prices);

    // Intentionally unsorted
    let dividends = vec![
        Dividend {
            timestamp: 20,
            amount: 1.0,
            provider_id: None,
        },
        Dividend {
            timestamp: 10,
            amount: 1.0,
            provider_id: None,
        },
    ];

    let engine = BacktestEngine::new(BacktestConfig::default());
    let result = engine.run_with_dividends("TEST", &candles, SmaCrossover::new(3, 6), &dividends);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("sorted"),
        "error should mention sorting: {msg}"
    );
}

#[test]
fn test_short_dividend_is_liability() {
    use crate::models::chart::Dividend;

    let candles = make_candles(&[100.0, 100.0, 100.0]);
    let dividends = vec![Dividend {
        timestamp: candles[1].timestamp,
        amount: 1.0,
        provider_id: None,
    }];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .allow_short(true)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run_with_dividends("TEST", &candles, EnterShortHold, &dividends)
        .unwrap();

    assert_eq!(result.trades.len(), 1);
    assert!(result.trades[0].dividend_income < 0.0);
    assert!(result.final_equity < 10_000.0);
}

#[test]
fn test_open_position_final_equity_includes_accrued_dividends() {
    use crate::models::chart::Dividend;

    let candles = make_candles(&[100.0, 100.0, 100.0]);
    let dividends = vec![Dividend {
        timestamp: candles[1].timestamp,
        amount: 1.0,
        provider_id: None,
    }];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .close_at_end(false)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
        .unwrap();

    assert!(result.open_position.is_some());
    assert!((result.final_equity - 10_100.0).abs() < 1e-6);
    let last_equity = result.equity_curve.last().map(|p| p.equity).unwrap_or(0.0);
    assert!((last_equity - 10_100.0).abs() < 1e-6);
}

#[test]
fn unsorted_candles_are_rejected() {
    let mut candles = make_candles(&[100.0, 101.0, 102.0, 103.0]);
    candles.swap(1, 2);
    use crate::backtesting::refs::*;
    use crate::backtesting::strategy::StrategyBuilder;
    let strategy = StrategyBuilder::new("s")
        .entry(price().above(0.0))
        .exit(price().below(0.0))
        .build();
    let err = BacktestEngine::new(BacktestConfig::default())
        .run("TEST", &candles, strategy)
        .unwrap_err();
    assert!(
        format!("{err}").contains("candles"),
        "expected a candle-ordering error, got {err}"
    );
}

/// Enters once via a limit order, then exits on a trailing stop.
///
/// `track` selects which path supplies the peak: `true` uses the engine's
/// running extremes, `false` makes the condition fall back to scanning from
/// the entry bar. Both must agree.
#[derive(Clone)]
struct LimitEntryTrailing {
    track: bool,
    trail: crate::backtesting::condition::TrailingStop,
    limit: f64,
}

impl Strategy for LimitEntryTrailing {
    fn name(&self) -> &str {
        "limit-entry-trailing"
    }

    fn required_indicators(&self) -> Vec<(String, crate::indicators::Indicator)> {
        vec![]
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        use crate::backtesting::condition::Condition;
        if ctx.position.is_none() {
            if ctx.index == 0 {
                return Signal::buy_limit(ctx.timestamp(), ctx.close(), self.limit);
            }
            return Signal::hold();
        }
        if self.trail.evaluate(ctx) {
            return ctx.signal_exit();
        }
        Signal::hold()
    }

    fn tracks_position_extremes(&self) -> bool {
        self.track
    }
}

#[test]
fn a_limit_entry_counts_its_own_fill_bar_in_the_peak() {
    // Pending orders fill partway through a bar, after the engine has
    // already folded that bar's extremes for a still-empty position. The
    // fill bar carries the highest high here, so skipping it lowers the
    // peak and moves the trailing-stop exit.
    let candles = vec![
        // bar 0: signal bar, queues the limit order
        Candle {
            timestamp: 0,
            open: 100.0,
            high: 100.0,
            low: 100.0,
            close: 100.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
        // bar 1: dips to 95 (fills), then spikes to 130 — the peak
        Candle {
            timestamp: 1,
            open: 99.0,
            high: 130.0,
            low: 94.0,
            close: 120.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
        Candle {
            timestamp: 2,
            open: 119.0,
            high: 121.0,
            low: 115.0,
            close: 116.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
        Candle {
            timestamp: 3,
            open: 115.0,
            high: 116.0,
            low: 110.0,
            close: 111.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
        Candle {
            timestamp: 4,
            open: 110.0,
            high: 111.0,
            low: 104.0,
            close: 105.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
        Candle {
            timestamp: 5,
            open: 104.0,
            high: 105.0,
            low: 100.0,
            close: 101.0,
            volume: 1000,
            adj_close: None,
            provider_id: None,
        },
    ];

    let config = BacktestConfig {
        initial_capital: 10_000.0,
        ..Default::default()
    };
    let run = |track: bool| {
        BacktestEngine::new(config.clone())
            .run(
                "TEST",
                &candles,
                LimitEntryTrailing {
                    track,
                    trail: crate::backtesting::condition::TrailingStop::new(0.10),
                    limit: 95.0,
                },
            )
            .unwrap()
    };

    let engine_path = run(true);
    let scan_path = run(false);

    assert_eq!(
        engine_path.trades.len(),
        1,
        "the limit order should fill and the trailing stop should close it"
    );
    assert_eq!(
        engine_path.trades.len(),
        scan_path.trades.len(),
        "the two peak sources disagreed on whether a trade closed"
    );
    assert_eq!(
        engine_path.trades[0].exit_timestamp, scan_path.trades[0].exit_timestamp,
        "engine-tracked extremes and the entry-bar scan chose different exits"
    );
    assert_eq!(
        engine_path.trades[0].pnl, scan_path.trades[0].pnl,
        "same exit bar should mean same P&L"
    );
}

#[test]
fn sweeps_reject_unsorted_candles_at_their_own_entry_point() {
    // Validation moved off the per-candidate path, so each sweep entry point
    // has to check the series itself or an unsorted run would slip through.
    use crate::backtesting::optimizer::{BayesianSearch, GridSearch, ParamRange, ParamValue};
    use crate::backtesting::refs::*;
    use crate::backtesting::strategy::StrategyBuilder;
    use std::collections::HashMap;

    let mut candles = make_candles(&(0..80).map(|i| 100.0 + i as f64).collect::<Vec<f64>>());
    candles.swap(1, 2);
    let config = BacktestConfig::default();
    let factory = |_: &HashMap<String, ParamValue>| {
        StrategyBuilder::new("s")
            .entry(price().above(0.0))
            .exit(price().below(0.0))
            .build()
    };

    let grid_err = GridSearch::new()
        .param("p", ParamRange::int_range(1, 2, 1))
        .run("TEST", &candles, &config, factory)
        .unwrap_err();
    assert!(
        format!("{grid_err}").contains("candles"),
        "grid search should reject unsorted candles, got {grid_err}"
    );

    let bayes_err = BayesianSearch::new()
        .param("p", ParamRange::int_range(1, 2, 1))
        .max_evaluations(4)
        .run("TEST", &candles, &config, factory)
        .unwrap_err();
    assert!(
        format!("{bayes_err}").contains("candles"),
        "bayesian search should reject unsorted candles, got {bayes_err}"
    );
}

#[test]
fn expires_in_bars_one_fills_on_its_only_eligible_bar() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 97.0, 100.0),
        make_candle_ohlc(2, 100.0, 100.0, 100.0, 100.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run(
            "TEST",
            &candles,
            BuyLimitAt {
                bar: 0,
                limit_price: 98.0,
                expires_in_bars: Some(1),
            },
        )
        .unwrap();

    let pos = result.open_position.expect("order should have filled");
    assert!((pos.entry_price - 98.0).abs() < 1e-9);
}

#[test]
fn expires_in_bars_one_cancels_before_its_second_bar() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 99.0, 100.0),
        make_candle_ohlc(2, 100.0, 100.0, 97.0, 100.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run(
            "TEST",
            &candles,
            BuyLimitAt {
                bar: 0,
                limit_price: 98.0,
                expires_in_bars: Some(1),
            },
        )
        .unwrap();

    assert!(result.open_position.is_none());
    assert!(result.trades.is_empty());
}

#[test]
fn expires_in_bars_zero_never_fills() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 97.0, 100.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run(
            "TEST",
            &candles,
            BuyLimitAt {
                bar: 0,
                limit_price: 98.0,
                expires_in_bars: Some(0),
            },
        )
        .unwrap();

    assert!(result.open_position.is_none());
    assert!(result.trades.is_empty());
}

#[test]
fn a_limit_fill_on_the_final_bar_counts_toward_max_leverage_used() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(2, 100.0, 100.0, 97.0, 100.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run(
            "TEST",
            &candles,
            BuyLimitAt {
                bar: 0,
                limit_price: 98.0,
                expires_in_bars: None,
            },
        )
        .unwrap();

    assert!(result.open_position.is_some());
    assert!(
        result.max_leverage_used > 0.99,
        "the final bar's limit fill must register exposure, got {}",
        result.max_leverage_used
    );
}

#[test]
fn config_trailing_stop_counts_a_limit_entrys_fill_bar_in_the_peak() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 99.0, 130.0, 94.0, 120.0),
        make_candle_ohlc(2, 119.0, 121.0, 115.0, 116.0),
        make_candle_ohlc(3, 115.0, 116.0, 110.0, 111.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .trailing_stop_pct(0.10)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run(
            "TEST",
            &candles,
            BuyLimitAt {
                bar: 0,
                limit_price: 95.0,
                expires_in_bars: None,
            },
        )
        .unwrap();

    assert_eq!(result.trades.len(), 1);
    assert_eq!(result.trades[0].exit_timestamp, 2);
    assert!((result.trades[0].exit_price - 117.0).abs() < 1e-9);
}

#[test]
fn intrabar_stop_exit_resyncs_the_bars_equity_point_to_realized_cash() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(2, 96.0, 96.0, 90.0, 85.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .stop_loss_pct(0.05)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine.run("TEST", &candles, EnterLongBar0).unwrap();

    assert_eq!(result.trades.len(), 1);
    assert!((result.trades[0].exit_price - 95.0).abs() < 1e-9);

    let exit_point = result
        .equity_curve
        .iter()
        .find(|p| p.timestamp == 2)
        .expect("equity point for the exit bar");
    assert!(
        (exit_point.equity - 9_500.0).abs() < 1e-6,
        "expected the exit bar's equity to reflect the realized stop fill, got {}",
        exit_point.equity
    );
    assert!(
        (exit_point.drawdown_pct - 0.05).abs() < 1e-6,
        "expected drawdown capped at the stop's 5%, got {}",
        exit_point.drawdown_pct
    );
}

#[test]
fn intrabar_take_profit_exit_does_not_leave_a_phantom_peak() {
    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(2, 105.0, 121.0, 104.0, 120.0),
        make_candle_ohlc(3, 110.0, 110.0, 110.0, 110.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .take_profit_pct(0.10)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine.run("TEST", &candles, EnterLongBar0).unwrap();

    assert_eq!(result.trades.len(), 1);
    assert!((result.trades[0].exit_price - 110.0).abs() < 1e-9);

    let exit_point = result
        .equity_curve
        .iter()
        .find(|p| p.timestamp == 2)
        .expect("equity point for the exit bar");
    assert!((exit_point.equity - 11_000.0).abs() < 1e-6);
    assert!(
        exit_point.drawdown_pct.abs() < 1e-9,
        "the pre-exit close-marked snapshot must not become the peak, got drawdown {}",
        exit_point.drawdown_pct
    );
    assert!(
        result.metrics.max_drawdown_pct.abs() < 1e-9,
        "expected no drawdown against the realized peak, got {}",
        result.metrics.max_drawdown_pct
    );
}

#[test]
fn ex_date_bar_curve_point_includes_the_same_bar_dividend() {
    use crate::models::chart::Dividend;

    let candles = make_candles(&[100.0; 6]);
    let dividends = vec![Dividend {
        timestamp: candles[2].timestamp,
        amount: 5.0,
        provider_id: None,
    }];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
        .unwrap();

    let ex_date_point = result
        .equity_curve
        .iter()
        .find(|p| p.timestamp == candles[2].timestamp)
        .expect("equity point on the ex-date bar");
    assert!(
        (ex_date_point.equity - 10_500.0).abs() < 1e-6,
        "expected the ex-date bar's curve point to include the dividend, got {}",
        ex_date_point.equity
    );
}

#[test]
fn open_position_metrics_include_dividends_and_span() {
    use crate::models::chart::Dividend;

    let candles = make_candles(&[100.0; 6]);
    let dividends = vec![Dividend {
        timestamp: candles[2].timestamp,
        amount: 5.0,
        provider_id: None,
    }];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
        .unwrap();

    assert!(result.trades.is_empty());
    assert!((result.metrics.total_dividend_income - 500.0).abs() < 1e-6);
    // Entered at bar 1's open, held through bar 5 of a 0..5 curve.
    assert!(
        (result.metrics.time_in_market_pct - 0.8).abs() < 1e-9,
        "expected 0.8, got {}",
        result.metrics.time_in_market_pct
    );
}

#[test]
fn a_buy_limit_follow_signal_queues_instead_of_market_filling() {
    #[derive(Clone)]
    struct ExitThenBuyLimit;
    impl Strategy for ExitThenBuyLimit {
        fn name(&self) -> &str {
            "ExitThenBuyLimit"
        }
        fn required_indicators(&self) -> Vec<(String, crate::indicators::Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match ctx.index {
                0 => Signal::long(ctx.timestamp(), ctx.close()),
                1 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                1 => Signal::buy_limit(ctx.timestamp(), ctx.close(), 90.0),
                _ => Signal::hold(),
            }
        }
    }

    let candles = vec![
        make_candle_ohlc(0, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0),
        make_candle_ohlc(2, 100.0, 100.0, 95.0, 97.0),
        make_candle_ohlc(3, 100.0, 100.0, 88.0, 90.0),
    ];

    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .close_at_end(false)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine.run("TEST", &candles, ExitThenBuyLimit).unwrap();

    let pos = result
        .open_position
        .expect("the follow limit order should have filled once price reached it");
    assert!(
        (pos.entry_price - 90.0).abs() < 1e-9,
        "follow signal should honor its limit price, not market-fill at the next open, got {}",
        pos.entry_price
    );
}
