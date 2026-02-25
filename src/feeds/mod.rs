//! RSS/Atom news feed aggregation.
//!
//! Requires the **`rss`** feature flag.
//!
//! Fetches and parses RSS/Atom feeds from named financial sources or arbitrary URLs.
//! Multiple feeds can be fetched and merged in one call with automatic deduplication.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::feeds::{self, FeedSource};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Fetch Federal Reserve press releases
//! let fed_news = feeds::fetch(FeedSource::FederalReserve).await?;
//! for entry in fed_news.iter().take(5) {
//!     println!("{}: {}", entry.published.as_deref().unwrap_or("?"), entry.title);
//! }
//!
//! // Aggregate multiple sources
//! let news = feeds::fetch_all(&[
//!     FeedSource::FederalReserve,
//!     FeedSource::SecPressReleases,
//!     FeedSource::MarketWatch,
//! ]).await?;
//! println!("Total entries: {}", news.len());
//! # Ok(())
//! # }
//! ```

use feed_rs::parser;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

use crate::error::{FinanceError, Result};

static FEED_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn feed_client() -> &'static reqwest::Client {
    FEED_CLIENT.get_or_init(|| {
        // SEC EDGAR requires "app/version (email)" — nothing else in the UA.
        // Other sites accept any reasonable UA. We use the email format when
        // EDGAR_EMAIL is set (same env var as the edgar module), falling back
        // to a github URL for environments without EDGAR configured.
        let ua = match std::env::var("EDGAR_EMAIL") {
            Ok(email) if !email.trim().is_empty() => {
                format!(
                    "finance-query/{} ({})",
                    env!("CARGO_PKG_VERSION"),
                    email.trim()
                )
            }
            _ => concat!(
                "finance-query/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/Verdenroz/finance-query)"
            )
            .to_string(),
        };
        reqwest::Client::builder()
            .user_agent(ua)
            .build()
            .expect("failed to build feeds HTTP client")
    })
}

/// A named or custom RSS/Atom feed source.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum FeedSource {
    /// Federal Reserve press releases and speeches
    FederalReserve,
    /// SEC press releases (enforcement actions, rule changes)
    SecPressReleases,
    /// SEC EDGAR filing feed — specify form type (e.g., `"10-K"`, `"8-K"`)
    SecFilings(String),
    /// MarketWatch top stories
    MarketWatch,
    /// CNBC Markets
    Cnbc,
    /// Bloomberg Markets news
    Bloomberg,
    /// Financial Times Markets section
    FinancialTimes,
    /// The New York Times Business section
    NytBusiness,
    /// The Guardian Business section
    GuardianBusiness,
    /// Investing.com all news
    Investing,
    /// U.S. Bureau of Economic Analysis data releases
    Bea,
    /// European Central Bank press releases and speeches
    Ecb,
    /// Consumer Financial Protection Bureau newsroom
    Cfpb,
    /// Wall Street Journal Markets top stories
    WsjMarkets,
    /// Fortune — business and finance news
    Fortune,
    /// Business Wire — official corporate press releases (earnings, dividends, M&A)
    BusinessWire,
    /// CoinDesk — cryptocurrency and blockchain news
    CoinDesk,
    /// CoinTelegraph — cryptocurrency news and analysis
    CoinTelegraph,
    /// TechCrunch — startup, VC, and tech industry news
    TechCrunch,
    /// Hacker News — community-curated tech posts with 100+ points
    HackerNews,
    /// OilPrice.com — crude oil, natural gas, and energy geopolitics
    OilPrice,
    /// Calculated Risk — housing starts, mortgage rates, and macro data
    CalculatedRisk,
    /// South China Morning Post — China business, regulation, and trade
    Scmp,
    /// Nikkei Asia — Japanese and Asian business news
    NikkeiAsia,
    /// Bank of England — UK monetary policy, rate decisions, and regulatory notices
    BankOfEngland,
    /// VentureBeat — AI funding rounds and enterprise technology
    VentureBeat,
    /// Y Combinator Blog — startup ecosystem announcements (low-frequency)
    YCombinator,
    /// The Economist — global economics and market analysis
    TheEconomist,
    /// Financial Post — Canadian market and business news
    FinancialPost,
    /// Financial Times Lex — short daily market commentary column
    FtLex,
    /// The Big Picture (Ritholtz) — macro finance analysis and commentary
    RitholtzBigPicture,
    /// Custom feed URL
    Custom(String),
}

