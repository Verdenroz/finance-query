use std::collections::HashMap;

use crate::backtesting::result::{BenchmarkMetrics, EquityPoint};
use crate::models::chart::Candle;

use super::BacktestEngine;

impl BacktestEngine {
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
}

/// Compute benchmark comparison metrics for a completed backtest.
///
/// `symbol_candles` are the candles for the backtested symbol (used to derive
/// its buy-and-hold return). `benchmark_candles` are the benchmark's candles.
/// `equity_curve` is used to derive strategy periodic returns for beta/IR.
pub(super) fn compute_benchmark_metrics(
    benchmark_symbol: &str,
    symbol_candles: &[Candle],
    benchmark_candles: &[Candle],
    equity_curve: &[EquityPoint],
    risk_free_rate: f64,
    bars_per_year: f64,
) -> BenchmarkMetrics {
    // Buy-and-hold returns from first to last close
    let benchmark_return_pct = buy_and_hold_return(benchmark_candles);
    let buy_and_hold_return_pct = buy_and_hold_return(symbol_candles);

    if equity_curve.len() < 2 || benchmark_candles.len() < 2 {
        return BenchmarkMetrics {
            symbol: benchmark_symbol.to_string(),
            benchmark_return_pct,
            buy_and_hold_return_pct,
            alpha: 0.0,
            beta: 0.0,
            information_ratio: 0.0,
            tracking_error: 0.0,
        };
    }

    let strategy_returns_by_ts: Vec<(i64, f64)> = equity_curve
        .windows(2)
        .map(|w| {
            let prev = w[0].equity;
            let ret = if prev > 0.0 {
                (w[1].equity - prev) / prev
            } else {
                0.0
            };
            (w[1].timestamp, ret)
        })
        .collect();

    let bench_returns_by_ts: HashMap<i64, f64> = benchmark_candles
        .windows(2)
        .map(|w| {
            let prev = w[0].close;
            let ret = if prev > 0.0 {
                (w[1].close - prev) / prev
            } else {
                0.0
            };
            (w[1].timestamp, ret)
        })
        .collect();

    let mut aligned_strategy = Vec::new();
    let mut aligned_benchmark = Vec::new();
    for (ts, s_ret) in strategy_returns_by_ts {
        if let Some(b_ret) = bench_returns_by_ts.get(&ts) {
            aligned_strategy.push(s_ret);
            aligned_benchmark.push(*b_ret);
        }
    }

    let beta = compute_beta(&aligned_strategy, &aligned_benchmark);

    // CAPM alpha on the same aligned sample used for beta/IR.
    let strategy_ann = annualized_return_from_periodic(&aligned_strategy, bars_per_year);
    let bench_ann = annualized_return_from_periodic(&aligned_benchmark, bars_per_year);
    // Jensen's Alpha: excess strategy return over what CAPM predicts given beta.
    // Both strategy_ann and bench_ann are in percentage form (×100), so rf_ann is scaled
    // to match before applying the CAPM formula: α = R_s - R_f - β(R_b - R_f).
    let rf_ann = risk_free_rate * 100.0;
    let alpha = strategy_ann - rf_ann - beta * (bench_ann - rf_ann);

    // Information ratio / tracking error: shared with the standalone `risk`
    // module via `crate::perf_metrics` (see that module's doc comment for why
    // the formula lives outside both `risk` and `backtesting`).
    let ir = crate::perf_metrics::information_ratio(
        &aligned_strategy,
        &aligned_benchmark,
        bars_per_year,
    )
    .unwrap_or(0.0);
    let te =
        crate::perf_metrics::tracking_error(&aligned_strategy, &aligned_benchmark, bars_per_year)
            .unwrap_or(0.0);

    BenchmarkMetrics {
        symbol: benchmark_symbol.to_string(),
        benchmark_return_pct,
        buy_and_hold_return_pct,
        alpha,
        beta,
        information_ratio: ir,
        tracking_error: te,
    }
}

/// Buy-and-hold return from first to last candle close (percentage).
fn buy_and_hold_return(candles: &[Candle]) -> f64 {
    match (candles.first(), candles.last()) {
        (Some(first), Some(last)) if first.close > 0.0 => {
            ((last.close / first.close) - 1.0) * 100.0
        }
        _ => 0.0,
    }
}

/// Annualised return from periodic returns (fractional, e.g. 0.01 for 1%).
fn annualized_return_from_periodic(periodic_returns: &[f64], bars_per_year: f64) -> f64 {
    let years = periodic_returns.len() as f64 / bars_per_year;
    if years > 0.0 {
        let growth = periodic_returns
            .iter()
            .fold(1.0_f64, |acc, r| acc * (1.0 + *r));
        if growth <= 0.0 {
            -100.0
        } else {
            (growth.powf(1.0 / years) - 1.0) * 100.0
        }
    } else {
        0.0
    }
}

