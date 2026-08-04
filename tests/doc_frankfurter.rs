//! Compile and runtime tests for docs/library/providers/frankfurter.md
//!
//! Requires the `frankfurter` feature flag:
//!   cargo test --test doc_frankfurter --features frankfurter
//!   cargo test --test doc_frankfurter --features frankfurter -- --ignored
//!
//! Offline behaviour (change computation, chronological ordering, the
//! same-currency short circuit, error mapping) is covered by the mock + unit
//! tests in `src/adapters/frankfurter/mod.rs`.

#![cfg(feature = "frankfurter")]

use finance_query::{Capability, Provider, Providers};

#[test]
fn frankfurter_provider_id_round_trips() {
    assert_eq!(Provider::Frankfurter.as_str(), "frankfurter");
    assert_eq!(
        Provider::from_id_str("frankfurter"),
        Some(Provider::Frankfurter)
    );
}

/// The point of the provider: `providers.forex()` works with nothing
/// configured, which was impossible before.
#[tokio::test]
#[ignore = "requires network access"]
async fn forex_works_with_no_key_configured() {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::Frankfurter])
        .build()
        .await
        .expect("Frankfurter needs no API key");

    let quote = providers.forex("USD", "EUR").quote().await.unwrap();

    assert_eq!(quote.symbol, "USDEUR");
    assert_eq!(quote.base_currency.as_deref(), Some("USD"));
    assert_eq!(quote.quote_currency.as_deref(), Some("EUR"));
    assert!(quote.price.is_some_and(|p| p > 0.0));
    // A reference fix has no two-way price.
    assert_eq!(quote.bid, None);
    assert_eq!(quote.ask, None);
    assert!(quote.timestamp.is_some());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn identical_currencies_quote_at_one() {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::Frankfurter])
        .build()
        .await
        .unwrap();

    let quote = providers.forex("USD", "USD").quote().await.unwrap();
    assert_eq!(quote.price, Some(1.0));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn currency_outside_ecb_coverage_is_not_found() {
    let providers = Providers::builder()
        .route(Capability::FOREX, [Provider::Frankfurter])
        .build()
        .await
        .unwrap();

    assert!(providers.forex("USD", "ZZZ").quote().await.is_err());
}