impl FeedSource {
    /// Return the URL for this feed source.
    pub fn url(&self) -> String {
        match self {
            Self::FederalReserve => {
                "https://www.federalreserve.gov/feeds/press_all.xml".to_string()
            }
            Self::SecPressReleases => "https://www.sec.gov/news/pressreleases.rss".to_string(),
            Self::SecFilings(form_type) => format!(
                "https://www.sec.gov/cgi-bin/browse-edgar?action=getcurrent&type={form_type}&output=atom"
            ),
            Self::MarketWatch => {
                "https://feeds.content.dowjones.io/public/rss/mw_topstories".to_string()
            }
            Self::Cnbc => "https://www.cnbc.com/id/100003114/device/rss/rss.html".to_string(),
            Self::Bloomberg => "https://feeds.bloomberg.com/markets/news.rss".to_string(),
            Self::FinancialTimes => "https://www.ft.com/markets?format=rss".to_string(),
            Self::NytBusiness => {
                "https://rss.nytimes.com/services/xml/rss/nyt/Business.xml".to_string()
            }
            Self::GuardianBusiness => "https://www.theguardian.com/business/rss".to_string(),
            Self::Investing => "https://www.investing.com/rss/news.rss".to_string(),
            Self::Bea => "https://apps.bea.gov/rss/rss.xml".to_string(),
            Self::Ecb => "https://www.ecb.europa.eu/rss/press.html".to_string(),
            Self::Cfpb => "https://www.consumerfinance.gov/about-us/newsroom/feed/".to_string(),
            Self::WsjMarkets => "https://feeds.a.dj.com/rss/RSSMarketsMain.xml".to_string(),
            Self::Fortune => "https://fortune.com/feed".to_string(),
            Self::BusinessWire => {
                "https://feed.businesswire.com/rss/home/?rss=G1QFDERJXkJeGVtQXw==".to_string()
            }
            Self::CoinDesk => "https://www.coindesk.com/arc/outboundfeeds/rss/".to_string(),
            Self::CoinTelegraph => "https://cointelegraph.com/rss".to_string(),
            Self::TechCrunch => "https://techcrunch.com/feed/".to_string(),
            Self::HackerNews => "https://hnrss.org/newest?points=100".to_string(),
            Self::OilPrice => "https://oilprice.com/rss/main".to_string(),
            Self::CalculatedRisk => "https://calculatedrisk.substack.com/feed".to_string(),
            Self::Scmp => "https://www.scmp.com/rss/91/feed".to_string(),
            Self::NikkeiAsia => "https://asia.nikkei.com/rss/feed/nar".to_string(),
            Self::BankOfEngland => "https://www.bankofengland.co.uk/rss/news".to_string(),
            Self::VentureBeat => "https://venturebeat.com/feed/".to_string(),
            Self::YCombinator => "https://blog.ycombinator.com/feed/".to_string(),
            Self::TheEconomist => {
                "https://www.economist.com/sections/economics/rss.xml".to_string()
            }
            Self::FinancialPost => "https://financialpost.com/feed".to_string(),
            Self::FtLex => "https://www.ft.com/lex?format=rss".to_string(),
            Self::RitholtzBigPicture => "https://ritholtz.com/feed/".to_string(),
            Self::Custom(url) => url.clone(),
        }
    }

