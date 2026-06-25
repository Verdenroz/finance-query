//! Network integration tests for `indicators()` / `indicator()` / `risk()` on
//! domain handles (#198). These reuse each handle's cached `chart()` and the
//! shared indicator/risk engines, with asset-class-aware risk annualization.
//!
//!   cargo test --test domain_analytics --features risk,fmp,polygon,crypto -- --ignored

#![cfg(all(feature = "risk", feature = "fmp"))]

use finance_query::indicators::{Indicator, IndicatorResult};
use finance_query::{Interval, Providers, TimeRange};

#[tokio::test]
#[ignore = "requires network access"]
async fn index_indicators_and_risk() {
    let providers = Providers::builder().build().await.unwrap();
    let spx = providers.index("^GSPC");

    let summary = spx
        .indicators(Interval::OneDay, TimeRange::SixMonths)
        .await
        .unwrap();
    // RSI is part of the full summary; just assert we got data back.
    let _ = summary;

    let rsi = spx
        .indicator(Indicator::Rsi(14), Interval::OneDay, TimeRange::SixMonths)
        .await
        .unwrap();
    assert!(matches!(rsi, IndicatorResult::Series(_)));

    let risk = spx
        .risk(Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    assert!(risk.beta.is_none(), "no benchmark → beta None");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn forex_indicator() {
    let providers = Providers::builder().build().await.unwrap();
    let eur = providers.forex("USD", "EUR");
    let ema = eur
        .indicator(Indicator::Ema(20), Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();
    assert!(matches!(ema, IndicatorResult::Series(_)));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn crypto_indicators_and_risk_use_vs_currency() {
    let providers = Providers::builder().build().await.unwrap();
    let btc = providers.crypto("BTC");

    let macd = btc
        .indicator(
            Indicator::Macd {
                fast: 12,
                slow: 26,
                signal: 9,
            },
            "USD",
            Interval::OneDay,
            TimeRange::SixMonths,
        )
        .await
        .unwrap();
    assert!(matches!(macd, IndicatorResult::Macd(_)));

    let risk = btc
        .risk("USD", Interval::OneDay, TimeRange::OneYear)
        .await
        .unwrap();
    assert!(risk.beta.is_none());
}
