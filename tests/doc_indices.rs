//! Compile and runtime tests for docs/library/indices.md
//!
//! Requires the `polygon` feature flag:
//!   cargo test --test doc_indices --features polygon
//!
//! Runtime behavior of the index quote path is covered without network access
//! by the mock + unit tests in `src/adapters/polygon/indices/snapshots.rs` and
//! `src/adapters/fmp/indices/mod.rs` (mocked HTTP → DTO → canonical `IndexQuote`).

#![cfg(feature = "polygon")]

use finance_query::{Capability, IndexQuote, Interval, Provider, Providers, TimeRange};

// ---------------------------------------------------------------------------
// IndexQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all IndexQuote fields documented in indices.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_index_quote_fields(q: IndexQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}

/// Verifies Chart struct fields documented in indices.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_chart_fields(chart: finance_query::Chart) {
    let _: String = chart.symbol;
    let _: Vec<finance_query::Candle> = chart.candles;
    let _: Option<Interval> = chart.interval;
    let _: Option<TimeRange> = chart.range;
}

// ---------------------------------------------------------------------------
// Capability and Provider constants — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies that Capability::INDICES and Provider::Polygon exist as documented.
#[test]
fn test_indices_capability_and_provider_exist() {
    let _cap = Capability::INDICES;
    let _prov = Provider::Polygon;
}

// ---------------------------------------------------------------------------
// Compile-time: handle API — mirrors the setup block in indices.md
// ---------------------------------------------------------------------------

/// Verifies the documented `Index` flow type-checks:
///   providers.index("I:SPX") → quote(), chart(), history()
/// Never called; exists only for the compiler to type-check. Runtime behavior
/// is covered by the mock + unit tests in
/// `src/adapters/polygon/indices/snapshots.rs`.
#[allow(dead_code)]
async fn _verify_index_api() -> finance_query::Result<()> {
    let providers = Providers::builder()
        .route(Capability::INDICES, &[Provider::Polygon])
        .build()
        .await?;

    let spx = providers.index("I:SPX");
    let _quote: IndexQuote = spx.quote().await?;
    let _chart: finance_query::Chart = spx.chart(Interval::OneDay, TimeRange::OneMonth).await?;
    let _history: finance_query::Chart = spx.history(TimeRange::OneMonth).await?;
    Ok(())
}
