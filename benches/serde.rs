//! Model (de)serialization benchmarks over **real** server responses.
//!
//! Every fixture in `benches/fixtures/` was captured from the running server
//! (`GET /v2/...`) so the shapes match production exactly — see
//! `.claude/rules/benches.md`. Two groups:
//!
//! 1. `typed_deserialize` / `serialize` — round-trips the heavy structured
//!    response types through their public `serde` impls. This is the path that
//!    actually regresses when a model changes, and mirrors the gate in
//!    `benches/regression.rs`.
//! 2. `parse_value` — parses every endpoint payload into `serde_json::Value`.
//!    This is the lower-bound parse cost and gives coverage for *every* endpoint
//!    we expose (including ones whose server response is an untyped JSON value).
//!
//! ```text
//! cargo bench --bench serde
//! ```

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use finance_query::{
    Chart, CompanyFacts, EdgarSubmissions, FinancialStatement, News, Options, Quote,
    ScreenerResults, SearchResults,
};

// ── Typed fixtures (fixture text, public type) ───────────────────────────────
static QUOTE: &str = include_str!("fixtures/quote.json");
static CHART: &str = include_str!("fixtures/chart.json");
static OPTIONS: &str = include_str!("fixtures/options.json");
static FINANCIALS: &str = include_str!("fixtures/financials.json");
static SCREENER: &str = include_str!("fixtures/screener.json");
static EDGAR_SUBMISSIONS: &str = include_str!("fixtures/edgar_submissions.json");
static EDGAR_FACTS: &str = include_str!("fixtures/edgar_facts.json");
static NEWS: &str = include_str!("fixtures/news_symbol.json");
static SEARCH: &str = include_str!("fixtures/search.json");

/// Every captured endpoint payload, benchmarked as a raw `Value` parse.
static ALL_FIXTURES: &[(&str, &str)] = &[
    ("quote", QUOTE),
    ("chart", CHART),
    ("options", OPTIONS),
    ("financials", FINANCIALS),
    ("screener", SCREENER),
    ("edgar_submissions", EDGAR_SUBMISSIONS),
    ("edgar_facts", EDGAR_FACTS),
    ("news", NEWS),
    ("search", SEARCH),
    ("holders", include_str!("fixtures/holders.json")),
    ("analysis", include_str!("fixtures/analysis.json")),
    (
        "recommendations",
        include_str!("fixtures/recommendations.json"),
    ),
    ("indices", include_str!("fixtures/indices.json")),
    ("sector", include_str!("fixtures/sector.json")),
    ("industry", include_str!("fixtures/industry.json")),
    ("transcripts", include_str!("fixtures/transcripts.json")),
    ("lookup", include_str!("fixtures/lookup.json")),
    ("exchanges", include_str!("fixtures/exchanges.json")),
    ("crypto", include_str!("fixtures/crypto.json")),
    ("fred_series", include_str!("fixtures/fred_series.json")),
    (
        "treasury_yields",
        include_str!("fixtures/treasury_yields.json"),
    ),
];

fn bench_typed_deserialize(c: &mut Criterion) {
    let mut g = c.benchmark_group("typed_deserialize");
    macro_rules! de {
        ($name:literal, $ty:ty, $json:expr) => {{
            g.throughput(Throughput::Bytes($json.len() as u64));
            g.bench_function($name, |b| {
                b.iter(|| {
                    let v: $ty = serde_json::from_str(black_box($json)).unwrap();
                    black_box(v);
                })
            });
        }};
    }
    de!("quote", Quote, QUOTE);
    de!("chart", Chart, CHART);
    de!("options", Options, OPTIONS);
    de!("financials", FinancialStatement, FINANCIALS);
    de!("screener", ScreenerResults, SCREENER);
    de!("edgar_submissions", EdgarSubmissions, EDGAR_SUBMISSIONS);
    de!("edgar_facts", CompanyFacts, EDGAR_FACTS);
    de!("news", Vec<News>, NEWS);
    de!("search", SearchResults, SEARCH);
    g.finish();
}

fn bench_serialize(c: &mut Criterion) {
    let mut g = c.benchmark_group("serialize");
    macro_rules! ser {
        ($name:literal, $ty:ty, $json:expr) => {{
            let value: $ty = serde_json::from_str($json).unwrap();
            g.bench_function($name, |b| {
                b.iter(|| {
                    let s = serde_json::to_string(black_box(&value)).unwrap();
                    black_box(s);
                })
            });
        }};
    }
    ser!("quote", Quote, QUOTE);
    ser!("chart", Chart, CHART);
    ser!("options", Options, OPTIONS);
    ser!("financials", FinancialStatement, FINANCIALS);
    ser!("screener", ScreenerResults, SCREENER);
    ser!("edgar_submissions", EdgarSubmissions, EDGAR_SUBMISSIONS);
    ser!("edgar_facts", CompanyFacts, EDGAR_FACTS);
    ser!("news", Vec<News>, NEWS);
    ser!("search", SearchResults, SEARCH);
    g.finish();
}

/// Raw `serde_json::Value` parse for every endpoint payload — covers endpoints
/// whose typed model isn't round-tripped above, by real response size.
fn bench_parse_value(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse_value");
    for (name, json) in ALL_FIXTURES {
        g.throughput(Throughput::Bytes(json.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(name), json, |b, json| {
            b.iter(|| {
                let v: serde_json::Value = serde_json::from_str(black_box(json)).unwrap();
                black_box(v);
            })
        });
    }
    g.finish();
}

criterion_group!(
    serde_benches,
    bench_typed_deserialize,
    bench_serialize,
    bench_parse_value
);
criterion_main!(serde_benches);
