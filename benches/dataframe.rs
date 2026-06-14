//! Polars DataFrame conversion benchmarks (feature: `dataframe`).
//!
//! Exercises the `ToDataFrame` derive output (`Candle::vec_to_dataframe`) and
//! the convenience wrappers `Chart::to_dataframe` / `Contracts::to_dataframe`.
//! Inputs are the real captured server responses in `benches/fixtures/` plus a
//! deterministic synthetic candle series for scaling.
//!
//! Criterion-only: this is deliberately excluded from the `regression` gate
//! because Polars is far too heavy to build/run under the CI valgrind gate.
//!
//! ```text
//! cargo bench --bench dataframe --features finance-query/dataframe
//! ```

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use finance_query::{Candle, Chart, Options};

static CHART: &str = include_str!("fixtures/chart.json");
static OPTIONS: &str = include_str!("fixtures/options.json");

/// Deterministic synthetic candles (`Candle` is `#[non_exhaustive]` — build via
/// serde_json, matching the other benches).
fn synthetic_candles(n: usize) -> Vec<Candle> {
    let mut price = 100.0_f64;
    (0..n)
        .map(|i| {
            price += ((i as f64) * 0.1).sin();
            serde_json::from_value(serde_json::json!({
                "timestamp": 1_700_000_000_i64 + i as i64 * 86_400,
                "open": price, "high": price + 1.0, "low": price - 1.0,
                "close": price, "volume": 1_000_000_i64, "adjClose": price,
            }))
            .unwrap()
        })
        .collect()
}

fn bench_candles_to_dataframe(c: &mut Criterion) {
    let mut g = c.benchmark_group("candles_to_dataframe");
    for n in [100usize, 1000, 5000] {
        let candles = synthetic_candles(n);
        g.bench_with_input(BenchmarkId::from_parameter(n), &candles, |b, candles| {
            b.iter(|| black_box(Candle::vec_to_dataframe(black_box(candles)).unwrap()))
        });
    }
    g.finish();
}

fn bench_real_responses(c: &mut Criterion) {
    let chart: Chart = serde_json::from_str(CHART).unwrap();
    let options: Options = serde_json::from_str(OPTIONS).unwrap();
    let calls = options.calls();

    let mut g = c.benchmark_group("response_to_dataframe");
    g.bench_function("chart", |b| {
        b.iter(|| black_box(black_box(&chart).to_dataframe().unwrap()))
    });
    g.bench_function("option_calls", |b| {
        b.iter(|| black_box(black_box(&calls).to_dataframe().unwrap()))
    });
    g.finish();
}

criterion_group!(
    dataframe_benches,
    bench_candles_to_dataframe,
    bench_real_responses
);
criterion_main!(dataframe_benches);
