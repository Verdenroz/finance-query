//! Compile and runtime tests for docs/library/commodities.md
//!
//! Requires the `fmp` feature flag:
//!   cargo test --test doc_commodities --features fmp
//!   cargo test --test doc_commodities --features fmp -- --ignored   (network tests)

#![cfg(feature = "fmp")]

use finance_query::CommodityQuote;

// ---------------------------------------------------------------------------
// CommodityQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all CommodityQuote fields documented in commodities.md exist with
/// the correct types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_commodity_quote_fields(q: CommodityQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.unit;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

/// Mirrors the main handle block from commodities.md.
/// Tests `providers.commodity(symbol)` with `quote()`, `chart()`, `history()`.
#[tokio::test]
#[ignore = "requires network access"]
async fn test_commodity_quote_chart_history() {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::COMMODITIES, &[Provider::Fmp])
        .build()
        .await
        .unwrap();
    let gold = providers.commodity("GCUSD");
    let quote = gold.quote().await.unwrap();
    let chart = gold
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let history = gold.history(TimeRange::OneMonth).await.unwrap();

    assert_eq!(quote.symbol, "GCUSD");
    assert!(!chart.symbol.is_empty());
    assert!(!history.symbol.is_empty());
}
