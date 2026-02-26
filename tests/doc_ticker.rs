//! Compile and runtime tests for docs/library/ticker.md
//!
//! Pure tests verify struct field names and types documented in ticker.md.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_ticker`
//! Run network tests: `cargo test --test doc_ticker -- --ignored`

use finance_query::{Dividend, DividendAnalytics};

// ---------------------------------------------------------------------------
// DividendAnalytics — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all DividendAnalytics fields documented in ticker.md exist with
/// the correct types. This function is never called; it exists only to be
/// type-checked by the compiler.
#[allow(dead_code)]
fn _verify_dividend_analytics_fields(a: DividendAnalytics) {
    let _: f64 = a.total_paid;
    let _: usize = a.payment_count;
    let _: f64 = a.average_payment;
    let _: Option<f64> = a.cagr;
    let _: Option<Dividend> = a.last_payment;
    let _: Option<Dividend> = a.first_payment;
}

/// Verifies Dividend struct fields (used in DividendAnalytics).
#[allow(dead_code)]
fn _verify_dividend_fields(d: Dividend) {
    let _: i64 = d.timestamp;
    let _: f64 = d.amount;
}

/// Verifies Chart struct fields from the `Chart Structure` block in ticker.md.
/// Checks that `interval` and `range` exist with the documented types.
/// `symbol`, `meta`, `candles` are exercised at runtime in `test_ticker_chart_meta_fields`.
#[allow(dead_code)]
fn _verify_chart_struct_fields(chart: finance_query::Chart) {
    let _: String = chart.symbol;
    let _: Vec<finance_query::Candle> = chart.candles;
    let _: Option<finance_query::Interval> = chart.interval;
    let _: Option<finance_query::TimeRange> = chart.range;
}