    /// Human-readable source name, used in [`FeedEntry::source`].
    pub fn name(&self) -> String {
        match self {
            Self::FederalReserve => "Federal Reserve".to_string(),
            Self::SecPressReleases => "SEC".to_string(),
            Self::SecFilings(form) => format!("SEC EDGAR ({form})"),
            Self::MarketWatch => "MarketWatch".to_string(),
            Self::Cnbc => "CNBC".to_string(),
            Self::Bloomberg => "Bloomberg".to_string(),
            Self::FinancialTimes => "Financial Times".to_string(),
            Self::NytBusiness => "New York Times".to_string(),
            Self::GuardianBusiness => "The Guardian".to_string(),
            Self::Investing => "Investing.com".to_string(),
            Self::Bea => "Bureau of Economic Analysis".to_string(),
            Self::Ecb => "European Central Bank".to_string(),
            Self::Cfpb => "CFPB".to_string(),
            Self::WsjMarkets => "Wall Street Journal".to_string(),
            Self::Fortune => "Fortune".to_string(),
            Self::BusinessWire => "Business Wire".to_string(),
            Self::CoinDesk => "CoinDesk".to_string(),
            Self::CoinTelegraph => "CoinTelegraph".to_string(),
            Self::TechCrunch => "TechCrunch".to_string(),
            Self::HackerNews => "Hacker News".to_string(),
            Self::OilPrice => "OilPrice.com".to_string(),
            Self::CalculatedRisk => "Calculated Risk".to_string(),
            Self::Scmp => "South China Morning Post".to_string(),
            Self::NikkeiAsia => "Nikkei Asia".to_string(),
            Self::BankOfEngland => "Bank of England".to_string(),
            Self::VentureBeat => "VentureBeat".to_string(),
            Self::YCombinator => "Y Combinator".to_string(),
            Self::TheEconomist => "The Economist".to_string(),
            Self::FinancialPost => "Financial Post".to_string(),
            Self::FtLex => "Financial Times Lex".to_string(),
            Self::RitholtzBigPicture => "The Big Picture".to_string(),
            Self::Custom(url) => url.clone(),
        }
    }
}

/// A single entry from an RSS/Atom feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FeedEntry {
    /// Article or item title
    pub title: String,
    /// Canonical link to the article
    pub url: String,
    /// Publication date/time as an RFC 3339 string (if available)
    pub published: Option<String>,
    /// Short summary or description
    pub summary: Option<String>,
    /// Name of the feed source
    pub source: String,
}

/// Fetch and parse a single feed source.
///
/// Returns an empty `Vec` (not an error) when the feed is reachable but empty.
pub async fn fetch(source: FeedSource) -> Result<Vec<FeedEntry>> {
    fetch_url(&source.url(), &source.name()).await
}

/// Fetch multiple feed sources concurrently and merge the results.
///
/// Results are deduplicated by URL and sorted newest-first when dates are available.
/// Feeds that fail individually are skipped (not propagated as errors).
pub async fn fetch_all(sources: &[FeedSource]) -> Result<Vec<FeedEntry>> {
    let futures: Vec<_> = sources
        .iter()
        .map(|s| fetch_url_owned(s.url(), s.name()))
        .collect();

    let results = join_all(futures).await;

    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut entries: Vec<FeedEntry> = results
        .into_iter()
        .flat_map(|r| r.unwrap_or_default())
        .filter(|e| seen_urls.insert(e.url.clone()))
        .collect();

    // Sort newest-first where dates are present
    entries.sort_by(|a, b| b.published.cmp(&a.published));

    Ok(entries)
}

async fn fetch_url(url: impl AsRef<str>, source_name: impl Into<String>) -> Result<Vec<FeedEntry>> {
    let url = url.as_ref();
    let source = source_name.into();

    let text = feed_client()
        .get(url)
        .send()
        .await
        .map_err(FinanceError::HttpError)?
        .text()
        .await
        .map_err(FinanceError::HttpError)?;

    let feed = parser::parse(text.as_bytes()).map_err(|e| FinanceError::FeedParseError {
        url: url.to_string(),
        context: e.to_string(),
    })?;

    let entries = feed
        .entries
        .into_iter()
        .filter_map(|entry| {
            let title = entry.title.map(|t| t.content)?.trim().to_string();
            if title.is_empty() {
                return None;
            }

            let url_str = entry
                .links
                .into_iter()
                .next()
                .map(|l| l.href)
                .unwrap_or_default();

            if url_str.is_empty() {
                return None;
            }

            let published = entry.published.or(entry.updated).map(|dt| dt.to_rfc3339());

            let summary = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body));

            Some(FeedEntry {
                title,
                url: url_str,
                published,
                summary,
                source: source.clone(),
            })
        })
        .collect();

    Ok(entries)
}

