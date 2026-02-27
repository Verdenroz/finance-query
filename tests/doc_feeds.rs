//! Compile and runtime tests for docs/library/feeds.md
//!
//! Requires the `rss` feature flag:
//!   cargo test --test doc_feeds --features rss
//!   cargo test --test doc_feeds --features rss -- --ignored   (network tests)

#![cfg(feature = "rss")]

use finance_query::feeds::{FeedEntry, FeedSource};

// ---------------------------------------------------------------------------
// FeedEntry — compile-time field verification
// ---------------------------------------------------------------------------

/// Verifies all FeedEntry fields documented in feeds.md.
#[allow(dead_code)]
fn _verify_feed_entry_fields(e: FeedEntry) {
    let _: String = e.title;
    let _: String = e.url;
    let _: Option<String> = e.published;
    let _: Option<String> = e.summary;
    let _: String = e.source;
}

// ---------------------------------------------------------------------------
// FeedSource — URL and name methods (pure, no network)
// ---------------------------------------------------------------------------

/// All named FeedSource variants must have non-empty URLs and names.
#[test]
fn test_feed_source_urls_are_non_empty() {
    let sources = [
        FeedSource::FederalReserve,
        FeedSource::SecPressReleases,
        FeedSource::SecFilings("10-K".to_string()),
        FeedSource::MarketWatch,
        FeedSource::Cnbc,
        FeedSource::Bloomberg,
        FeedSource::FinancialTimes,
        FeedSource::NytBusiness,
        FeedSource::GuardianBusiness,
        FeedSource::Investing,
        FeedSource::Bea,
        FeedSource::Ecb,
        FeedSource::Cfpb,
        FeedSource::WsjMarkets,
        FeedSource::Fortune,
        FeedSource::BusinessWire,
        FeedSource::CoinDesk,
        FeedSource::CoinTelegraph,
        FeedSource::TechCrunch,
        FeedSource::HackerNews,
        FeedSource::OilPrice,
        FeedSource::CalculatedRisk,
        FeedSource::Scmp,
        FeedSource::NikkeiAsia,
        FeedSource::BankOfEngland,
        FeedSource::VentureBeat,
        FeedSource::YCombinator,
        FeedSource::TheEconomist,
        FeedSource::FinancialPost,
        FeedSource::FtLex,
        FeedSource::RitholtzBigPicture,
        FeedSource::Custom("https://example.com/feed.xml".to_string()),
    ];

    for source in &sources {
        let url = source.url();
        assert!(
            !url.is_empty(),
            "URL should not be empty for {:?}",
            source.name()
        );
        assert!(
            url.starts_with("http://") || url.starts_with("https://"),
            "URL should be http(s): {} for {}",
            url,
            source.name()
        );
    }
}

#[test]
fn test_feed_source_names_are_non_empty() {
    let sources = [
        FeedSource::FederalReserve,
        FeedSource::SecPressReleases,
        FeedSource::SecFilings("8-K".to_string()),
        FeedSource::MarketWatch,
        FeedSource::Bloomberg,
        FeedSource::WsjMarkets,
        FeedSource::CoinDesk,
        FeedSource::TechCrunch,
        FeedSource::Custom("https://example.com/feed.xml".to_string()),
    ];

    for source in &sources {
        let name = source.name();
        assert!(!name.is_empty(), "name should not be empty");
    }
}

#[test]
fn test_sec_filings_url_contains_form_type() {
    let source = FeedSource::SecFilings("10-K".to_string());
    let url = source.url();
    assert!(
        url.contains("10-K"),
        "SEC filings URL should contain the form type"
    );
}

