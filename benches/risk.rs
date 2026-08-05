//! Criterion benchmarks for the `risk` analytics module.
//!
//! Pure-compute hot paths (VaR, Sharpe/Sortino/Calmar, beta, drawdown) over
//! deterministic synthetic return series — no network, no randomness beyond a
//! seeded walk. Run with:
//!
//! ```text
//! cargo bench --bench risk --features risk
//! ```

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use finance_query::risk::{
    beta, calmar_ratio, historical_cvar, historical_var, information_ratio, kelly_criterion,
    max_drawdown, omega_ratio, parametric_cvar, parametric_var, sharpe_ratio, sortino_ratio,
    tracking_error, ulcer_index, win_loss_stats,
};

/// Deterministic pseudo-random return series (seeded xorshift, no `rand` dep).
fn synthetic_returns(n: usize) -> Vec<f64> {
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    (0..n)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            // Centred roughly on zero, ~±3% daily moves.
            ((state >> 40) as f64 / (1u64 << 24) as f64 - 0.5) * 0.06
        })
        .collect()
}

fn bench_value_at_risk(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_at_risk");
    for n in [252usize, 1260, 2520] {
        let returns = synthetic_returns(n);
        group.bench_with_input(BenchmarkId::new("historical_95", n), &returns, |b, r| {
            b.iter(|| historical_var(black_box(r), 0.95))
        });
        group.bench_with_input(BenchmarkId::new("historical_99", n), &returns, |b, r| {
            b.iter(|| historical_var(black_box(r), 0.99))
        });
        group.bench_with_input(BenchmarkId::new("parametric_95", n), &returns, |b, r| {
            b.iter(|| parametric_var(black_box(r), 0.95))
        });
    }
    group.finish();
}

fn bench_ratios(c: &mut Criterion) {
    let mut group = c.benchmark_group("ratios");
    for n in [252usize, 1260, 2520] {
        let returns = synthetic_returns(n);
        group.bench_with_input(BenchmarkId::new("sharpe", n), &returns, |b, r| {
            b.iter(|| sharpe_ratio(black_box(r), 0.0, 252.0))
        });
        group.bench_with_input(BenchmarkId::new("sortino", n), &returns, |b, r| {
            b.iter(|| sortino_ratio(black_box(r), 0.0, 252.0))
        });
    }
    group.bench_function("calmar", |b| {
        b.iter(|| calmar_ratio(black_box(0.18), black_box(5.0), black_box(0.30)))
    });
    group.finish();
}

fn bench_drawdown(c: &mut Criterion) {
    let mut group = c.benchmark_group("drawdown");
    for n in [252usize, 1260, 2520] {
        let returns = synthetic_returns(n);
        group.bench_with_input(BenchmarkId::new("max_drawdown", n), &returns, |b, r| {
            b.iter(|| max_drawdown(black_box(r)))
        });
    }
    group.finish();
}

fn bench_beta(c: &mut Criterion) {
    let mut group = c.benchmark_group("beta");
    for n in [252usize, 1260, 2520] {
        let asset = synthetic_returns(n);
        let benchmark = synthetic_returns(n + 1); // different seed offset via length
        group.bench_with_input(
            BenchmarkId::new("beta", n),
            &(asset, benchmark),
            |b, (a, m)| b.iter(|| beta(black_box(a), black_box(m))),
        );
    }
    group.finish();
}

fn bench_cvar(c: &mut Criterion) {
    let mut group = c.benchmark_group("conditional_value_at_risk");
    for n in [252usize, 1260, 2520] {
        let returns = synthetic_returns(n);
        group.bench_with_input(BenchmarkId::new("historical_95", n), &returns, |b, r| {
            b.iter(|| historical_cvar(black_box(r), 0.95))
        });
        group.bench_with_input(BenchmarkId::new("parametric_95", n), &returns, |b, r| {
            b.iter(|| parametric_cvar(black_box(r), 0.95))
        });
    }
    group.finish();
}

fn bench_promoted_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("promoted_metrics");
    for n in [252usize, 1260, 2520] {
        let returns = synthetic_returns(n);
        group.bench_with_input(BenchmarkId::new("omega_ratio", n), &returns, |b, r| {
            b.iter(|| omega_ratio(black_box(r)))
        });
        group.bench_with_input(BenchmarkId::new("ulcer_index", n), &returns, |b, r| {
            b.iter(|| ulcer_index(black_box(r)))
        });
        group.bench_with_input(BenchmarkId::new("win_loss_stats", n), &returns, |b, r| {
            b.iter(|| win_loss_stats(black_box(r)))
        });
    }
    group.bench_function("kelly_criterion", |b| {
        b.iter(|| kelly_criterion(black_box(0.55), black_box(1.8), black_box(-1.2)))
    });

    for n in [252usize, 1260, 2520] {
        let asset = synthetic_returns(n);
        let benchmark = synthetic_returns(n + 1);
        group.bench_with_input(
            BenchmarkId::new("information_ratio", n),
            &(asset.clone(), benchmark.clone()),
            |b, (a, m)| b.iter(|| information_ratio(black_box(a), black_box(m), 252.0)),
        );
        group.bench_with_input(
            BenchmarkId::new("tracking_error", n),
            &(asset, benchmark),
            |b, (a, m)| b.iter(|| tracking_error(black_box(a), black_box(m), 252.0)),
        );
    }
    group.finish();
}

criterion_group!(
    risk_benches,
    bench_value_at_risk,
    bench_cvar,
    bench_ratios,
    bench_drawdown,
    bench_beta,
    bench_promoted_metrics,
);
criterion_main!(risk_benches);
