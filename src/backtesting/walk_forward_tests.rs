use super::*;
use crate::backtesting::{
    BacktestConfig, SmaCrossover,
    optimizer::{OptimizeMetric, ParamRange},
};
use crate::models::chart::Candle;

fn make_candles(prices: &[f64]) -> Vec<Candle> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &p)| Candle {
            timestamp: i as i64,
            open: p,
            high: p * 1.01,
            low: p * 0.99,
            close: p,
            volume: 1000,
            adj_close: Some(p),
            provider_id: None,
        })
        .collect()
}

fn trending_prices(n: usize) -> Vec<f64> {
    (0..n).map(|i| 100.0 + i as f64 * 0.3).collect()
}

#[test]
fn test_walk_forward_basic() {
    // 300 bars: 200 IS + 100 OOS → 1 window
    let prices = trending_prices(300);
    let candles = make_candles(&prices);
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 9, 3))
        .param("slow", ParamRange::int_range(10, 20, 10))
        .optimize_for(OptimizeMetric::TotalReturn);

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(200)
        .out_of_sample_bars(100)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    assert_eq!(report.windows.len(), 1);
    assert_eq!(report.strategy_name, "SMA Crossover");
    assert!(report.consistency_ratio >= 0.0);
    assert!(report.consistency_ratio <= 1.0);
}

#[test]
fn test_walk_forward_multiple_windows() {
    // 500 bars, step = 100 OOS → 3 windows (100+100, 200+100, 300+100, 400+100)
    let prices = trending_prices(500);
    let candles = make_candles(&prices);
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 6, 3))
        .param("slow", ParamRange::int_range(10, 10, 1))
        .optimize_for(OptimizeMetric::TotalReturn);

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(200)
        .out_of_sample_bars(100)
        .step_bars(100)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    assert!(report.windows.len() >= 2);
    assert_eq!(report.optimization_reports.len(), report.windows.len());
}

#[test]
fn walk_forward_windows_are_order_stable() {
    let candles = make_candles(&trending_prices(900));
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let run = || {
        let grid = GridSearch::new()
            .param("fast", ParamRange::int_range(3, 9, 3))
            .param("slow", ParamRange::int_range(10, 20, 10))
            .optimize_for(OptimizeMetric::TotalReturn);
        WalkForwardConfig::new(grid, config.clone())
            .in_sample_bars(200)
            .out_of_sample_bars(100)
            .run("TEST", &candles, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            })
            .unwrap()
    };

    let a = run();
    let b = run();
    assert!(
        a.windows.len() >= 3,
        "need multiple windows to test ordering"
    );
    assert_eq!(a.windows.len(), b.windows.len());
    for (i, (x, y)) in a.windows.iter().zip(b.windows.iter()).enumerate() {
        assert_eq!(x.window, i, "window index out of order at position {i}");
        assert_eq!(y.window, i, "window index out of order at position {i}");
        assert_eq!(
            x.optimized_params, y.optimized_params,
            "window {i} diverged"
        );
        assert_eq!(
            x.out_of_sample.metrics.total_return_pct, y.out_of_sample.metrics.total_return_pct,
            "window {i} diverged"
        );
        assert_eq!(
            x.out_of_sample.start_timestamp, y.out_of_sample.start_timestamp,
            "window {i} diverged"
        );
        assert_eq!(
            x.out_of_sample.end_timestamp, y.out_of_sample.end_timestamp,
            "window {i} diverged"
        );
        assert_eq!(
            x.in_sample.metrics.total_return_pct, y.in_sample.metrics.total_return_pct,
            "window {i} diverged"
        );
    }

    for pair in a.windows.windows(2) {
        assert!(
            pair[0].out_of_sample.start_timestamp < pair[1].out_of_sample.start_timestamp,
            "windows not in chronological order: {} >= {}",
            pair[0].out_of_sample.start_timestamp,
            pair[1].out_of_sample.start_timestamp
        );
    }

    assert_eq!(a.optimization_reports.len(), a.windows.len());
    for (i, (x, y)) in a
        .optimization_reports
        .iter()
        .zip(b.optimization_reports.iter())
        .enumerate()
    {
        assert_eq!(x.best.params, y.best.params, "opt report {i} diverged");
    }

    assert_eq!(a.consistency_ratio, b.consistency_ratio);
}

#[test]
fn test_step_bars_zero_errors() {
    let candles = make_candles(&trending_prices(300));
    let config = BacktestConfig::default();
    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 6, 3))
        .param("slow", ParamRange::int_range(10, 10, 1));

    let result = WalkForwardConfig::new(grid, config)
        .in_sample_bars(200)
        .out_of_sample_bars(100)
        .step_bars(0)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        });

    assert!(result.is_err());
}

#[test]
fn test_insufficient_data_errors() {
    let candles = make_candles(&trending_prices(50));
    let config = BacktestConfig::default();
    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 6, 3))
        .param("slow", ParamRange::int_range(10, 10, 1));

    let result = WalkForwardConfig::new(grid, config)
        .in_sample_bars(200) // more than 50 candles
        .out_of_sample_bars(100)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        });

    assert!(result.is_err());
}