/// Compile-time verification that the manual builder methods from ticker.md exist.
/// `.proxy()` is only checked for compilation — using a fake URL at runtime would fail.
#[allow(dead_code)]
fn _verify_builder_manual_methods() {
    use std::time::Duration;
    let _b = finance_query::Ticker::builder("AAPL")
        .lang("en-US")
        .region_code("US")
        .timeout(Duration::from_secs(20))
        .proxy("http://proxy.example.com:8080");
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_quote() {
    use finance_query::Ticker;

    let ticker = Ticker::new("AAPL").await.unwrap();
    let quote = ticker.quote().await.unwrap();

    assert_eq!(quote.symbol, "AAPL");
    let price = quote
        .regular_market_price
        .as_ref()
        .and_then(|v| v.raw)
        .unwrap_or(0.0);
    assert!(price > 0.0, "price should be positive");
    println!("AAPL price: ${:.2}", price);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_dividend_analytics() {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();
    let analytics = ticker
        .dividend_analytics(TimeRange::FiveYears)
        .await
        .unwrap();

    println!("Total paid:      ${:.2}", analytics.total_paid);
    println!("Payments:        {}", analytics.payment_count);
    println!("Average payment: ${:.4}", analytics.average_payment);
    if let Some(cagr) = analytics.cagr {
        println!("CAGR:            {:.1}%", cagr * 100.0);
    }
    if let Some(last) = &analytics.last_payment {
        println!(
            "Most recent:     ${:.4} at timestamp {}",
            last.amount, last.timestamp
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_chart_candles() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("MSFT").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    assert!(!chart.candles.is_empty());
    for candle in chart.candles.iter().take(3) {
        let _: i64 = candle.timestamp;
        let _: f64 = candle.open;
        let _: f64 = candle.high;
        let _: f64 = candle.low;
        let _: f64 = candle.close;
        let _: i64 = candle.volume;
        let _: Option<f64> = candle.adj_close;
        println!(
            "{}: O={:.2} H={:.2} L={:.2} C={:.2} V={}",
            candle.timestamp, candle.open, candle.high, candle.low, candle.close, candle.volume
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_dividends() {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();
    let dividends = ticker.dividends(TimeRange::TwoYears).await.unwrap();

    for div in &dividends {
        println!("timestamp={}, amount=${:.4}", div.timestamp, div.amount);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_splits() {
    use finance_query::{Ticker, TimeRange};

    let ticker = Ticker::new("NVDA").await.unwrap();
    let splits = ticker.splits(TimeRange::Max).await.unwrap();

    for split in &splits {
        println!(
            "timestamp={}, ratio={} ({}/{})",
            split.timestamp, split.ratio, split.numerator, split.denominator
        );
    }
}

#[cfg(feature = "risk")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_risk() {
    use finance_query::{Interval, Ticker, TimeRange};

    let ticker = Ticker::new("AAPL").await.unwrap();
    let risk = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, Some("^GSPC"))
        .await
        .unwrap();

    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    if let Some(sharpe) = risk.sharpe {
        println!("Sharpe:       {:.2}", sharpe);
    }
    if let Some(beta) = risk.beta {
        println!("Beta:         {:.2}", beta);
    }
    assert!(risk.var_95 >= 0.0);
    assert!(risk.max_drawdown >= 0.0);
}

#[cfg(feature = "risk")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_risk_no_benchmark() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Risk Analytics" — without a benchmark
    let ticker = Ticker::new("AAPL").await.unwrap();
    let risk = ticker
        .risk(Interval::OneDay, TimeRange::OneYear, None)
        .await
        .unwrap();

    println!("VaR 95%:      {:.2}%", risk.var_95 * 100.0);
    println!("Max Drawdown: {:.2}%", risk.max_drawdown * 100.0);
    if let Some(sharpe) = risk.sharpe {
        println!("Sharpe:       {:.2}", sharpe);
    }
    assert!(
        risk.beta.is_none(),
        "beta should be None without a benchmark"
    );
    assert!(risk.var_95 >= 0.0);
    assert!(risk.max_drawdown >= 0.0);
}

// ---------------------------------------------------------------------------
// Network tests — Builder Pattern from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_with_region() {
    use finance_query::{Region, Ticker};
    use std::time::Duration;

    // From ticker.md "Builder Pattern" — region + timeout
    let ticker = Ticker::builder("2330.TW")
        .region(Region::Taiwan)
        .timeout(Duration::from_secs(30))
        .build()
        .await
        .unwrap();
    let _ = ticker;
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_with_logo_and_cache() {
    use finance_query::Ticker;
    use std::time::Duration;

    // From ticker.md "Builder Pattern" — logo + cache
    let ticker = Ticker::builder("AAPL")
        .logo()
        .cache(Duration::from_secs(300))
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    println!("Logo: {:?}", quote.logo_url);
    println!("Company Logo: {:?}", quote.company_logo_url);
    assert_eq!(quote.symbol, "AAPL");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_builder_manual_config() {
    use finance_query::Ticker;
    use std::time::Duration;

    // From ticker.md "Builder Pattern" — manual lang/region_code/timeout
    // proxy is verified at compile time only (_verify_builder_manual_methods)
    let ticker = Ticker::builder("AAPL")
        .lang("en-US")
        .region_code("US")
        .timeout(Duration::from_secs(20))
        .build()
        .await
        .unwrap();

    let quote = ticker.quote().await.unwrap();
    assert_eq!(quote.symbol, "AAPL");
}

// ---------------------------------------------------------------------------
// Network tests — Aggregated Quote from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_quote_full_fields() {
    use finance_query::Ticker;

    // From ticker.md "Aggregated Quote" section — full field access pattern
    let ticker = Ticker::builder("AAPL").logo().build().await.unwrap();
    let quote = ticker.quote().await.unwrap();

    println!("Symbol: {}", quote.symbol);
    println!("Name: {}", quote.short_name.as_deref().unwrap_or("N/A"));
    let price = quote
        .regular_market_price
        .as_ref()
        .and_then(|v| v.raw)
        .unwrap_or(0.0);
    println!("Price: ${:.2}", price);
    let change = quote
        .regular_market_change
        .as_ref()
        .and_then(|v| v.raw)
        .unwrap_or(0.0);
    let change_pct = quote
        .regular_market_change_percent
        .as_ref()
        .and_then(|v| v.raw)
        .unwrap_or(0.0);
    println!("Change: {:+.2} ({:+.2}%)", change, change_pct);
    let market_cap = quote.market_cap.as_ref().and_then(|v| v.raw).unwrap_or(0);
    println!("Market Cap: ${}", market_cap);
    println!("Logo: {:?}", quote.logo_url);
    println!("Company Logo: {:?}", quote.company_logo_url);

    assert!(!quote.symbol.is_empty());
    assert!(price > 0.0);
}

// ---------------------------------------------------------------------------
// Network tests — Quote Modules from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_quote_modules() {
    use finance_query::Ticker;

    // From ticker.md "Quote Modules" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // First access triggers ONE API call for ALL modules
    let price = ticker.price().await.unwrap();
    if let Some(p) = price {
        println!(
            "Market State: {}",
            p.market_state.as_deref().unwrap_or("N/A")
        );
        println!("Currency: {}", p.currency.as_deref().unwrap_or("N/A"));
    }

    // Subsequent calls use cached data (no network request)
    let financial_data = ticker.financial_data().await.unwrap();
    if let Some(fd) = financial_data {
        let revenue = fd.total_revenue.as_ref().and_then(|v| v.raw).unwrap_or(0);
        println!("Revenue: ${}", revenue);
        let profit_margins = fd
            .profit_margins
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("Profit Margin: {:.2}%", profit_margins * 100.0);
    }

    // Get EPS from DefaultKeyStatistics
    if let Some(stats) = ticker.key_stats().await.unwrap() {
        let eps = stats
            .trailing_eps
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("EPS: ${:.2}", eps);
    }

    let profile = ticker.asset_profile().await.unwrap();
    if let Some(prof) = profile {
        println!("Sector: {}", prof.sector.as_deref().unwrap_or("N/A"));
        println!("Industry: {}", prof.industry.as_deref().unwrap_or("N/A"));
        println!("Website: {}", prof.website.as_deref().unwrap_or("N/A"));
        println!(
            "Description: {}",
            prof.long_business_summary.as_deref().unwrap_or("N/A")
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — Company Analysis example from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_company_analysis() {
    use finance_query::Ticker;

    // From ticker.md "Example: Company Analysis" section
    let ticker = Ticker::new("MSFT").await.unwrap();

    if let Some(fd) = ticker.financial_data().await.unwrap() {
        println!("Financials:");
        let revenue = fd.total_revenue.as_ref().and_then(|v| v.raw).unwrap_or(0) as f64;
        println!("  Revenue: ${:.2}B", revenue / 1e9);
        let profit_margins = fd
            .profit_margins
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  Profit Margin: {:.2}%", profit_margins * 100.0);
        let roe = fd
            .return_on_equity
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  ROE: {:.2}%", roe * 100.0);
        let dte = fd
            .debt_to_equity
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("  Debt to Equity: {:.2}", dte);
    }

    if let Some(sd) = ticker.summary_detail().await.unwrap() {
        println!("\nValuation:");
        let trailing_pe = sd.trailing_pe.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
        println!("  P/E Ratio: {:.2}", trailing_pe);
        let forward_pe = sd.forward_pe.as_ref().and_then(|v| v.raw).unwrap_or(0.0);
        println!("  Forward P/E: {:.2}", forward_pe);
    }

    if let Some(rt) = ticker.recommendation_trend().await.unwrap()
        && let Some(latest) = rt.trend.first()
    {
        println!("\nAnalyst Recommendations:");
        println!("  Strong Buy: {}", latest.strong_buy.unwrap_or(0));
        println!("  Buy: {}", latest.buy.unwrap_or(0));
        println!("  Hold: {}", latest.hold.unwrap_or(0));
        println!("  Sell: {}", latest.sell.unwrap_or(0));
        println!("  Strong Sell: {}", latest.strong_sell.unwrap_or(0));
    }
}

// ---------------------------------------------------------------------------
// Network tests — Chart meta fields from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_chart_meta_fields() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Chart (OHLCV) Data" section — meta field access
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    println!("Symbol: {}", chart.symbol);
    println!(
        "Currency: {}",
        chart.meta.currency.as_deref().unwrap_or("N/A")
    );
    println!(
        "Exchange: {}",
        chart.meta.exchange_name.as_deref().unwrap_or("N/A")
    );
    println!(
        "Timezone: {}",
        chart.meta.timezone.as_deref().unwrap_or("N/A")
    );

    assert!(!chart.symbol.is_empty());
    assert!(!chart.candles.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Capital Gains from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_capital_gains() {
    use finance_query::{Ticker, TimeRange};

    // From ticker.md "Capital Gains" section
    let ticker = Ticker::new("SPY").await.unwrap();
    let gains = ticker.capital_gains(TimeRange::FiveYears).await.unwrap();

    for gain in &gains {
        println!(
            "timestamp={}, amount=${:.4} per share",
            gain.timestamp, gain.amount
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — Technical Indicators from ticker.md (indicators feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_indicators_summary() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Technical Indicators — Summary API" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let indicators = ticker
        .indicators(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    if let Some(rsi) = indicators.rsi_14 {
        println!("RSI(14): {:.2}", rsi);
        if rsi < 30.0 {
            println!("  -> Oversold");
        } else if rsi > 70.0 {
            println!("  -> Overbought");
        }
    }

    if let Some(sma) = indicators.sma_200 {
        println!("SMA(200): {:.2}", sma);
    }

    if let Some(macd) = &indicators.macd
        && let (Some(line), Some(signal)) = (macd.macd, macd.signal)
    {
        println!("MACD: {:.4} | Signal: {:.4}", line, signal);
        if line > signal {
            println!("  -> Bullish");
        }
    }

    if let Some(bb) = &indicators.bollinger_bands
        && let (Some(upper), Some(lower)) = (bb.upper, bb.lower)
    {
        println!("Bollinger: Upper={:.2}, Lower={:.2}", upper, lower);
    }
}

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_chart_extension_methods() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Chart Extension Methods" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    let sma_15 = chart.sma(15);
    let rsi_21 = chart.rsi(21).unwrap();
    let macd = chart.macd(8, 21, 5).unwrap(); // Fast, slow, signal

    if let Some(&latest_rsi) = rsi_21.last().and_then(|v| v.as_ref()) {
        println!("RSI(21): {:.2}", latest_rsi);
    }
    let _ = sma_15;
    let _ = macd;
}

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_direct_indicator_functions() {
    use finance_query::indicators::{rsi, sma};
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Direct Functions" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();
    let closes: Vec<f64> = chart.candles.iter().map(|c| c.close).collect();

    let sma_25 = sma(&closes, 25);
    let rsi_10 = rsi(&closes, 10).unwrap();

    if let Some(&latest) = rsi_10.last().and_then(|v| v.as_ref()) {
        println!("RSI(10): {:.2}", latest);
    }
    let _ = sma_25;
}

#[cfg(feature = "indicators")]
#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_candlestick_patterns() {
    use finance_query::indicators::PatternSentiment;
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Candlestick Patterns" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let chart = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    // Returns Vec<Option<CandlePattern>>, 1:1 aligned with chart.candles
    let signals = chart.patterns();

    // Zip patterns with candles for context
    for (candle, pattern) in chart.candles.iter().zip(signals.iter()) {
        if let Some(p) = pattern {
            println!(
                "timestamp={}: {:?} ({:?})",
                candle.timestamp,
                p,
                p.sentiment()
            );
        }
    }

    // Count bullish signals in the period
    let bullish_count = signals
        .iter()
        .filter(|s| {
            s.map(|p| p.sentiment() == PatternSentiment::Bullish)
                .unwrap_or(false)
        })
        .count();
    println!("{bullish_count} bullish patterns detected");
}

// ---------------------------------------------------------------------------
// Network tests — Recommendations from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_recommendations() {
    use finance_query::Ticker;

    // From ticker.md "Recommendations" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let rec = ticker.recommendations(5).await.unwrap();

    println!("Similar stocks to {}:", ticker.symbol());
    for similar in &rec.recommendations {
        println!("  {} - {}", similar.symbol, similar.score);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Financial Statements from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_financials() {
    use finance_query::{Frequency, StatementType, Ticker};

    // From ticker.md "Financial Statements" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Annual income statement
    let income = ticker
        .financials(StatementType::Income, Frequency::Annual)
        .await
        .unwrap();

    if let Some(revenue_map) = income.statement.get("TotalRevenue") {
        for (date, value) in revenue_map {
            println!("{}: Revenue ${:.2}B", date, value / 1e9);
        }
    }

    if let Some(net_income_map) = income.statement.get("NetIncome") {
        for (date, value) in net_income_map {
            println!("{}: Net Income ${:.2}B", date, value / 1e9);
        }
    }

    // Quarterly balance sheet and cashflow
    let balance = ticker
        .financials(StatementType::Balance, Frequency::Quarterly)
        .await
        .unwrap();
    let cashflow = ticker
        .financials(StatementType::CashFlow, Frequency::Annual)
        .await
        .unwrap();
    let _ = balance;
    let _ = cashflow;
}

// ---------------------------------------------------------------------------
// Network tests — Options Data from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_options() {
    use finance_query::Ticker;

    // From ticker.md "Options Data" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let options = ticker.options(None).await.unwrap();

    println!("Available expiration dates:");
    for exp in options.expiration_dates() {
        println!("  {}", exp); // Unix timestamp (i64)
    }

    // calls() and puts() return a Contracts collection
    println!("\nCalls:");
    for call in &*options.calls() {
        println!(
            "  Strike ${:.2}: last=${:.2}, volume={}",
            call.strike,
            call.last_price.unwrap_or(0.0),
            call.volume.unwrap_or(0),
        );
    }

    println!("\nPuts:");
    for put in &*options.puts() {
        println!(
            "  Strike ${:.2}: last=${:.2}, IV={:.4}",
            put.strike,
            put.last_price.unwrap_or(0.0),
            put.implied_volatility.unwrap_or(0.0),
        );
    }

    let exp_dates = options.expiration_dates();
    if exp_dates.len() > 1 {
        let _options_dated = ticker.options(Some(exp_dates[1])).await.unwrap();
    }
}

// ---------------------------------------------------------------------------
// Network tests — News from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_news() {
    use finance_query::Ticker;

    // From ticker.md "News" section
    let ticker = Ticker::new("AAPL").await.unwrap();
    let news = ticker.news().await.unwrap();

    for article in news.iter().take(3) {
        println!("{}", article.title);
        println!("  Source: {}", article.source);
        println!("  Published: {}", article.time);
        println!("  URL: {}", article.link);
        println!();
    }

    assert!(!news.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Earnings Transcripts from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_earnings_transcripts() {
    use finance_query::Ticker;
    use finance_query::finance;

    // From ticker.md "Earnings Transcripts" section
    let ticker = Ticker::new("AAPL").await.unwrap();

    // Get latest transcript
    let transcript = finance::earnings_transcript(ticker.symbol(), None, None)
        .await
        .unwrap();

    println!(
        "Transcript for {} - Q{} {}",
        ticker.symbol(),
        transcript.quarter(),
        transcript.year()
    );

    // Access paragraphs with speaker names resolved
    for (paragraph, speaker) in transcript.paragraphs_with_speakers() {
        if let Some(name) = speaker {
            println!("[{:.1}s] {}: {}", paragraph.start, name, paragraph.text);
        }
    }

    // Get a specific quarter transcript
    let _q1 = finance::earnings_transcript(ticker.symbol(), Some("Q1"), Some(2024))
        .await
        .unwrap();

    // Get all available transcripts (metadata only)
    let all_transcripts = finance::earnings_transcripts(ticker.symbol(), None)
        .await
        .unwrap();

    for meta in all_transcripts.iter().take(5) {
        println!(
            "{} {} - {}",
            meta.year.unwrap_or(0),
            meta.quarter.as_deref().unwrap_or("?"),
            meta.title
        );
    }

    assert!(!all_transcripts.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Chart caching from ticker.md
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_ticker_chart_caching() {
    use finance_query::{Interval, Ticker, TimeRange};

    // From ticker.md "Chart Data" caching section — same+different (interval, range) combos
    let ticker = Ticker::new("AAPL").await.unwrap();

    // First call -> 1 API call
    let daily_1mo = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Same interval+range -> cached (0 API calls)
    let daily_1mo_again = ticker
        .chart(Interval::OneDay, TimeRange::OneMonth)
        .await
        .unwrap();

    // Different interval or range -> new API call
    let hourly_1mo = ticker
        .chart(Interval::OneHour, TimeRange::OneMonth)
        .await
        .unwrap();
    let daily_3mo = ticker
        .chart(Interval::OneDay, TimeRange::ThreeMonths)
        .await
        .unwrap();

    assert!(!daily_1mo.candles.is_empty());
    assert_eq!(daily_1mo.symbol, daily_1mo_again.symbol);
    let _ = hourly_1mo;
    let _ = daily_3mo;
}
