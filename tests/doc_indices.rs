//! Compile and runtime tests for docs/library/indices.md
//!
//! Requires the `polygon` feature flag:
//!   cargo test --test doc_indices --features polygon
//!   cargo test --test doc_indices --features polygon -- --ignored   (network tests)

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
// Network tests — mirror the setup block in indices.md
// ---------------------------------------------------------------------------

/// Mirrors the main setup block in indices.md:
///   providers.index("I:SPX") → quote(), chart(), history()
#[tokio::test]
#[ignore = "requires network access"]
async fn test_indices_spx_quote_chart_history() {
    // Build fails without POLYGON_API_KEY (e.g. CI without the secret); skip then.
    let Ok(providers) = Providers::builder()
        .route(Capability::INDICES, &[Provider::Polygon])
        .build()
        .await
    else {
        return;
    };

    let spx = providers.index("I:SPX");

    let quote = spx.quote().await.unwrap();
    assert!(!quote.symbol.is_empty(), "symbol should be set");

    let chart = spx
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty(), "chart should return candles");

    let history = spx.history(TimeRange::OneMonth).await.unwrap();
    assert!(!history.candles.is_empty(), "history should return candles");
}
