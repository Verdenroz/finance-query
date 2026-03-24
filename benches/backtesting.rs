use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use finance_query::Candle;
use finance_query::backtesting::{
    BacktestConfig, BacktestEngine, BayesianSearch, GridSearch, MonteCarloConfig, OptimizeMetric,
    ParamRange, SmaCrossover,
};
use std::hint::black_box;

/// Generate deterministic synthetic OHLCV candles.
///
/// Candle is #[non_exhaustive] outside the crate, so we construct via serde_json.
fn synthetic_candles(n: usize) -> Vec<Candle> {
    let mut price = 100.0_f64;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let phase = (i as f64) * 0.1;
        let swing = 5.0 * phase.sin();
        let open = price;
        let close = price + swing + (i as f64) * 0.01;
        let high = open.max(close) + 1.0 + (phase * 0.5).abs();
        let low = open.min(close) - 1.0 - (phase * 0.5).abs();
        let volume = (1_000_000.0 + 100_000.0 * (phase * 2.0).sin()) as i64;
        let candle: Candle = serde_json::from_value(serde_json::json!({
            "timestamp": 1_700_000_000_i64 + i as i64 * 86400,
            "open": open, "high": high, "low": low, "close": close,
            "volume": volume, "adjClose": close
        }))
        .unwrap();
        out.push(candle);
        price = close;
    }
    out
}

// ── Engine run ───────────────────────────────────────────────────────────────

fn bench_backtest_engine(c: &mut Criterion) {
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("backtest_engine");

    for n in [500usize, 1000, 2000] {
        let candles = synthetic_candles(n);
        group.bench_with_input(
            BenchmarkId::new("sma_crossover", n),
            &candles,
            |b, candles| {
                b.iter(|| {
                    let engine = BacktestEngine::new(black_box(config.clone()));
                    let strategy = SmaCrossover::new(10, 20);
                    black_box(engine.run(black_box("BENCH"), candles, strategy))
                })
            },
        );
    }

    group.finish();
}

// ── PerformanceMetrics computation ───────────────────────────────────────────

fn bench_performance_metrics(c: &mut Criterion) {
    let candles_2000 = synthetic_candles(2000);
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("performance_metrics");

    // Full 2000-bar backtest including metrics computation at end
    group.bench_function("full_backtest_2000", |b| {
        b.iter(|| {
            let engine = BacktestEngine::new(config.clone());
            black_box(engine.run("BENCH", &candles_2000, SmaCrossover::new(10, 50)))
        })
    });

    group.finish();
}

// ── Grid search optimiser ────────────────────────────────────────────────────

fn bench_grid_search(c: &mut Criterion) {
    let candles = synthetic_candles(500);
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("grid_search");

    // 3×3 = 9 evaluations
    group.bench_function("sma_3x3", |b| {
        b.iter(|| {
            let search = GridSearch::new()
                .param("fast", ParamRange::int_range(5, 15, 5))
                .param("slow", ParamRange::int_range(20, 40, 10))
                .optimize_for(OptimizeMetric::SharpeRatio);
            black_box(search.run("BENCH", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            }))
        })
    });

    // 5×5 = 25 evaluations
    group.bench_function("sma_5x5", |b| {
        b.iter(|| {
            let search = GridSearch::new()
                .param("fast", ParamRange::int_range(5, 25, 5))
                .param("slow", ParamRange::int_range(20, 60, 10))
                .optimize_for(OptimizeMetric::SharpeRatio);
            black_box(search.run("BENCH", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            }))
        })
    });

    group.finish();
}

// ── Monte Carlo ──────────────────────────────────────────────────────────────

fn bench_monte_carlo(c: &mut Criterion) {
    let candles = synthetic_candles(500);
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let engine = BacktestEngine::new(config);
    let result = engine
        .run("BENCH", &candles, SmaCrossover::new(10, 20))
        .unwrap();

    let mut group = c.benchmark_group("monte_carlo");

    for n_sims in [100usize, 500, 1000] {
        group.bench_with_input(BenchmarkId::new("iid_shuffle", n_sims), &n_sims, |b, &n| {
            let mc_config = MonteCarloConfig::new().num_simulations(n).seed(42);
            b.iter(|| black_box(mc_config.run(black_box(&result))))
        });
    }

    group.finish();
}

// ── Bayesian optimiser ───────────────────────────────────────────────────────

fn bench_bayesian_search(c: &mut Criterion) {
    let candles = synthetic_candles(500);
    let config = BacktestConfig::builder()
        .initial_capital(10_000.0)
        .commission_pct(0.001)
        .build()
        .unwrap();

    let mut group = c.benchmark_group("bayesian_search");

    // 2-param search, 20 evaluations (10 LHS + 10 sequential)
    group.bench_function("sma_2param_20eval", |b| {
        b.iter(|| {
            let search = BayesianSearch::new()
                .param("fast", ParamRange::int_bounds(5, 25))
                .param("slow", ParamRange::int_bounds(20, 60))
                .optimize_for(OptimizeMetric::SharpeRatio)
                .max_evaluations(20)
                .seed(42);
            std::hint::black_box(search.run("BENCH", &candles, &config, |params| {
                SmaCrossover::new(
                    params["fast"].as_int() as usize,
                    params["slow"].as_int() as usize,
                )
            }))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_backtest_engine,
    bench_performance_metrics,
    bench_grid_search,
    bench_monte_carlo,
    bench_bayesian_search,
);
criterion_main!(benches);