async fn fetch_url_owned(url: String, source_name: String) -> Result<Vec<FeedEntry>> {
    fetch_url(url, source_name).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_source_urls() {
        assert!(FeedSource::FederalReserve.url().starts_with("https://"));
        assert!(FeedSource::SecPressReleases.url().starts_with("https://"));
        assert!(
            FeedSource::SecFilings("10-K".to_string())
                .url()
                .contains("10-K")
        );
        assert_eq!(
            FeedSource::Custom("https://example.com/feed.rss".to_string()).url(),
            "https://example.com/feed.rss"
        );
        assert!(FeedSource::Bloomberg.url().starts_with("https://"));
        assert!(FeedSource::FinancialTimes.url().starts_with("https://"));
        assert!(FeedSource::NytBusiness.url().starts_with("https://"));
        assert!(FeedSource::GuardianBusiness.url().starts_with("https://"));
        assert!(FeedSource::Investing.url().starts_with("https://"));
        assert!(FeedSource::Bea.url().starts_with("https://"));
        assert!(FeedSource::Ecb.url().starts_with("https://"));
        assert!(FeedSource::Cfpb.url().starts_with("https://"));
        // New sources
        assert!(FeedSource::WsjMarkets.url().contains("dj.com"));
        assert!(FeedSource::Fortune.url().contains("fortune.com"));
        assert!(FeedSource::BusinessWire.url().contains("businesswire.com"));
        assert!(FeedSource::CoinDesk.url().contains("coindesk.com"));
        assert!(
            FeedSource::CoinTelegraph
                .url()
                .contains("cointelegraph.com")
        );
        assert!(FeedSource::TechCrunch.url().contains("techcrunch.com"));
        assert!(FeedSource::HackerNews.url().contains("hnrss.org"));
    }

    #[test]
    fn test_feed_source_names() {
        assert_eq!(FeedSource::FederalReserve.name(), "Federal Reserve");
        assert_eq!(FeedSource::MarketWatch.name(), "MarketWatch");
        assert_eq!(FeedSource::Bloomberg.name(), "Bloomberg");
        assert_eq!(FeedSource::FinancialTimes.name(), "Financial Times");
        assert_eq!(FeedSource::NytBusiness.name(), "New York Times");
        assert_eq!(FeedSource::GuardianBusiness.name(), "The Guardian");
        assert_eq!(FeedSource::Investing.name(), "Investing.com");
        assert_eq!(FeedSource::Bea.name(), "Bureau of Economic Analysis");
        assert_eq!(FeedSource::Ecb.name(), "European Central Bank");
        assert_eq!(FeedSource::Cfpb.name(), "CFPB");
        // New sources
        assert_eq!(FeedSource::WsjMarkets.name(), "Wall Street Journal");
        assert_eq!(FeedSource::Fortune.name(), "Fortune");
        assert_eq!(FeedSource::BusinessWire.name(), "Business Wire");
        assert_eq!(FeedSource::CoinDesk.name(), "CoinDesk");
        assert_eq!(FeedSource::CoinTelegraph.name(), "CoinTelegraph");
        assert_eq!(FeedSource::TechCrunch.name(), "TechCrunch");
        assert_eq!(FeedSource::HackerNews.name(), "Hacker News");
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_fed_reserve() {
        let entries = fetch(FeedSource::FederalReserve).await;
        assert!(entries.is_ok(), "Expected ok, got: {:?}", entries.err());
        let entries = entries.unwrap();
        assert!(!entries.is_empty());
        for e in entries.iter().take(3) {
            assert!(!e.title.is_empty());
            assert!(!e.url.is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_bloomberg() {
        let entries = fetch(FeedSource::Bloomberg).await;
        assert!(entries.is_ok(), "Expected ok, got: {:?}", entries.err());
        let entries = entries.unwrap();
        assert!(!entries.is_empty());
        for e in entries.iter().take(3) {
            assert!(!e.title.is_empty());
            assert!(!e.url.is_empty());
            assert_eq!(e.source, "Bloomberg");
        }
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_financial_times() {
        let entries = fetch(FeedSource::FinancialTimes).await;
        assert!(entries.is_ok(), "Expected ok, got: {:?}", entries.err());
        let entries = entries.unwrap();
        assert!(!entries.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_bea() {
        let entries = fetch(FeedSource::Bea).await;
        assert!(entries.is_ok(), "Expected ok, got: {:?}", entries.err());
        let entries = entries.unwrap();
        assert!(!entries.is_empty());
        assert_eq!(entries[0].source, "Bureau of Economic Analysis");
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_fetch_ecb() {
        let entries = fetch(FeedSource::Ecb).await;
        assert!(entries.is_ok(), "Expected ok, got: {:?}", entries.err());
        let entries = entries.unwrap();
        assert!(!entries.is_empty());
    }
}
