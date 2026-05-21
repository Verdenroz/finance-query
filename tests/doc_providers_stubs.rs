//! Compile and runtime tests for provider stub pages
//! (docs/library/providers/polygon.md, fmp.md, alphavantage.md)
//!
//! Run with: `cargo test --test doc_providers_stubs`
//! Run network tests: `cargo test --test doc_providers_stubs -- --ignored`

use finance_query::Ticker;

// ---------------------------------------------------------------------------
// Compile-time: verify builder patterns from all three stubs compile
// ---------------------------------------------------------------------------

/// Polygon builder from polygon.md — routing through Providers API
#[allow(dead_code)]
#[cfg(feature = "polygon")]
fn _verify_polygon_builder() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let _providers = Providers::builder()
        .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential);
}

/// FMP builder from fmp.md — routing through Providers API
#[allow(dead_code)]
#[cfg(feature = "fmp")]
fn _verify_fmp_builder() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let _providers = Providers::builder()
        .route(Capability::QUOTE, &[Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::Sequential);
}

/// Alpha Vantage builder from alphavantage.md — routing through Providers API
#[allow(dead_code)]
#[cfg(feature = "alphavantage")]
fn _verify_alphavantage_builder() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let _providers = Providers::builder()
        .route(
            Capability::QUOTE,
            &[Provider::AlphaVantage, Provider::Yahoo],
        )
        .fetch(Fetch::Sequential);
}

/// Yahoo default builder (always available)
#[allow(dead_code)]
fn _verify_yahoo_default() {
    let _ = Ticker::builder("AAPL");
}

// ---------------------------------------------------------------------------
// Network tests — require respective API keys
// ---------------------------------------------------------------------------

#[cfg(feature = "polygon")]
#[tokio::test]
#[ignore = "requires network access and POLYGON_API_KEY"]
async fn test_polygon_quote() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let providers = Providers::builder()
        .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await
        .unwrap();
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}

#[cfg(feature = "fmp")]
#[tokio::test]
#[ignore = "requires network access and FMP_API_KEY"]
async fn test_fmp_quote() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let providers = Providers::builder()
        .route(Capability::QUOTE, &[Provider::Fmp, Provider::Yahoo])
        .fetch(Fetch::Sequential)
        .build()
        .await
        .unwrap();
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}

#[cfg(feature = "alphavantage")]
#[tokio::test]
#[ignore = "requires network access and ALPHA_VANTAGE_API_KEY"]
async fn test_alphavantage_quote() {
    use finance_query::{Capability, Fetch, Provider, Providers};
    let providers = Providers::builder()
        .route(
            Capability::QUOTE,
            &[Provider::AlphaVantage, Provider::Yahoo],
        )
        .fetch(Fetch::Sequential)
        .build()
        .await
        .unwrap();
    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}
