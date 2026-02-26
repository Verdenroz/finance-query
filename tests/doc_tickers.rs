//! Compile and runtime tests for docs/library/tickers.md
//!
//! Pure tests verify builder patterns, struct field access, and API shape.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_tickers`
//! Run network tests: `cargo test --test doc_tickers -- --ignored`

// ---------------------------------------------------------------------------
// Network tests — mirrors tickers.md code blocks
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_simple_construction() {
    use finance_query::Tickers;

    // From tickers.md "Simple Construction" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT", "GOOGL"]).await.unwrap();
    let response = tickers.quotes().await.unwrap();
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_builder_pattern() {
    use finance_query::{Region, Tickers};
    use std::time::Duration;

    // From tickers.md "Builder Pattern" section
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .region(Region::UnitedStates)
        .timeout(Duration::from_secs(30))
        .build()
        .await
        .unwrap();

    let response = tickers.quotes().await.unwrap();
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_max_concurrency() {
    use finance_query::Tickers;

    // From tickers.md "max_concurrency" section
    let tickers = Tickers::builder(vec!["AAPL", "MSFT", "GOOGL", "TSLA"])
        .max_concurrency(3)
        .build()
        .await
        .unwrap();

    let response = tickers.quotes().await.unwrap();
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_shared_session() {
    use finance_query::{Ticker, Tickers};

    // From tickers.md "Sharing a Session" section
    let aapl = Ticker::new("AAPL").await.unwrap();
    let handle = aapl.client_handle();

    // Reuses AAPL's authenticated session — no extra auth round-trip
    let tickers = Tickers::builder(["MSFT", "GOOGL"])
        .client(handle)
        .build()
        .await
        .unwrap();

    let response = tickers.quotes().await.unwrap();
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_quotes_with_logo() {
    use finance_query::Tickers;

    // From tickers.md "Batch Quotes" section
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .logo()
        .build()
        .await
        .unwrap();
    let response = tickers.quotes().await.unwrap();

    // Process successful quotes
    for (symbol, quote) in &response.quotes {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{} Price: ${:.2}", symbol, price);
        if let Some(logo) = &quote.logo_url {
            println!("  Logo: {}", logo);
        }
    }

    // Handle errors (prints any but doesn't fail)
    for (symbol, error) in &response.errors {
        eprintln!("Failed to fetch {}: {}", symbol, error);
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_quotes_utility_methods() {
    use finance_query::Tickers;

    // From tickers.md "Batch Response Utility Methods" section
    let tickers = Tickers::builder(vec!["AAPL", "GOOGL"])
        .logo()
        .build()
        .await
        .unwrap();
    let response = tickers.quotes().await.unwrap();

    println!("Successful: {}", response.success_count());
    println!("Failed:     {}", response.error_count());

    if !response.all_successful() {
        for (symbol, error) in &response.errors {
            eprintln!("Failed to fetch {}: {}", symbol, error);
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_charts() {
    use finance_query::{Interval, Tickers, TimeRange};

    // From tickers.md "Batch Charts" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch charts concurrently
    let response = tickers
        .charts(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Process successful charts
    for (symbol, chart) in &response.charts {
        println!("{}: {} candles", symbol, chart.candles.len());
        if let Some(last) = chart.candles.last() {
            println!("  Last Close: ${:.2}", last.close);
        }
    }

    for (symbol, error) in &response.errors {
        eprintln!("Failed to fetch chart for {}: {}", symbol, error);
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_spark() {
    use finance_query::{Interval, Tickers, TimeRange};

    // From tickers.md "Spark Data" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch spark data for all symbols
    let response = tickers
        .spark(Interval::OneDay, TimeRange::FiveDays)
        .await
        .unwrap();

    // Process successful sparks
    for (symbol, spark) in &response.sparks {
        println!("{}: {} data points", symbol, spark.len());

        if let Some(change) = spark.percent_change() {
            println!("  Change: {:+.2}%", change);
        }

        if let Some(min) = spark.min_close() {
            println!("  Low: ${:.2}", min);
        }

        if let Some(max) = spark.max_close() {
            println!("  High: ${:.2}", max);
        }
    }

    for (symbol, error) in &response.errors {
        eprintln!("Failed to fetch spark for {}: {}", symbol, error);
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_dividends() {
    use finance_query::{Tickers, TimeRange};

    // From tickers.md "Batch Dividends" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch dividends for all symbols
    let response = tickers.dividends(TimeRange::OneYear).await.unwrap();

    // Process successful dividends
    for (symbol, dividends) in &response.dividends {
        println!("{}: {} dividends", symbol, dividends.len());
        for div in dividends {
            println!("  Timestamp: {}, Amount: ${:.2}", div.timestamp, div.amount);
        }
    }

    for (symbol, error) in &response.errors {
        eprintln!("Failed to fetch dividends for {}: {}", symbol, error);
    }

    // AAPL and MSFT both pay dividends
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_splits() {
    use finance_query::{Tickers, TimeRange};

    // From tickers.md "Batch Splits" section
    let tickers = Tickers::new(vec!["NVDA", "TSLA", "AAPL"]).await.unwrap();
    let response = tickers.splits(TimeRange::FiveYears).await.unwrap();

    // Process splits
    for (symbol, splits) in &response.splits {
        if !splits.is_empty() {
            println!("{}: {} splits", symbol, splits.len());
            for split in splits {
                println!("  Timestamp: {}, Ratio: {}", split.timestamp, split.ratio);
            }
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_capital_gains() {
    use finance_query::{Tickers, TimeRange};

    // From tickers.md "Batch Capital Gains" section — ETFs
    let etfs = Tickers::new(vec!["SPY", "VOO", "VTI"]).await.unwrap();
    let response = etfs.capital_gains(TimeRange::TwoYears).await.unwrap();

    // Process capital gains
    for (symbol, gains) in &response.capital_gains {
        if !gains.is_empty() {
            println!("{}: {} capital gains distributions", symbol, gains.len());
            for gain in gains {
                println!(
                    "  Timestamp: {}, Amount: ${:.2}",
                    gain.timestamp, gain.amount
                );
            }
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_financials() {
    use finance_query::{Frequency, StatementType, Tickers};

    // From tickers.md "Batch Financials" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch quarterly income statements
    let response = tickers
        .financials(StatementType::Income, Frequency::Quarterly)
        .await
        .unwrap();

    // Process financial statements
    for (symbol, statement) in &response.financials {
        println!("{}: {} metrics", symbol, statement.statement.len());

        // Access specific metrics
        if let Some(revenue_data) = statement.statement.get("TotalRevenue") {
            println!("  Revenue data points: {}", revenue_data.len());

            // Get most recent revenue
            if let Some((date, value)) = revenue_data.iter().next() {
                println!("  Latest Revenue ({}): ${}", date, value);
            }
        }

        if let Some(income_data) = statement.statement.get("NetIncome")
            && let Some((date, value)) = income_data.iter().next()
        {
            println!("  Latest Net Income ({}): ${}", date, value);
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_news() {
    use finance_query::Tickers;

    // From tickers.md "Batch News" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch news for all symbols
    let response = tickers.news().await.unwrap();

    // Process news
    for (symbol, articles) in &response.news {
        println!("{}: {} news articles", symbol, articles.len());
        for article in articles.iter().take(3) {
            println!("  Title: {}", article.title);
            println!("  Source: {}", article.source);
            println!("  Link: {}", article.link);
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_recommendations() {
    use finance_query::Tickers;

    // From tickers.md "Batch Recommendations" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch recommendations with limit
    let response = tickers.recommendations(5).await.unwrap();

    // Process recommendations
    for (symbol, rec) in &response.recommendations {
        println!("{}: {} recommendations", symbol, rec.recommendations.len());
        for r in &rec.recommendations {
            println!("  {} ({})", r.symbol, r.score);
        }
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_options() {
    use finance_query::Tickers;

    // From tickers.md "Batch Options" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Fetch options for all symbols (nearest expiration)
    let response = tickers.options(None).await.unwrap();

    // Process options
    for (symbol, options) in &response.options {
        let exp_dates = options.expiration_dates();
        println!("{}: {} expirations", symbol, exp_dates.len());

        // Show calls and puts count for nearest expiration
        let calls = options.calls();
        let puts = options.puts();
        println!("  Calls: {} contracts", calls.len());
        println!("  Puts: {} contracts", puts.len());
    }

    // Fetch for a specific expiration date — use a dynamic date from the first response
    // so the call is valid (the doc uses a hardcoded Unix timestamp as an example)
    let first_exp = response
        .options
        .values()
        .next()
        .and_then(|opts| opts.expiration_dates().into_iter().nth(1));
    if let Some(specific_date) = first_exp {
        let _dated = tickers.options(Some(specific_date)).await.unwrap();
    }

    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_dynamic_symbol_management() {
    use finance_query::Tickers;

    // From tickers.md "Dynamic Symbol Management" section
    let mut tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();
    println!("Initial symbols: {:?}", tickers.symbols());

    // Add more symbols
    tickers.add_symbols(&["GOOGL", "TSLA", "NVDA"]);
    println!("After adding: {:?}", tickers.symbols());

    // Remove symbols (also clears their cached data)
    tickers.remove_symbols(&["MSFT", "TSLA"]).await;
    println!("After removing: {:?}", tickers.symbols());

    // Fetch quotes for current symbols
    let response = tickers.quotes().await.unwrap();
    // Response will only include AAPL, GOOGL, NVDA
    assert!(response.success_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_caching() {
    use finance_query::Tickers;
    use std::time::Duration;

    // From tickers.md "Caching" section
    let tickers = Tickers::builder(vec!["AAPL", "MSFT"])
        .cache(Duration::from_secs(30))
        .build()
        .await
        .unwrap();

    // First call: Network request, result cached for 30s
    let response1 = tickers.quotes().await.unwrap();

    // Second call within TTL: Returns cached data (no network request)
    let response2 = tickers.quotes().await.unwrap();

    // Clear all caches to force fresh data
    tickers.clear_cache().await;
    let response3 = tickers.quotes().await.unwrap(); // Network request

    // Or clear selectively:
    tickers.clear_quote_cache().await; // Quotes only
    tickers.clear_chart_cache().await; // Charts, sparks, and events

    assert_eq!(response1.success_count(), response2.success_count());
    assert_eq!(response2.success_count(), response3.success_count());
}

// ---------------------------------------------------------------------------
// Network tests — Individual Access from tickers.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_individual_access() {
    use finance_query::{Interval, Tickers, TimeRange};

    // From tickers.md "Individual Access" section
    let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

    // Get single quote (uses cache if available)
    let aapl = tickers.quote("AAPL").await.unwrap();
    println!(
        "AAPL price: {:?}",
        aapl.regular_market_price.as_ref().and_then(|v| v.raw)
    );

    // Get single chart (uses cache if available)
    let msft_chart = tickers
        .chart("MSFT", Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    println!("MSFT candles: {}", msft_chart.candles.len());

    assert!(!aapl.symbol.is_empty());
    assert!(!msft_chart.candles.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Best Practices from tickers.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_tickers_best_practices() {
    use finance_query::{Interval, Tickers, TimeRange};

    // From tickers.md "Best Practices" admonition — reuse instance, handle partial failures
    let tickers = Tickers::builder(vec!["AAPL", "GOOGL", "INVALID", "MSFT"])
        .logo()
        .build()
        .await
        .unwrap();

    // First operation - fetches data
    let quotes_response = tickers.quotes().await.unwrap();

    // Handle partial failures - check which symbols failed
    for (symbol, error) in &quotes_response.errors {
        println!("Failed to fetch {}: {}", symbol, error);
    }

    // Process successful results
    for (symbol, quote) in &quotes_response.quotes {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{}: ${:.2}", symbol, price);
    }

    // Second operation - uses cached data (no network request)
    let charts_response = tickers
        .charts(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    assert!(quotes_response.success_count() > 0);
    let _ = charts_response;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_batch_indicators() {
    #[cfg(feature = "indicators")]
    {
        use finance_query::{Interval, Tickers, TimeRange};

        // From tickers.md "Batch Indicators" section
        let tickers = Tickers::new(vec!["AAPL", "MSFT"]).await.unwrap();

        // Fetch indicators for all symbols
        let response = tickers
            .indicators(Interval::OneDay, TimeRange::OneMonth)
            .await
            .unwrap();

        // Process indicators
        for (symbol, indicators) in &response.indicators {
            println!("{} Indicators:", symbol);

            if let Some(rsi) = indicators.rsi_14 {
                println!("  RSI(14): {:.2}", rsi);
            }

            if let Some(sma) = indicators.sma_20 {
                println!("  SMA(20): {:.2}", sma);
            }

            if let Some(macd) = &indicators.macd
                && let Some(line) = macd.macd
            {
                println!("  MACD: {:.2}", line);
            }
        }

        assert!(response.success_count() > 0);
    }
}
