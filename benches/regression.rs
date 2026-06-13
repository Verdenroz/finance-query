//! Deterministic instruction-count regression gate (iai-callgrind).
//!
//! Unlike the criterion benches in this directory — which measure wall-clock
//! time and are too noisy to gate CI on — these benchmarks count CPU
//! instructions via valgrind/callgrind. Instruction counts are deterministic
//! across runs and machines, so a change in the count is a real change in the
//! work done, not measurement noise. That makes them safe to fail CI on.
//!
//! Each benchmark carries a soft limit of +5% on the instruction count (`Ir`).
//! CI saves a baseline from the target branch, then re-runs on the PR; any
//! benchmark exceeding its baseline by more than 5% fails the gate.
//!
//! ## Running
//!
//! Requires `valgrind`. This host's environment matters: a glibc compiled for
//! `x86-64-v4` (e.g. CachyOS/Arch with AVX-512) emits AVX-512 in its startup
//! code, which valgrind 3.25 cannot decode (SIGILL). Run on vanilla glibc
//! (CI's Ubuntu, or `make bench-regression` which uses a Debian container):
//!
//! ```text
//! cargo bench --bench regression --features finance-query/risk
//! ```
//!
//! The `GLIBC_TUNABLES` entry below additionally stops glibc from dispatching
//! to AVX-512 string routines (`memcpy`/`memmove`) at runtime, hardening the
//! gate on AVX-512 hosts that *do* have a valgrind-decodable loader.

use finance_query::indicators::{ema, macd, rsi, sma};
use finance_query::risk::{
    beta, historical_var, max_drawdown, parametric_var, sharpe_ratio, sortino_ratio,
};
use finance_query::{Currency, News, SearchResults};
use iai_callgrind::{
    Callgrind, EventKind, LibraryBenchmarkConfig, library_benchmark, library_benchmark_group, main,
};
use std::hint::black_box;

// ── Deterministic synthetic inputs (no network, no randomness) ───────────────

/// A reproducible pseudo-random walk of close prices.
fn synthetic_closes(n: usize) -> Vec<f64> {
    let mut price = 100.0_f64;
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    (0..n)
        .map(|_| {
            // xorshift — deterministic, branch-light
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let step = ((state >> 40) as f64 / (1u64 << 24) as f64 - 0.5) * 2.0;
            price = (price + step).max(1.0);
            price
        })
        .collect()
}

/// Close-to-close simple returns derived from a synthetic price series.
fn synthetic_returns(n: usize) -> Vec<f64> {
    let closes = synthetic_closes(n + 1);
    closes.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect()
}

// Setup functions run *outside* the measured region — iai-callgrind requires a
// function path (not a closure) for the `setup` argument.
fn closes_2000() -> Vec<f64> {
    synthetic_closes(2000)
}
fn returns_1000() -> Vec<f64> {
    synthetic_returns(1000)
}
fn returns_pair_1000() -> (Vec<f64>, Vec<f64>) {
    (synthetic_returns(1000), synthetic_returns(1000))
}

// ── Indicator hot paths ──────────────────────────────────────────────────────

#[library_benchmark]
#[bench::n2000(setup = closes_2000)]
fn ind_sma(closes: Vec<f64>) -> Vec<Option<f64>> {
    black_box(sma(black_box(&closes), 200))
}

#[library_benchmark]
#[bench::n2000(setup = closes_2000)]
fn ind_ema(closes: Vec<f64>) -> Vec<Option<f64>> {
    black_box(ema(black_box(&closes), 200))
}

#[library_benchmark]
#[bench::n2000(setup = closes_2000)]
fn ind_rsi(closes: Vec<f64>) -> Vec<Option<f64>> {
    black_box(rsi(black_box(&closes), 14).unwrap())
}

#[library_benchmark]
#[bench::n2000(setup = closes_2000)]
fn ind_macd(closes: Vec<f64>) -> finance_query::indicators::MacdResult {
    black_box(macd(black_box(&closes), 12, 26, 9).unwrap())
}

library_benchmark_group!(
    name = indicators;
    benchmarks = ind_sma, ind_ema, ind_rsi, ind_macd
);

// ── Risk metric hot paths ────────────────────────────────────────────────────

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_historical_var(returns: Vec<f64>) -> Option<f64> {
    black_box(historical_var(black_box(&returns), 0.95))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_parametric_var(returns: Vec<f64>) -> Option<f64> {
    black_box(parametric_var(black_box(&returns), 0.95))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_sharpe(returns: Vec<f64>) -> Option<f64> {
    black_box(sharpe_ratio(black_box(&returns), 0.0, 252.0))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_sortino(returns: Vec<f64>) -> Option<f64> {
    black_box(sortino_ratio(black_box(&returns), 0.0, 252.0))
}

#[library_benchmark]
#[bench::n1000(setup = returns_1000)]
fn risk_max_drawdown(returns: Vec<f64>) -> (f64, Option<u64>) {
    // `DrawdownResult` is not publicly nameable; return its fields so the full
    // computation is still measured and not optimised away.
    let dd = max_drawdown(black_box(&returns));
    black_box((dd.max_drawdown, dd.recovery_periods))
}

#[library_benchmark]
#[bench::n1000(setup = returns_pair_1000)]
fn risk_beta(series: (Vec<f64>, Vec<f64>)) -> Option<f64> {
    black_box(beta(black_box(&series.0), black_box(&series.1)))
}

library_benchmark_group!(
    name = risk;
    benchmarks = risk_historical_var, risk_parametric_var, risk_sharpe, risk_sortino,
        risk_max_drawdown, risk_beta
);

// ── Deserialization hot paths (real Yahoo-shaped fixtures) ───────────────────

static SEARCH_JSON: &str = include_str!("fixtures/search.json");
static NEWS_JSON: &str = include_str!("fixtures/news.json");
static CURRENCIES_JSON: &str = include_str!("fixtures/currencies.json");

#[library_benchmark]
fn de_search() -> SearchResults {
    black_box(serde_json::from_str(black_box(SEARCH_JSON)).unwrap())
}

#[library_benchmark]
fn de_news() -> Vec<News> {
    black_box(serde_json::from_str(black_box(NEWS_JSON)).unwrap())
}

#[library_benchmark]
fn de_currencies() -> Vec<Currency> {
    black_box(serde_json::from_str(black_box(CURRENCIES_JSON)).unwrap())
}

library_benchmark_group!(
    name = deserialize;
    benchmarks = de_search, de_news, de_currencies
);

// ── Gate: fail any benchmark whose instruction count regresses > 5% ──────────

main!(
    config = LibraryBenchmarkConfig::default()
        .tool(Callgrind::default().soft_limits([(EventKind::Ir, 5.0)]))
        .env(
            "GLIBC_TUNABLES",
            "glibc.cpu.hwcaps=-AVX512F,-AVX512VL,-AVX512BW,-AVX512DQ,-AVX512CD,-AVX512IFMA,-AVX512_VBMI,-AVX512_VBMI2,-AVX512_VNNI,-AVX512_BITALG,-AVX512_VPOPCNTDQ",
        );
    library_benchmark_groups = indicators, risk, deserialize
);
