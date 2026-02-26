//! Compile and runtime tests for docs/library/finance.md
//!
//! Pure tests verify type/variant existence and field access patterns.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_finance`
//! Run network tests: `cargo test --test doc_finance -- --ignored`

use finance_query::{FearAndGreed, FearGreedLabel, Sector};

// ---------------------------------------------------------------------------
// FearAndGreed — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies FearAndGreed struct fields documented in finance.md.
#[allow(dead_code)]
fn _verify_fear_and_greed_fields(fg: FearAndGreed) {
    let _: u8 = fg.value;
    let _: FearGreedLabel = fg.classification;
    let _: i64 = fg.timestamp;
}

#[test]
fn test_fear_greed_label_variants_and_as_str() {
    // All variants documented in finance.md must exist and have non-empty as_str().
    let cases = [
        (FearGreedLabel::ExtremeFear, "Extreme Fear"),
        (FearGreedLabel::Fear, "Fear"),
        (FearGreedLabel::Neutral, "Neutral"),
        (FearGreedLabel::Greed, "Greed"),
        (FearGreedLabel::ExtremeGreed, "Extreme Greed"),
    ];
    for (variant, expected) in cases {
        assert_eq!(variant.as_str(), expected);
    }
}

// ---------------------------------------------------------------------------
// Sector enum — compile-time variant verification (all 11 in finance.md)
// ---------------------------------------------------------------------------

