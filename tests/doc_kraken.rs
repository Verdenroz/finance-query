//! Compile and runtime tests for docs/library/providers/kraken.md
//!
//! Requires the `kraken` feature flag:
//!   cargo test --test doc_kraken --features kraken
//!   cargo test --test doc_kraken --features kraken -- --ignored
//!
//! Offline behaviour (XBT/XDG aliasing, X/Z prefix stripping, the HTTP-200
//! error envelope, the `last` cursor) is covered by the mock + unit tests in
//! `src/adapters/kraken/`.

#![cfg(feature = "kraken")]

use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[test]
fn kraken_provider_id_round_trips() {
    assert_eq!(Provider::Kraken.as_str(), "kraken");
    assert_eq!(Provider::from_id_str("kraken"), Some(Provider::Kraken));
}

/// The point of this provider: keyless *and* reachable from the US, unlike
/// Binance.
#[tokio::test]
#[ignore = "requires network access"]
async fn keyless_quote_by_coin_id() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Kraken])
        .build()
        .await
        .expect("Kraken public data needs no API key");

    let quote = providers.crypto("bitcoin").quote("usd").await.unwrap();

    assert_eq!(quote.id, "bitcoin");
    // Kraken calls it XBT internally; callers see BTC.
    assert_eq!(quote.symbol, "BTC");
    assert_eq!(quote.name, "Bitcoin");
    assert!(quote.price.is_some_and(|p| p > 0.0));
    assert!(quote.high_24h.is_some());
    assert_eq!(quote.market_cap, None);
}

/// A modern pair with no legacy prefixes must work the same as a legacy one.
#[tokio::test]
#[ignore = "requires network access"]
async fn modern_pairs_need_no_aliasing() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Kraken])
        .build()
        .await
        .unwrap();

    let quote = providers.crypto("solana").quote("usd").await.unwrap();
    assert_eq!(quote.symbol, "SOL");
    assert!(quote.price.is_some_and(|p| p > 0.0));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn daily_candles_are_already_in_seconds() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Kraken])
        .route(Capability::CHART, [Provider::Kraken])
        .build()
        .await
        .unwrap();

    let chart = providers
        .crypto("BTC")
        .chart("usd", Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    assert!(!chart.candles.is_empty());
    assert!(chart.candles[0].timestamp < 100_000_000_000);
    assert!(
        chart
            .candles
            .windows(2)
            .all(|w| w[0].timestamp < w[1].timestamp),
        "candles are not in ascending time order"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn unknown_pair_is_an_error_despite_kraken_returning_http_200() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Kraken])
        .build()
        .await
        .unwrap();

    assert!(
        providers
            .crypto("definitely-not-a-coin")
            .quote("usd")
            .await
            .is_err()
    );
}
