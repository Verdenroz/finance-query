//! Network integration tests for `chart()` / `history()` on domain handles (#197).
//!
//! All chart-capable handles route through `Capability::CHART` (Yahoo by
//! default), so these exercise the symbol mapping per asset class against the
//! live Yahoo chart endpoint.
//!
//!   cargo test --test domain_charts --features fmp,polygon,crypto -- --ignored

#![cfg(feature = "fmp")]

use finance_query::{Interval, Providers, TimeRange};

#[tokio::test]
#[ignore = "requires network access"]
async fn forex_chart_maps_to_yahoo_fx_symbol() {
    let providers = Providers::builder().build().await.unwrap();
    let pair = providers.forex("USD", "EUR");
    let chart = pair
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty(), "expected USDEUR=X candles");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn index_chart_passes_symbol_through() {
    let providers = Providers::builder().build().await.unwrap();
    let idx = providers.index("^GSPC");
    let chart = idx.history(TimeRange::OneMonth).await.unwrap();
    assert!(!chart.candles.is_empty(), "expected ^GSPC candles");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn commodity_chart_uses_futures_symbol() {
    let providers = Providers::builder().build().await.unwrap();
    let gold = providers.commodity("GC=F");
    let chart = gold
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty(), "expected GC=F candles");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn crypto_chart_builds_ticker_pair_symbol() {
    let providers = Providers::builder().build().await.unwrap();
    // Ticker-style id resolves on the default Yahoo route ("BTC-USD").
    let btc = providers.crypto("BTC");
    let chart = btc
        .chart("USD", Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty(), "expected BTC-USD candles");
}

#[cfg(feature = "polygon")]
#[tokio::test]
#[ignore = "requires network access"]
async fn futures_chart_passes_symbol_through() {
    let providers = Providers::builder().build().await.unwrap();
    let cl = providers.futures("CL=F");
    let chart = cl.history(TimeRange::OneMonth).await.unwrap();
    assert!(!chart.candles.is_empty(), "expected CL=F candles");
}
