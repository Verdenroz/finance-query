//! Compile and runtime tests for docs/library/providers/binance.md
//!
//! Requires the `binance` feature flag:
//!   cargo test --test doc_binance --features binance
//!   cargo test --test doc_binance --features binance -- --ignored
//!
//! Offline behaviour (symbol normalisation, kline parsing, millisecond
//! conversion, error mapping) is covered by the mock + unit tests in
//! `src/adapters/binance/`.
//!
//! The network tests are additionally skipped by CI runners in a region
//! Binance geo-blocks — they will fail with an HTTP 451 there, which is the
//! documented behaviour rather than a bug.

#![cfg(feature = "binance")]

use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

#[test]
fn binance_provider_id_round_trips() {
    assert_eq!(Provider::Binance.as_str(), "binance");
    assert_eq!(Provider::from_id_str("binance"), Some(Provider::Binance));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn keyless_quote_by_coin_id() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Binance])
        .build()
        .await
        .expect("Binance public data needs no API key");

    let quote = providers.crypto("bitcoin").quote("usd").await.unwrap();

    assert_eq!(quote.id, "bitcoin");
    assert_eq!(quote.symbol, "BTC");
    assert_eq!(quote.name, "Bitcoin");
    assert!(quote.price.is_some_and(|p| p > 0.0));
    // An exchange reports flow, not supply.
    assert_eq!(quote.market_cap, None);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn hourly_candles_come_back_in_seconds() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Binance])
        .route(Capability::CHART, [Provider::Binance])
        .build()
        .await
        .unwrap();

    let chart = providers
        .crypto("BTC")
        .chart("usd", Interval::OneHour, TimeRange::FiveDays)
        .await
        .unwrap();

    assert!(!chart.candles.is_empty());
    // Seconds, not Binance's milliseconds.
    assert!(chart.candles[0].timestamp < 100_000_000_000);
    assert!(
        chart
            .candles
            .windows(2)
            .all(|w| w[0].timestamp < w[1].timestamp),
        "candles are not in ascending time order"
    );
}

/// A window longer than Binance's 1000-candle page cap must be walked, not
/// truncated.
#[tokio::test]
#[ignore = "requires network access"]
async fn long_windows_page_past_the_thousand_candle_cap() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Binance])
        .route(Capability::CHART, [Provider::Binance])
        .build()
        .await
        .unwrap();

    let chart = providers
        .crypto("BTC")
        .chart("usd", Interval::OneHour, TimeRange::SixMonths)
        .await
        .unwrap();

    assert!(
        chart.candles.len() > 1000,
        "expected pagination past the 1000-candle cap, got {}",
        chart.candles.len()
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn unlisted_market_is_an_error() {
    let providers = Providers::builder()
        .route(Capability::CRYPTO, [Provider::Binance])
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