#[test]
fn test_custom_source_url_is_passthrough() {
    let custom_url = "https://example.com/custom-feed.rss";
    let source = FeedSource::Custom(custom_url.to_string());
    assert_eq!(source.url(), custom_url);
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_federal_reserve() {
    use finance_query::feeds;

    let entries = feeds::fetch(FeedSource::FederalReserve).await.unwrap();
    println!("Federal Reserve entries: {}", entries.len());
    for entry in entries.iter().take(3) {
        println!(
            "[{}] {}: {}",
            entry.source,
            entry.published.as_deref().unwrap_or("?"),
            entry.title
        );
        assert!(!entry.title.is_empty());
        assert!(!entry.url.is_empty());
        assert_eq!(entry.source, "Federal Reserve");
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_all_deduplicates() {
    use finance_query::feeds;
    use std::collections::HashSet;

    let news = feeds::fetch_all(&[
        FeedSource::FederalReserve,
        FeedSource::SecPressReleases,
        FeedSource::MarketWatch,
    ])
    .await
    .unwrap();

    // Verify deduplication: all URLs must be unique
    let urls: HashSet<&str> = news.iter().map(|e| e.url.as_str()).collect();
    assert_eq!(
        urls.len(),
        news.len(),
        "fetch_all should deduplicate by URL"
    );

    println!("Aggregated entries (deduplicated): {}", news.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_single_feed_iteration() {
    use finance_query::feeds;

    // From feeds.md "Fetching a Single Feed" section — exact iteration pattern
    let fed_news = feeds::fetch(FeedSource::FederalReserve).await.unwrap();

    for entry in fed_news.iter().take(5) {
        println!(
            "{}: {}",
            entry.published.as_deref().unwrap_or("?"),
            entry.title
        );
        if let Some(_url) = entry.url.as_str().chars().next() {
            println!("  {}", entry.url);
        }
    }

    assert!(!fed_news.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_all_five_sources() {
    use finance_query::feeds;

    // From feeds.md "Fetching Multiple Feeds" section — all 5 documented sources
    let news = feeds::fetch_all(&[
        FeedSource::FederalReserve,
        FeedSource::SecPressReleases,
        FeedSource::MarketWatch,
        FeedSource::Bloomberg,
        FeedSource::WsjMarkets,
    ])
    .await
    .unwrap();

    println!("Total entries (deduplicated): {}", news.len());
    for entry in news.iter().take(10) {
        println!(
            "[{}] {}: {}",
            entry.source,
            entry.published.as_deref().unwrap_or("?"),
            entry.title
        );
    }

    assert!(!news.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_feed_url() {
    use finance_query::feeds;

    // From feeds.md "Custom Feed URLs" section
    // Tested with the Federal Reserve URL to verify the Custom path works
    // (example.com/feed.xml from the doc is not a real RSS feed)
    let fed_url = FeedSource::FederalReserve.url().to_string();
    let custom = feeds::fetch(FeedSource::Custom(fed_url)).await.unwrap();

    assert!(!custom.is_empty());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_sec_filings_10k() {
    use finance_query::{FinanceError, feeds};

    // SEC EDGAR's cgi-bin atom feed requires a User-Agent with a contact email
    // (format: "app/version (email)"). The feeds client reads EDGAR_EMAIL from
    // the environment; without it the UA falls back to a GitHub URL, which SEC
    // rejects with an HTML page → FeedParseError. Skip gracefully in that case;
    // the feed is verified to work when EDGAR_EMAIL is configured (e.g. via
    // server/.env). Connection/timeout errors are also skipped.
    let result = feeds::fetch(FeedSource::SecFilings("10-K".to_string())).await;
    let filings = match result {
        Ok(f) => f,
        Err(FinanceError::FeedParseError { .. }) => {
            eprintln!(
                "SEC filings feed skipped: likely missing EDGAR_EMAIL env var (SEC requires email in User-Agent)"
            );
            return;
        }
        Err(FinanceError::HttpError(ref e)) if e.is_connect() || e.is_timeout() => {
            eprintln!("SEC filings feed skipped: connection error");
            return;
        }
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    println!("SEC 10-K filings: {}", filings.len());
    for f in filings.iter().take(3) {
        println!("{}: {}", f.published.as_deref().unwrap_or("?"), f.title);
        println!("  {}", f.url);
    }

    assert!(!filings.is_empty());
}