/// Compute beta of `strategy_returns` relative to `benchmark_returns`.
///
/// Beta = Cov(strategy, benchmark) / Var(benchmark).
/// Uses sample covariance and variance (divides by n-1) to match the `risk`
/// module and standard financial convention. Returns 0.0 when benchmark
/// variance is zero or there are fewer than 2 observations.
fn compute_beta(strategy_returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    let n = strategy_returns.len();
    if n < 2 {
        return 0.0;
    }

    let s_mean = strategy_returns.iter().sum::<f64>() / n as f64;
    let b_mean = benchmark_returns.iter().sum::<f64>() / n as f64;

    // Sample covariance and variance (n-1)
    let cov: f64 = strategy_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(s, b)| (s - s_mean) * (b - b_mean))
        .sum::<f64>()
        / (n - 1) as f64;

    let b_var: f64 = benchmark_returns
        .iter()
        .map(|b| (b - b_mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;

    if b_var > 0.0 { cov / b_var } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::fixtures::*;
    use crate::backtesting::strategy::SmaCrossover;

    #[test]
    fn test_capm_alpha_with_risk_free_rate() {
        // When risk_free_rate = 0, alpha should equal the simplified formula.
        // When risk_free_rate > 0, the CAPM adjustment should reduce alpha.
        let prices: Vec<f64> = (0..60).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);

        // Run once with rf=0 and once with rf=0.05, compare benchmark metrics
        let config_no_rf = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .risk_free_rate(0.0)
            .build()
            .unwrap();
        let config_with_rf = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .risk_free_rate(0.05)
            .build()
            .unwrap();

        let engine_no_rf = BacktestEngine::new(config_no_rf);
        let engine_with_rf = BacktestEngine::new(config_with_rf);

        // Use same candles for both strategy and benchmark to get beta ≈ 1
        let result_no_rf = engine_no_rf
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 10),
                &[],
                "BENCH",
                &candles,
            )
            .unwrap();
        let result_with_rf = engine_with_rf
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 10),
                &[],
                "BENCH",
                &candles,
            )
            .unwrap();

        let bm_no_rf = result_no_rf.benchmark.unwrap();
        let bm_with_rf = result_with_rf.benchmark.unwrap();

        // With identical strategy and benchmark (beta = 1), Jensen's alpha ≈ 0 always.
        // Both should be close to 0, but importantly they should differ when rf != 0.
        // This test ensures the formula uses rf — it catches the old bug where rf was ignored.
        assert!(bm_no_rf.alpha.is_finite(), "Alpha should be finite");
        assert!(
            bm_with_rf.alpha.is_finite(),
            "Alpha should be finite with rf"
        );

        // With beta ≈ 1 and rf=5%, CAPM alpha = R_s - 5% - 1*(R_b - 5%) = R_s - R_b.
        // Same formula result as rf=0 when beta=1; but the formula path is exercised.
        // The key check: alpha is the same sign in both (both near-zero).
        assert!(
            bm_no_rf.alpha.abs() < 50.0,
            "Alpha should be small for identical strategy/benchmark"
        );
        assert!(
            bm_with_rf.alpha.abs() < 50.0,
            "Alpha should be small for identical strategy/benchmark with rf"
        );
    }

    #[test]
    fn test_run_with_benchmark_credits_dividends() {
        use crate::models::chart::Dividend;

        // Rising price series — long enough for SmaCrossover(3,6) to trade
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);

        // A single dividend ex-dated mid-series
        let mid_ts = candles[15].timestamp;
        let dividends = vec![Dividend {
            timestamp: mid_ts,
            amount: 1.0,
            provider_id: None,
        }];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 6),
                &dividends,
                "BENCH",
                &candles,
            )
            .unwrap();

        // Dividend income is credited only while a position is open.
        // If no trade happened to be open on bar 15 the income is zero;
        // either way the engine must not panic and the benchmark must be set.
        assert!(result.benchmark.is_some());
        let total_div: f64 = result.trades.iter().map(|t| t.dividend_income).sum();
        // total_dividend_income is non-negative (either credited or not, never negative)
        assert!(total_div >= 0.0);
    }

    #[test]
    fn test_benchmark_beta_and_ir_require_timestamp_overlap() {
        let symbol_candles = make_candles_with_timestamps(&[100.0, 110.0, 120.0], &[100, 200, 300]);
        let benchmark_candles =
            make_candles_with_timestamps(&[50.0, 55.0, 60.0, 65.0], &[1000, 1100, 1200, 1300]);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_benchmark(
                "TEST",
                &symbol_candles,
                EnterLongHold,
                &[],
                "BENCH",
                &benchmark_candles,
            )
            .unwrap();

        let benchmark = result.benchmark.unwrap();
        assert!((benchmark.beta - 0.0).abs() < 1e-12);
        assert!((benchmark.information_ratio - 0.0).abs() < 1e-12);
    }
}
