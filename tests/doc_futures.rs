//! Compile and runtime tests for docs/library/futures.md
//!
//! Run: cargo test --test doc_futures --features polygon
//! Network tests: cargo test --test doc_futures --features polygon -- --ignored

#![cfg(feature = "polygon")]

use finance_query::FuturesQuote;

// ---------------------------------------------------------------------------
// FuturesQuote — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all FuturesQuote fields documented in futures.md exist with correct
/// types. Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_futures_quote_fields(q: FuturesQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.name;
    let _: Option<String> = q.underlying;
    let _: Option<String> = q.exchange;
    let _: Option<String> = q.expiration_date;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<u64> = q.open_interest;
    let _: Option<u64> = q.volume;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Network test — mirrors the "Getting a Handle" block in futures.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access (POLYGON_API_KEY)"]
async fn test_futures_contract() {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    // Build fails without POLYGON_API_KEY (e.g. CI without the secret); skip then.
    let Ok(providers) = Providers::builder()
        .route(Capability::FUTURES, &[Provider::Polygon])
        .build()
        .await
    else {
        return;
    };

    let contract = providers.futures("ES");
    let _quote = contract.quote().await.unwrap();
    let chart = contract
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty());
    let history = contract.history(TimeRange::OneMonth).await.unwrap();
    assert!(!history.candles.is_empty());
}
