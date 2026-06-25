//! Compile and runtime tests for docs/library/forex.md
//!
//! Run: cargo test --test doc_forex --features alphavantage
//! Network tests: cargo test --test doc_forex --features alphavantage -- --ignored
#![cfg(feature = "alphavantage")]

// ---------------------------------------------------------------------------
// Compile-time: ForexQuote field verification
// ---------------------------------------------------------------------------

/// Verifies all ForexQuote fields documented in forex.md exist with correct types.
/// Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_forex_quote_fields(q: finance_query::ForexQuote) {
    let _: String = q.symbol;
    let _: Option<String> = q.base_currency;
    let _: Option<String> = q.quote_currency;
    let _: Option<f64> = q.bid;
    let _: Option<f64> = q.ask;
    let _: Option<f64> = q.price;
    let _: Option<f64> = q.change;
    let _: Option<f64> = q.change_percent;
    let _: Option<i64> = q.timestamp;
}

// ---------------------------------------------------------------------------
// Network tests (ignored by default)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access (ALPHAVANTAGE_API_KEY)"]
async fn test_forex_pair() {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};
    let providers = Providers::builder()
        .route(Capability::FOREX, &[Provider::AlphaVantage])
        .build()
        .await
        .unwrap();
    let pair = providers.forex("EUR", "USD");
    let _quote = pair.quote().await.unwrap();
    let chart = pair
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    assert!(!chart.candles.is_empty());
    let history = pair.history(TimeRange::OneMonth).await.unwrap();
    assert!(!history.candles.is_empty());
}

#[tokio::test]
#[ignore = "requires network access (ALPHAVANTAGE_API_KEY)"]
async fn test_forex_quote_fields() {
    use finance_query::{Capability, Provider, Providers};

    let providers = Providers::builder()
        .route(Capability::FOREX, &[Provider::AlphaVantage])
        .build()
        .await
        .unwrap();

    let pair = providers.forex("EUR", "USD");
    let quote = pair.quote().await.unwrap();

    println!("Symbol: {}", quote.symbol);
    if let Some(price) = quote.price {
        println!("Rate: {:.6}", price);
    }
    if let Some(bid) = quote.bid {
        println!("Bid:  {:.6}", bid);
    }
    if let Some(ask) = quote.ask {
        println!("Ask:  {:.6}", ask);
    }
    if let (Some(chg), Some(pct)) = (quote.change, quote.change_percent) {
        println!("Change: {:+.6} ({:+.4}%)", chg, pct);
    }
    assert!(!quote.symbol.is_empty());
}

#[tokio::test]
#[ignore = "requires network access (ALPHAVANTAGE_API_KEY)"]
async fn test_forex_chart() {
    use finance_query::{Capability, Interval, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::FOREX, &[Provider::AlphaVantage])
        .build()
        .await
        .unwrap();

    let pair = providers.forex("EUR", "USD");
    let chart = pair
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    println!("Pair: {}", chart.symbol);
    assert!(!chart.candles.is_empty());

    for candle in &chart.candles {
        println!(
            "{}: O={:.6}, H={:.6}, L={:.6}, C={:.6}",
            candle.timestamp, candle.open, candle.high, candle.low, candle.close
        );
    }
}

#[tokio::test]
#[ignore = "requires network access (ALPHAVANTAGE_API_KEY)"]
async fn test_forex_history() {
    use finance_query::{Capability, Provider, Providers, TimeRange};

    let providers = Providers::builder()
        .route(Capability::FOREX, &[Provider::AlphaVantage])
        .build()
        .await
        .unwrap();

    let pair = providers.forex("EUR", "USD");
    let history = pair.history(TimeRange::OneMonth).await.unwrap();

    assert!(!history.candles.is_empty());

    if let Some(last) = history.candles.last() {
        println!("Most recent close: {:.6}", last.close);
    }
}

#[tokio::test]
#[ignore = "requires network access (ALPHAVANTAGE_API_KEY)"]
async fn test_forex_caching() {
    use finance_query::{Capability, Provider, Providers};
    use std::time::Duration;

    let providers = Providers::builder()
        .route(Capability::FOREX, &[Provider::AlphaVantage])
        .build()
        .await
        .unwrap();

    let pair = providers.forex("EUR", "USD").cache(Duration::from_secs(60));

    // First call hits the network; subsequent calls within 60 s are cached.
    let _q1 = pair.quote().await.unwrap();
    let _q2 = pair.quote().await.unwrap(); // served from cache
}
