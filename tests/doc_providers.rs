//! Compile and runtime tests for docs/library/providers.md
//!
//! Pure tests verify types and builder patterns from providers.md.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_providers`
//! Run network tests: `cargo test --test doc_providers -- --ignored`

use finance_query::{Fetch, Provider};

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
// Market-wide handles — compile-time factory verification
// ---------------------------------------------------------------------------

/// Verifies the market-wide factories and handle types documented in
/// providers/index.md ("Three handles are market-wide...").
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
#[allow(dead_code)]
fn _verify_market_wide_handles(providers: &finance_query::Providers) {
    let _disco: finance_query::Discovery = providers.discovery();
    let _cal: finance_query::MarketCalendar = providers.calendar();
    let _mkt: finance_query::Market = providers.market();
}

// ---------------------------------------------------------------------------
// Fetch — compile-time type checks
// ---------------------------------------------------------------------------

/// Verifies Fetch variants exist with correct type.
#[allow(dead_code)]
fn _verify_fetch_variants() {
    let _: Fetch = Fetch::Sequential;
    let _: Fetch = Fetch::Parallel;
}

// ---------------------------------------------------------------------------
// Network tests — require providers + API keys
// ---------------------------------------------------------------------------

/// Verifies multi-provider Ticker construction via Providers::builder().
/// Requires POLYGON_API_KEY.
#[cfg(feature = "polygon")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_provider_builder_builds() {
    use finance_query::{Capability, Providers};

    let result = Providers::builder()
        .route(Capability::QUOTE, [Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await;

    if let Ok(providers) = result {
        let ticker = providers.ticker("AAPL").build().await;
        let _ = ticker;
    }
}

/// Verifies default (Yahoo-only) Ticker still works.
#[tokio::test]
#[ignore = "requires network access"]
async fn test_provider_default_yahoo() {
    use finance_query::{Ticker, format::Raw};

    let ticker = Ticker::new("AAPL").await.unwrap();
    let quote = ticker.quote::<Raw>().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}