#[test]
fn test_sector_variants_compile() {
    // Mirrors the sector() example in finance.md.
    let _ = Sector::Technology;
    let _ = Sector::Healthcare;
    let _ = Sector::FinancialServices;
    let _ = Sector::BasicMaterials;
    let _ = Sector::CommunicationServices;
    let _ = Sector::ConsumerCyclical;
    let _ = Sector::ConsumerDefensive;
    let _ = Sector::Energy;
    let _ = Sector::Industrials;
    let _ = Sector::RealEstate;
    let _ = Sector::Utilities;
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fear_and_greed() {
    use finance_query::finance;

    let fg = finance::fear_and_greed().await.unwrap();
    println!(
        "Fear & Greed: {} / 100 ({})",
        fg.value,
        fg.classification.as_str()
    );
    assert!(fg.timestamp > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_sector_technology() {
    use finance_query::finance;

    let tech = finance::sector(Sector::Technology).await.unwrap();
    println!("Sector: {}", tech.name);
    assert!(!tech.name.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_industry_semiconductors() {
    use finance_query::finance;

    let semi = finance::industry("semiconductors").await.unwrap();
    println!("Industry: {}", semi.name);
    assert!(!semi.name.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search() {
    use finance_query::{SearchOptions, finance};

    let results = finance::search("Apple", &SearchOptions::default())
        .await
        .unwrap();
    println!("Found {} quotes", results.result_count());
    assert!(results.result_count() > 0);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_market_summary() {
    use finance_query::finance;

    let summary = finance::market_summary(None).await.unwrap();
    assert!(!summary.is_empty());
    for quote in &summary {
        let price = quote
            .regular_market_price
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{}: {:.2}", quote.symbol, price);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_trending() {
    use finance_query::finance;

    let trending = finance::trending(None).await.unwrap();
    assert!(!trending.is_empty());
    for quote in &trending {
        println!("{}", quote.symbol);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Search (advanced options, from finance.md "Search" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_search_advanced_options() {
    use finance_query::{Region, SearchOptions, finance};

    // From finance.md "Search" section — advanced search with options
    let options = SearchOptions::new()
        .quotes_count(10)
        .news_count(5)
        .enable_research_reports(true)
        .enable_fuzzy_query(true)
        .region(Region::Canada);

    let results = finance::search("tesla", &options).await.unwrap();
    println!("Found {} results", results.result_count());

    for quote in &results.quotes {
        let exchange = quote.exchange.as_deref().unwrap_or("N/A");
        let name = quote.short_name.as_deref().unwrap_or("N/A");
        println!("{} ({}): {}", quote.symbol, exchange, name);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Lookup (from finance.md "Lookup" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_lookup() {
    use finance_query::{LookupOptions, LookupType, finance};

    // From finance.md "Lookup" section
    // Simple lookup
    let results = finance::lookup("NVDA", &LookupOptions::default())
        .await
        .unwrap();
    assert!(!results.quotes.is_empty());

    // Lookup equities with logos
    let options = LookupOptions::new()
        .lookup_type(LookupType::Equity)
        .count(10)
        .include_logo(true);

    let results = finance::lookup("tech", &options).await.unwrap();
    for quote in &results.quotes {
        let name = quote.short_name.as_deref().unwrap_or("N/A");
        println!("{}: {} - {:?}", quote.symbol, name, quote.logo_url);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Market Summary region (from finance.md "Market Summary" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_market_summary_region() {
    use finance_query::{Region, finance};

    // From finance.md "Market Summary" section — specific region + full field access pattern
    let summary = finance::market_summary(Some(Region::Canada)).await.unwrap();
    for quote in &summary {
        let price = quote.regular_market_price.as_ref().and_then(|v| v.raw);
        let change_pct = quote
            .regular_market_change_percent
            .as_ref()
            .and_then(|v| v.raw)
            .unwrap_or(0.0);
        println!("{}: ${:?} ({:+.2}%)", quote.symbol, price, change_pct);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Trending region (from finance.md "Trending" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_trending_region() {
    use finance_query::{Region, finance};

    // From finance.md "Trending" section — specific region
    let trending = finance::trending(Some(Region::Singapore)).await.unwrap();
    for quote in &trending {
        println!("{}", quote.symbol);
    }
    assert!(!trending.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Market Hours (from finance.md "Market Hours" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_market_hours() {
    use finance_query::finance;

    // From finance.md "Market Hours" section
    // US market hours (default)
    let hours = finance::hours(None).await.unwrap();
    assert!(!hours.markets.is_empty());

    // Japan market hours
    let hours = finance::hours(Some("JP")).await.unwrap();
    for market in &hours.markets {
        println!("{}: {}", market.name, market.status);
        println!("  Open: {:?}", market.open);
        println!("  Close: {:?}", market.close);
    }
}

// ---------------------------------------------------------------------------
// Network tests — Indices (from finance.md "Indices" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_indices() {
    use finance_query::{IndicesRegion, finance};

    // From finance.md "Indices" section

    // All world indices
    let all = finance::indices(None).await.unwrap();
    println!("Fetched {} indices", all.success_count());
    assert!(all.success_count() > 0);

    // Only Americas indices (^DJI, ^GSPC, ^IXIC, etc.)
    let americas = finance::indices(Some(IndicesRegion::Americas))
        .await
        .unwrap();
    for (symbol, quote) in &americas.quotes {
        if let (Some(price_fv), Some(change_pct_fv)) = (
            &quote.regular_market_price,
            &quote.regular_market_change_percent,
        ) && let (Some(price), Some(change_pct)) = (price_fv.raw, change_pct_fv.raw)
        {
            println!("{}: {:.2} ({:+.2}%)", symbol, price, change_pct);
        }
    }

    // Other regions
    let _europe = finance::indices(Some(IndicesRegion::Europe)).await.unwrap();
    let _asia = finance::indices(Some(IndicesRegion::AsiaPacific))
        .await
        .unwrap();
}

// ---------------------------------------------------------------------------
// Network tests — Predefined Screeners (from finance.md "Predefined Screeners" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_screener_predefined() {
    use finance_query::{Screener, finance};

    // From finance.md "Predefined Screeners" section

    // Top gainers
    let gainers = finance::screener(Screener::DayGainers, 25).await.unwrap();

    // Most actives
    let actives = finance::screener(Screener::MostActives, 25).await.unwrap();

    // Day losers
    let losers = finance::screener(Screener::DayLosers, 25).await.unwrap();

    // Process results — regular_market_change_percent is FormattedValue<f64> (not Option)
    for quote in &gainers.quotes {
        let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
        println!("{}: {:+.2}%", quote.symbol, change_pct);
    }

    assert!(!gainers.quotes.is_empty());
    let _ = actives;
    let _ = losers;
}

// ---------------------------------------------------------------------------
// Network tests — Custom Screener (from finance.md "Custom Screeners" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_screener() {
    use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt, finance};

    // From finance.md "Custom Screeners" section — find US large-cap tech stocks
    let query = EquityScreenerQuery::new()
        .size(50)
        .sort_by(EquityField::IntradayMarketCap, false)
        .add_condition(EquityField::Region.eq_str("us"))
        .add_condition(EquityField::Sector.eq_str("Technology"))
        .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
        .add_condition(EquityField::AvgDailyVol3M.gt(1_000_000.0));

    let results = finance::custom_screener(query).await.unwrap();
    println!("Found {} stocks", results.quotes.len());
    assert!(!results.quotes.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Sector full fields (from finance.md "Sectors" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_sector_full_fields() {
    use finance_query::{Sector, finance};

    // From finance.md "Sectors" section — full field access pattern
    let tech = finance::sector(Sector::Technology).await.unwrap();

    println!("Sector: {}", tech.name);
    if let Some(overview) = &tech.overview {
        if let Some(count) = overview.companies_count {
            println!("  Companies: {}", count);
        }
        if let Some(market_cap_fv) = &overview.market_cap
            && let Some(market_cap) = market_cap_fv.raw
        {
            println!("  Market Cap: ${:.2}B", market_cap / 1_000_000_000.0);
        }
    }

    // Top companies in the sector
    println!("Top companies:");
    for company in tech.top_companies.iter().take(10) {
        println!(
            "  {} - {}",
            company.symbol,
            company.name.as_deref().unwrap_or("N/A")
        );
    }

    // Sector ETFs
    println!("Sector ETFs: {}", tech.top_etfs.len());

    // Industries in this sector
    println!("Industries: {}", tech.industries.len());

    assert!(!tech.name.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Industry full fields (from finance.md "Industries" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_industry_full_fields() {
    use finance_query::finance;

    // From finance.md "Industries" section — full field access pattern
    let semiconductors = finance::industry("semiconductors").await.unwrap();

    println!("Industry: {}", semiconductors.name);
    if let Some(overview) = &semiconductors.overview {
        println!("  Companies: {:?}", overview.companies_count);
        println!("  Market Cap: ${:?}B", overview.market_cap.map(|m| m / 1e9));
    }

    // Top companies
    for company in semiconductors.top_companies.iter().take(5) {
        println!(
            "  {} - {}",
            company.symbol,
            company.name.as_deref().unwrap_or("")
        );
    }

    assert!(!semiconductors.name.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — General News (from finance.md "News & Transcripts" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_news() {
    use finance_query::finance;

    // From finance.md "General News" section
    let news = finance::news().await.unwrap();

    for article in news.iter().take(10) {
        println!("{}", article.title);
        println!("  Source: {}", article.source);
        println!("  Time: {}", article.time);
        println!("  Link: {}", article.link);
    }

    assert!(!news.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Earnings Transcripts (from finance.md "Earnings Transcripts" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_earnings_transcripts() {
    use finance_query::finance;

    // From finance.md "Earnings Transcripts" section

    // Get the latest transcript
    let latest = finance::earnings_transcript("AAPL", None, None)
        .await
        .unwrap();
    println!("Quarter: {} {}", latest.quarter(), latest.year());
    println!(
        "Speakers: {}",
        latest.transcript_content.speaker_mapping.len()
    );

    // Get specific quarter
    let _q4_2024 = finance::earnings_transcript("AAPL", Some("Q4"), Some(2024))
        .await
        .unwrap();

    // Get all available transcripts
    let all = finance::earnings_transcripts("MSFT", None).await.unwrap();
    println!("Found {} transcripts", all.len());

    // Get only recent transcripts
    let recent = finance::earnings_transcripts("NVDA", Some(5))
        .await
        .unwrap();
    for t in &recent {
        println!(
            "{}: {} {}",
            t.title,
            t.transcript.quarter(),
            t.transcript.year()
        );
    }
}

// ---------------------------------------------------------------------------
// Network tests — Exchanges (from finance.md "Exchanges" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_exchanges() {
    use finance_query::finance;

    // From finance.md "Exchanges" section
    let exchanges = finance::exchanges().await.unwrap();
    for exchange in &exchanges {
        println!(
            "{} - {} (suffix: {})",
            exchange.country, exchange.market, exchange.suffix
        );
    }
    assert!(!exchanges.is_empty());
}

// ---------------------------------------------------------------------------
// Network tests — Currencies (from finance.md "Currencies" section)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_currencies() {
    use finance_query::finance;

    // From finance.md "Currencies" section
    let currencies = finance::currencies().await.unwrap();
    for currency in &currencies {
        let symbol = currency.symbol.as_deref().unwrap_or("N/A");
        let name = currency.short_name.as_deref().unwrap_or("N/A");
        println!("{}: {}", symbol, name);
    }
    assert!(!currencies.is_empty());
}
