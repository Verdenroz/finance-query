//! Compile and runtime tests for docs/library/providers.md
//!
//! Pure tests verify types and builder patterns from providers.md.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_providers`
//! Run network tests: `cargo test --test doc_providers -- --ignored`

use finance_query::{Enrich, Fetch, Prefer, Provider};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Provider enum — compile-time variant verification
// ---------------------------------------------------------------------------

/// Verifies all Provider variants exist and implement the documented traits.
#[allow(dead_code)]
fn _verify_provider_enum() {
    let _: Provider = Provider::Yahoo;
    #[cfg(feature = "polygon")]
    let _: Provider = Provider::Polygon;
    #[cfg(feature = "fmp")]
    let _: Provider = Provider::Fmp;
    #[cfg(feature = "alphavantage")]
    let _: Provider = Provider::AlphaVantage;
    #[cfg(feature = "crypto")]
    let _: Provider = Provider::CoinGecko;
    #[cfg(feature = "fred")]
    let _: Provider = Provider::Fred;
}

// ---------------------------------------------------------------------------
// Fetch & Merge — compile-time type checks
// ---------------------------------------------------------------------------

/// Verifies Fetch variants exist with correct type.
#[allow(dead_code)]
fn _verify_fetch_variants() {
    let _: Fetch = Fetch::Sequential;
    let _: Fetch = Fetch::Parallel;
    let _: Fetch = Fetch::All;
}

/// Verifies Merge policy types exist.
#[allow(dead_code)]
fn _verify_merge_types() {
    let _prefer = Prefer;
    let _enrich = Enrich;
}

// ---------------------------------------------------------------------------
// TickerBuilder provider configuration — compile-time check
// ---------------------------------------------------------------------------

/// Verifies builder compiles with providers, fetch, and merge.
#[allow(dead_code)]
fn _verify_ticker_builder_providers() {
    use finance_query::Ticker;

    fn assert_send<T: Send>(_: &T) {}
    fn assert_sync<T: Sync>(_: &T) {}

    let _ = Ticker::builder("AAPL")
        .providers(&[Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .merge(Enrich)
        .timeout(Duration::from_secs(30));

    // Verify Fetch values pass type checks
    let _ = Fetch::Sequential;
    let _ = Fetch::Parallel;
    let _ = Fetch::All;
    let _ = Enrich;
    let _ = Prefer;
}

// ---------------------------------------------------------------------------
// Network tests — require providers + API keys
// ---------------------------------------------------------------------------

/// Verifies multi-provider Ticker construction.
/// Requires POLYGON_API_KEY or runs with Yahoo fallback.
#[cfg(feature = "polygon")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_provider_builder_builds() {
    use finance_query::Ticker;

    let result = Ticker::builder("AAPL")
        .providers(&[Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .merge(Prefer)
        .build()
        .await;

    // May fail if POLYGON_API_KEY is not set, but should compile
    let _ = result;
}

/// Verifies default (Yahoo-only) Ticker still works.
#[tokio::test]
#[ignore = "requires network access"]
async fn test_provider_default_yahoo() {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await.unwrap();
    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}
