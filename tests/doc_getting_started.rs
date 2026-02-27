//! Compile and runtime tests for docs/library/getting-started.md
//!
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_getting_started`
//! Run network tests: `cargo test --test doc_getting_started -- --ignored`
//! With indicators: `cargo test --test doc_getting_started --features indicators -- --ignored`
//! With backtesting: `cargo test --test doc_getting_started --features backtesting -- --ignored`

// ---------------------------------------------------------------------------
// Network tests — Quick Example from getting-started.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_quick_example() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::builder("AAPL").logo().build().await.unwrap();

    // Get quote
    let quote = ticker.quote().await.unwrap();
    println!(
        "{}: ${:.2}",
        quote.symbol,
        quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0)
    );

    // Get chart
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    println!("Candles: {}", chart.candles.len());

    assert!(!quote.symbol.is_empty());
    assert!(!chart.candles.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Stock Data & Analysis from getting-started.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_stock_data_and_analysis() {
    use finance_query::Ticker;

    // Quotes, financials, options, news
    let ticker = Ticker::builder("MSFT").logo().build().await.unwrap();
    let quote = ticker.quote().await.unwrap(); // fetch quote with logo if available
    let financials = ticker.financial_data().await.unwrap();
    let options = ticker.options(None).await.unwrap();

    assert!(!quote.symbol.is_empty());
    let _ = financials;
    let _ = options;
}

// ---------------------------------------------------------------------------
// Network tests — Batch Operations from getting-started.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_operations() {
    use finance_query::{Interval, Tickers, TimeRange};

    // Fetch multiple symbols efficiently
    let tickers = Tickers::builder(vec!["AAPL", "MSFT", "GOOGL"])
        .logo()
        .build()
        .await
        .unwrap();
    let quotes = tickers.quotes().await.unwrap(); // fetch quotes with logos if available
    let sparks = tickers
        .spark(Interval::OneDay, TimeRange::FiveDays)
        .await
        .unwrap();

    assert!(quotes.success_count() > 0);
    assert!(sparks.success_count() > 0);
}

// ---------------------------------------------------------------------------
// Network tests — Market Discovery from getting-started.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_market_discovery() {
    use finance_query::{Screener, SearchOptions, finance};

    // Search, screeners, trending stocks
    let results = finance::search("Tesla", &SearchOptions::default())
        .await
        .unwrap();
    let actives = finance::screener(Screener::MostActives, 25).await.unwrap();
    let trending = finance::trending(None).await.unwrap();

    let _ = results;
    let _ = actives;
    assert!(!trending.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Technical Indicators from getting-started.md
// ---------------------------------------------------------------------------

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_indicators() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();

    // 52+ indicators: RSI, MACD, Bollinger Bands, etc.
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    if let Some(rsi) = indicators.rsi_14 {
        println!("RSI: {:.2}", rsi);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Backtesting from getting-started.md
// ---------------------------------------------------------------------------

#[cfg(feature = "backtesting")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_backtesting() {
    use finance_query::backtesting::SmaCrossover;
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();

    // Test strategies against historical data
    let result = ticker
        .backtest(
            SmaCrossover::new(10, 20),
            Interval::OneDay,
            TimeRange::OneYear,
            None,
        )
        .await
        .unwrap();

    println!("Return: {:.2}%", result.metrics.total_return_pct);
}

// ---------------------------------------------------------------------------
// Real-time Streaming — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies PriceUpdate fields used in the getting-started.md streaming example.
/// Mirrors: `price.id`, `price.price`, `price.change_percent`
#[allow(dead_code)]
fn _verify_price_update_fields(price: finance_query::streaming::PriceUpdate) {
    // From getting-started.md streaming code block
    let _: String = price.id;
    let _: f32 = price.price;
    let _: f32 = price.change_percent;
}

// ---------------------------------------------------------------------------
// Network tests — Real-time Streaming from getting-started.md
// (connect only; don't consume the stream in tests to avoid hanging)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_streaming_connects() {
    use finance_query::streaming::PriceStream;

    // Subscribe to real-time price updates via WebSocket
    let stream = PriceStream::subscribe(&["AAPL", "NVDA", "TSLA"])
        .await
        .unwrap();

    // Verify the stream was created successfully (don't consume — would block indefinitely)
    let _ = stream;
}

// ---------------------------------------------------------------------------
// Network tests — SEC EDGAR Filings from getting-started.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_edgar_filings() {
    use finance_query::edgar;

    // Init once per process (SEC requires contact email)
    edgar::init("user@example.com").unwrap();

    // Resolve ticker to CIK number
    let cik = edgar::resolve_cik("AAPL").await.unwrap(); // 320193
    assert_eq!(cik, 320193);

    // Fetch all SEC filings metadata
    let submissions = edgar::submissions(cik).await.unwrap();
    if let Some(recent) = submissions.filings.as_ref().and_then(|f| f.recent.as_ref()) {
        println!("Recent filings: {}", recent.form.len());
    }

    // Fetch structured XBRL financial data
    let facts = edgar::company_facts(cik).await.unwrap();
    let _ = facts;
}

// ---------------------------------------------------------------------------
// Network tests — DataFrame Support from getting-started.md
// ---------------------------------------------------------------------------

#[cfg(feature = "dataframe")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_dataframe_support() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From getting-started.md "DataFrame Support" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let df = chart.to_dataframe().unwrap();
    assert!(df.height() > 0, "DataFrame should have rows");
    println!("DataFrame rows: {}", df.height());
}

// ---------------------------------------------------------------------------
// Network tests — Risk Analytics from getting-started.md
// ---------------------------------------------------------------------------

#[cfg(feature = "risk")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_risk_analytics() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();

    // VaR, Sharpe/Sortino/Calmar ratio, Beta, max drawdown
    let summary = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, Some("SPY"))
        .await
        .unwrap();

    println!("VaR 95%:      {:.2}%", summary.var_95 * 100.0);
    println!("Sharpe:       {:.2}", summary.sharpe.unwrap_or(0.0));
    println!("Max Drawdown: {:.2}%", summary.max_drawdown * 100.0);
    println!("Beta vs SPY:  {:.2}", summary.beta.unwrap_or(0.0));
}