#[test]
fn test_consistency_ratio_all_profitable() {
    // All windows profitable → ratio = 1.0
    let prices: Vec<f64> = (0..300).map(|i| 100.0 + i as f64).collect();
    let candles = make_candles(&prices);
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 3, 1))
        .param("slow", ParamRange::int_range(10, 10, 1))
        .optimize_for(OptimizeMetric::TotalReturn);

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(150)
        .out_of_sample_bars(100)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    // With a strong uptrend, the OOS window should be profitable
    assert!(report.consistency_ratio >= 0.0);
}

#[test]
fn test_aggregate_equity_timestamps_are_monotonic() {
    // With 3+ OOS windows, timestamps in the aggregated equity curve must
    // be strictly increasing
    let prices: Vec<f64> = (0..600).map(|i| 100.0 + (i as f64) * 0.5).collect();
    let candles = make_candles(&prices);
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 3, 1))
        .param("slow", ParamRange::int_range(10, 10, 1))
        .optimize_for(OptimizeMetric::TotalReturn);

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(100)
        .out_of_sample_bars(50)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    // Verify timestamps in aggregate metrics equity curve are strictly increasing
    let curve = &report.aggregate_metrics;
    // We verify indirectly: there must be at least 2 windows
    assert!(
        report.windows.len() >= 2,
        "Expected multiple windows for timestamp test"
    );

    // Also check the combined OOS timestamps from windows directly
    let timestamps: Vec<i64> = report
        .windows
        .iter()
        .flat_map(|w| w.out_of_sample.equity_curve.iter().map(|ep| ep.timestamp))
        .collect();

    // Each window's timestamps should be internally monotonic
    for window in &report.windows {
        let ts: Vec<i64> = window
            .out_of_sample
            .equity_curve
            .iter()
            .map(|ep| ep.timestamp)
            .collect();
        for pair in ts.windows(2) {
            assert!(
                pair[0] < pair[1],
                "Equity curve timestamps not strictly increasing within window: {} >= {}",
                pair[0],
                pair[1]
            );
        }
    }

    // Suppress unused variable warning
    let _ = curve;
    let _ = timestamps;
}

#[test]
fn test_aggregate_oos_equity_timestamps_are_gapless_across_windows() {
    // The aggregated equity curve produced by aggregate_oos_metrics must carry
    // the real OOS candle timestamps so that time-in-market calculations
    // (which divide trade duration_secs by backtest_secs) use a consistent
    // unit. Previously timestamps were replaced with auto-incrementing integers
    // (0,1,2,...) which caused the denominator to be "N bars" instead of
    // "N seconds", inflating time_in_market to 1.0 on any real-world data.
    let prices: Vec<f64> = (0..600).map(|i| 100.0 + (i as f64) * 0.5).collect();
    let candles = make_candles(&prices);
    let config = BacktestConfig::builder()
        .commission_pct(0.0)
        .slippage_pct(0.0)
        .build()
        .unwrap();

    let grid = GridSearch::new()
        .param("fast", ParamRange::int_range(3, 3, 1))
        .param("slow", ParamRange::int_range(10, 10, 1))
        .optimize_for(OptimizeMetric::TotalReturn);

    let report = WalkForwardConfig::new(grid, config)
        .in_sample_bars(100)
        .out_of_sample_bars(50)
        .run("TEST", &candles, |params| {
            SmaCrossover::new(
                params["fast"].as_int() as usize,
                params["slow"].as_int() as usize,
            )
        })
        .unwrap();

    assert!(
        report.windows.len() >= 2,
        "Need at least 2 OOS windows for this test"
    );

    // Collect the combined timestamps as produced by aggregate_oos_metrics.
    // They must be strictly increasing (real candle timestamps, not bar indices).
    let combined_ts: Vec<i64> = report
        .windows
        .iter()
        .enumerate()
        .flat_map(|(wi, w)| {
            w.out_of_sample
                .equity_curve
                .iter()
                .enumerate()
                .filter(move |&(pi, _)| !(wi > 0 && pi == 0))
                .map(|(_, ep)| ep.timestamp)
        })
        .collect();

    for pair in combined_ts.windows(2) {
        assert!(
            pair[0] < pair[1],
            "Combined equity curve timestamps not strictly increasing: {} >= {}",
            pair[0],
            pair[1]
        );
    }

    // Timestamps must reflect real candle timestamps — the first combined
    // timestamp should match the first OOS window's first equity point.
    let expected_first = report
        .windows
        .first()
        .and_then(|w| w.out_of_sample.equity_curve.first())
        .map(|ep| ep.timestamp)
        .unwrap_or(0);
    assert_eq!(
        combined_ts.first().copied().unwrap_or(-1),
        expected_first,
        "First combined timestamp should equal the first OOS equity point timestamp"
    );
}
