//! Criterion benchmarks for provider-dispatch capability filtering.
//!
//! The network side of dispatch can't be benchmarked offline, but the routing
//! decision that runs before every fetch — selecting which providers serve a
//! requested [`Capability`] — is pure bitflag logic on the hot path. This
//! measures that selection over a registry the size of the real one.
//!
//! ```text
//! cargo bench --bench providers
//! ```

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use finance_query::Capability;

/// A registry mirroring the real provider set: each entry is the union of
/// capabilities a provider declares (see `src/providers/<name>.rs`).
fn registry() -> Vec<(&'static str, Capability)> {
    use Capability as C;
    vec![
        (
            "yahoo",
            C::QUOTE | C::CHART | C::FUNDAMENTALS | C::CORPORATE | C::OPTIONS,
        ),
        (
            "fmp",
            C::QUOTE
                | C::CHART
                | C::FUNDAMENTALS
                | C::FOREX
                | C::CRYPTO
                | C::COMMODITIES
                | C::INDICES,
        ),
        (
            "alphavantage",
            C::QUOTE
                | C::CHART
                | C::FUNDAMENTALS
                | C::OPTIONS
                | C::FOREX
                | C::CRYPTO
                | C::ECONOMIC
                | C::COMMODITIES,
        ),
        (
            "polygon",
            C::QUOTE
                | C::CHART
                | C::FUNDAMENTALS
                | C::CORPORATE
                | C::OPTIONS
                | C::CRYPTO
                | C::FOREX
                | C::FUTURES
                | C::INDICES
                | C::FILINGS
                | C::ECONOMIC,
        ),
        ("coingecko", C::CRYPTO),
        ("fred", C::ECONOMIC),
        ("edgar", C::FILINGS),
    ]
}

/// Replicates the per-fetch selection: collect providers declaring `wanted`.
fn select(registry: &[(&'static str, Capability)], wanted: Capability) -> Vec<&'static str> {
    registry
        .iter()
        .filter(|(_, caps)| caps.contains(wanted))
        .map(|(id, _)| *id)
        .collect()
}

fn bench_capability_filtering(c: &mut Criterion) {
    let registry = registry();
    let mut group = c.benchmark_group("capability_filtering");

    for (label, wanted) in [
        ("quote", Capability::QUOTE),
        ("crypto", Capability::CRYPTO),
        ("filings", Capability::FILINGS),
        ("quote_and_chart", Capability::QUOTE | Capability::CHART),
    ] {
        group.bench_with_input(BenchmarkId::new("select", label), &wanted, |b, &w| {
            b.iter(|| select(black_box(&registry), black_box(w)))
        });
    }
    group.finish();
}

fn bench_capability_predicate(c: &mut Criterion) {
    let mut group = c.benchmark_group("capability_predicate");
    let combined = Capability::QUOTE | Capability::CHART | Capability::FUNDAMENTALS;

    group.bench_function("contains_hit", |b| {
        b.iter(|| black_box(combined).contains(black_box(Capability::CHART)))
    });
    group.bench_function("contains_miss", |b| {
        b.iter(|| black_box(combined).contains(black_box(Capability::OPTIONS)))
    });
    group.bench_function("name", |b| {
        b.iter(|| black_box(Capability::FUNDAMENTALS).name())
    });
    group.finish();
}

criterion_group!(
    providers_benches,
    bench_capability_filtering,
    bench_capability_predicate,
);
criterion_main!(providers_benches);
