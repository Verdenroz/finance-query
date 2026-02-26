//! Compile and runtime tests for docs/library/streaming.md
//!
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_streaming`
//! Run network tests: `cargo test --test doc_streaming -- --ignored`

// ---------------------------------------------------------------------------
// Compile-time — PriceUpdate field access documented in streaming.md
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn _verify_price_update_fields(p: finance_query::streaming::PriceUpdate) {
    let _: String = p.id;
    let _: f32 = p.price;
    let _: f32 = p.change;
    let _: f32 = p.change_percent;
    let _: f32 = p.day_high;
    let _: f32 = p.day_low;
    let _: i64 = p.day_volume;
    let _: f32 = p.open_price;
    let _: f32 = p.previous_close;
    let _: String = p.short_name;
    let _: String = p.currency;
    let _: String = p.exchange;
    let _: finance_query::streaming::QuoteType = p.quote_type;
    let _: finance_query::streaming::MarketHoursType = p.market_hours;
    let _: i64 = p.time;
}

// ---------------------------------------------------------------------------
// Compile-time — MarketHoursType variants documented in streaming.md
// ---------------------------------------------------------------------------

#[test]
fn test_market_hours_type_variants_compile() {
    use finance_query::streaming::MarketHoursType;

    let _ = MarketHoursType::PreMarket;
    let _ = MarketHoursType::RegularMarket;
    let _ = MarketHoursType::PostMarket;
    let _ = MarketHoursType::ExtendedHoursMarket;
}

// ---------------------------------------------------------------------------
// Compile-time — QuoteType variants documented in streaming.md
// ---------------------------------------------------------------------------

#[test]
fn test_quote_type_variants_compile() {
    use finance_query::streaming::QuoteType;

    let _ = QuoteType::Equity;
    let _ = QuoteType::Etf;
    let _ = QuoteType::Cryptocurrency;
    let _ = QuoteType::Index;
    let _ = QuoteType::MutualFund;
    let _ = QuoteType::Currency;
    let _ = QuoteType::Future;
}

// ---------------------------------------------------------------------------
// Network tests — mirrors streaming.md code blocks
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_simple_subscribe() {
    use finance_query::streaming::PriceStream;

    // From streaming.md "Simple Subscribe" section
    let stream = PriceStream::subscribe(&["AAPL", "GOOGL"]).await.unwrap();
    let _ = stream;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_builder_pattern() {
    use finance_query::streaming::PriceStreamBuilder;
    use std::time::Duration;

    // From streaming.md "Builder Pattern" section
    let stream = PriceStreamBuilder::new()
        .symbols(&["AAPL", "MSFT", "NVDA"])
        .reconnect_delay(Duration::from_secs(5))
        .build()
        .await
        .unwrap();

    let _ = stream;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_dynamic_subscriptions() {
    use finance_query::streaming::PriceStream;

    // From streaming.md "Dynamic Subscriptions" section
    let stream = PriceStream::subscribe(&["AAPL"]).await.unwrap();

    // Add more symbols
    stream.add_symbols(&["NVDA", "TSLA"]).await;

    // Remove symbols
    stream.remove_symbols(&["AAPL"]).await;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_resubscribe() {
    use finance_query::streaming::PriceStream;

    // From streaming.md "Multiple Consumers" section
    let stream1 = PriceStream::subscribe(&["AAPL", "NVDA"]).await.unwrap();
    let stream2 = stream1.resubscribe();

    // Both handles share the same WebSocket connection
    let _ = stream1;
    let _ = stream2;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_close() {
    use finance_query::streaming::PriceStream;

    // From streaming.md "Closing the Stream" section
    let stream = PriceStream::subscribe(&["AAPL"]).await.unwrap();
    stream.close().await;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_quick_start() {
    use finance_query::streaming::PriceStream;
    use futures::StreamExt;
    use tokio::time::{Duration, timeout};

    // From streaming.md "Quick Start" section — full pattern with stream.next()
    let mut stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"])
        .await
        .unwrap();

    // Try to receive one update (may time out if market is closed)
    if let Ok(Some(price)) = timeout(Duration::from_secs(3), stream.next()).await {
        println!(
            "{}: ${:.2} ({:+.2}%)",
            price.id, price.price, price.change_percent
        );
    }

    stream.close().await;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_filtering_by_market_hours() {
    use finance_query::streaming::{MarketHoursType, PriceStream};
    use futures::StreamExt;
    use tokio::time::{Duration, timeout};

    // From streaming.md "Filtering Updates" section
    let mut stream = PriceStream::subscribe(&["AAPL", "MSFT", "GOOGL"])
        .await
        .unwrap();

    // Try to receive one update (may time out if market is closed)
    if let Ok(Some(price)) = timeout(Duration::from_secs(3), stream.next()).await
        && price.market_hours == MarketHoursType::RegularMarket
    {
        println!("{}: ${:.2}", price.id, price.price);
    }

    stream.close().await;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_multiple_consumers_with_spawn() {
    use finance_query::streaming::PriceStream;
    use futures::StreamExt;
    use tokio::time::{Duration, timeout};

    // From streaming.md "Multiple Consumers" section — exact tokio::spawn pattern
    let mut stream1 = PriceStream::subscribe(&["AAPL", "NVDA"]).await.unwrap();
    let mut stream2 = stream1.resubscribe();

    // Both streams receive the same updates
    tokio::spawn(async move {
        if let Ok(Some(price)) = timeout(Duration::from_secs(2), stream2.next()).await {
            println!("Consumer 2: {} ${:.2}", price.id, price.price);
        }
    });

    if let Ok(Some(price)) = timeout(Duration::from_secs(3), stream1.next()).await {
        println!("Consumer 1: {} ${:.2}", price.id, price.price);
    }

    stream1.close().await;
}
