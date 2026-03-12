mod cache;
mod graphql;
mod metrics;
mod rate_limit;
mod services;

use axum::{
    Router,
    extract::{
        Extension, Path, Query, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Json},
    routing::{get, post},
};
use cache::Cache;
use finance_query::{
    EquityField, EquityScreenerQuery, FinanceError, Frequency, FundField, FundScreenerQuery,
    QuoteType, Region, Screener, Sector, StatementType, ValueFormat, feeds::FeedSource, finance,
    streaming::PriceStream,
};
use futures_util::{SinkExt, StreamExt};
use rate_limit::{RateLimitConfig, RateLimiterState};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    pub cache: Cache,
    pub stream_hub: StreamHub,
}

/// Process-wide hub that maintains a single upstream Yahoo Finance stream.
///
/// Multiple downstream WebSocket clients can subscribe/unsubscribe to symbols.
/// Symbol subscriptions are ref-counted so each symbol is only subscribed once upstream.
#[derive(Clone, Default)]
pub struct StreamHub {
    inner: Arc<tokio::sync::Mutex<StreamHubInner>>,
}

#[derive(Default)]
struct StreamHubInner {
    upstream: Option<PriceStream>,
    symbol_ref_counts: HashMap<String, usize>,
}

impl StreamHub {
    fn new() -> Self {
        Self::default()
    }

    pub async fn resubscribe(&self) -> Option<PriceStream> {
        let inner = self.inner.lock().await;
        inner.upstream.as_ref().map(|s| s.resubscribe())
    }

    pub async fn subscribe_symbols(&self, symbols: &[String]) -> Result<(), FinanceError> {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return Ok(());
        }

        let mut inner = self.inner.lock().await;

        // Track which symbols are newly needed upstream.
        let mut newly_needed: Vec<String> = Vec::new();
        for symbol in &unique {
            let count = inner.symbol_ref_counts.entry(symbol.clone()).or_insert(0);
            if *count == 0 {
                newly_needed.push(symbol.clone());
            }
            *count += 1;
        }

        // Create upstream stream if this is the first active subscription.
        if inner.upstream.is_none() {
            let refs: Vec<&str> = unique.iter().map(|s| s.as_str()).collect();
            let stream = PriceStream::subscribe(&refs).await?;
            inner.upstream = Some(stream);
            return Ok(());
        }

        // Add newly needed symbols to upstream.
        if !newly_needed.is_empty()
            && let Some(upstream) = inner.upstream.as_ref()
        {
            let refs: Vec<&str> = newly_needed.iter().map(|s| s.as_str()).collect();
            upstream.add_symbols(&refs).await;
        }

        Ok(())
    }

    pub async fn unsubscribe_symbols(&self, symbols: &[String]) {
        let unique: HashSet<String> = symbols
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if unique.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;

        let mut newly_unneeded: Vec<String> = Vec::new();
        for symbol in &unique {
            if let Some(count) = inner.symbol_ref_counts.get_mut(symbol)
                && *count > 0
            {
                *count -= 1;
                if *count == 0 {
                    newly_unneeded.push(symbol.clone());
                }
            }
        }

        for symbol in &newly_unneeded {
            inner.symbol_ref_counts.remove(symbol);
        }

        if let Some(upstream) = inner.upstream.as_ref()
            && !newly_unneeded.is_empty()
        {
            let refs: Vec<&str> = newly_unneeded.iter().map(|s| s.as_str()).collect();
            upstream.remove_symbols(&refs).await;
        }

        // If nothing is subscribed anywhere, close upstream to stop background tasks.
        if inner.symbol_ref_counts.is_empty()
            && let Some(upstream) = inner.upstream.take()
        {
            upstream.close().await;
        }
    }
}

// Server-specific default values
mod defaults {
    /// Default number of similar stocks to return
    pub const SIMILAR_STOCKS_LIMIT: u32 = 5;
    /// Default number of search results
    pub const SEARCH_HITS: u32 = 10;
    /// Default chart interval
    pub const DEFAULT_INTERVAL: &str = "1d";
    /// Default chart range
    pub const DEFAULT_RANGE: &str = "1mo";
    /// Default server port
    pub const SERVER_PORT: u16 = 8000;
    /// Default screeners count
    pub const SCREENERS_COUNT: u32 = 25;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: String,
    version: String,
    timestamp: String,
    notices: &'static [&'static str],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PingResponse {
    message: String,
}

// Query parameter structs
/// Parse format query parameter into ValueFormat
fn parse_format(s: Option<&str>) -> ValueFormat {
    s.and_then(ValueFormat::parse).unwrap_or_default()
}

/// Parse comma-separated field names into a set for filtering
fn parse_fields(s: Option<&str>) -> Option<std::collections::HashSet<String>> {
    s.map(|fields_str| {
        fields_str
            .split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect()
    })
}

/// Parse region code string into Region enum
fn parse_region(s: &str) -> Option<Region> {
    s.parse().ok()
}

/// Recursively filter a JSON value to only include specified fields.
///
/// Strategy: an object is treated as a **data object** if it has at least one
/// key that directly matches `fields`. In that case only matching keys are kept
/// and all non-matching keys (including nested containers) are dropped.
///
/// If an object has *no* direct matches it is treated as a **transparent
/// wrapper** (e.g. `{ "quotes": { "AAPL": {...} } }`). Its container-typed
/// values are recursed into and the key is kept only when the result is
/// non-empty — preventing false positives from deeply nested metadata fields
/// (e.g. `equityPerformance.benchmark.symbol`) leaking through.
///
/// Arrays always recurse into each element; empty objects produced by
/// filtering are removed from the result.
fn filter_fields(
    value: serde_json::Value,
    fields: &std::collections::HashSet<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let has_direct_match = map.keys().any(|k| fields.contains(k));

            let filtered = map
                .into_iter()
                .filter_map(|(k, v)| {
                    if fields.contains(&k) {
                        // Explicitly requested: keep whole value.
                        Some((k, v))
                    } else if !has_direct_match
                        && matches!(
                            v,
                            serde_json::Value::Object(_) | serde_json::Value::Array(_)
                        )
                    {
                        // Pure wrapper — recurse and prune if nothing matched.
                        let filtered_v = filter_fields(v, fields);
                        match &filtered_v {
                            serde_json::Value::Object(m) if m.is_empty() => None,
                            serde_json::Value::Array(a) if a.is_empty() => None,
                            _ => Some((k, filtered_v)),
                        }
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::Value::Object(filtered)
        }
        serde_json::Value::Array(arr) => {
            let filtered: Vec<_> = arr
                .into_iter()
                .map(|v| filter_fields(v, fields))
                .filter(|v| !matches!(v, serde_json::Value::Object(m) if m.is_empty()))
                .collect();
            serde_json::Value::Array(filtered)
        }
        other => other,
    }
}

/// Apply format transformation and optional field filtering
fn apply_transforms(
    value: serde_json::Value,
    format: ValueFormat,
    fields: Option<&std::collections::HashSet<String>>,
) -> serde_json::Value {
    let formatted = format.transform(value);
    match fields {
        Some(f) => filter_fields(formatted, f),
        None => formatted,
    }
}

#[derive(Deserialize)]
struct QuoteQuery {
    /// Whether to include company logo URL (default: false)
    #[serde(default)]
    logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct QuotesQuery {
    symbols: String, // Comma-separated symbols
    /// Whether to include company logo URLs (default: false)
    #[serde(default)]
    logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct RecommendationsQuery {
    #[serde(default = "default_limit")]
    limit: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct ChartQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Include events (dividends, splits, capital gains) in response
    #[serde(default)]
    events: bool,
    /// Detect candlestick patterns and include per-candle signals in response.
    /// The `patterns` array aligns 1:1 with the `candles` array; `null` means
    /// no pattern was detected on that bar.
    #[serde(default)]
    patterns: bool,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/spark
#[derive(Deserialize)]
struct SparkQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchChartsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Detect candlestick patterns and include per-candle signals in response.
    #[serde(default)]
    patterns: bool,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchDividendsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchSplitsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchCapitalGainsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchFinancialsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Statement type (required): income, balance, cashflow
    statement: String,
    #[serde(default = "default_frequency")]
    frequency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchRecommendationsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_recommendations_limit")]
    limit: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn default_recommendations_limit() -> u32 {
    10
}

#[derive(Deserialize)]
struct BatchOptionsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Expiration date (Unix timestamp, optional)
    date: Option<i64>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct BatchIndicatorsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct RangeQuery {
    #[serde(default = "default_max_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn default_max_range() -> String {
    "max".to_string()
}

#[derive(Deserialize)]
struct SearchQuery {
    /// Search query string (required)
    q: String,
    /// Maximum number of quote results (default: 6)
    #[serde(default = "default_hits")]
    quotes: u32,
    /// Maximum number of news results (default: 0 = disabled)
    #[serde(default)]
    news: u32,
    /// Enable fuzzy matching for typos (default: false)
    #[serde(default)]
    fuzzy: bool,
    /// Enable logo URLs in results (default: true)
    #[serde(default = "default_logo")]
    logo: bool,
    /// Enable research reports (default: false)
    #[serde(default)]
    research: bool,
    /// Enable cultural assets/NFT indices (default: false)
    #[serde(default)]
    cultural: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct LookupQuery {
    /// Lookup query string (required)
    q: String,
    /// Asset type filter: all, equity, mutualfund, etf, index, future, currency, cryptocurrency
    #[serde(default = "default_lookup_type")]
    #[serde(rename = "type")]
    lookup_type: String,
    /// Maximum number of results (default: 25)
    #[serde(default = "default_lookup_count")]
    count: u32,
    /// Include logo URLs (requires additional API call, default: false)
    #[serde(default)]
    logo: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn default_lookup_type() -> String {
    "all".to_string()
}

fn default_lookup_count() -> u32 {
    25
}

#[derive(Deserialize)]
struct OptionsQuery {
    date: Option<i64>, // Optional expiration timestamp
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct FinancialsQuery {
    /// Frequency: annual or quarterly (default: annual)
    #[serde(default = "default_frequency")]
    frequency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct HoldersQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct AnalysisQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct NewsQuery {
    /// Maximum number of articles to return (default: 10)
    #[serde(default = "default_news_count")]
    count: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn default_news_count() -> u32 {
    10
}

fn default_frequency() -> String {
    "annual".to_string()
}

fn parse_frequency(s: &str) -> Frequency {
    match s.to_lowercase().as_str() {
        "quarterly" | "q" => Frequency::Quarterly,
        _ => Frequency::Annual,
    }
}

// EDGAR query structs
#[derive(Deserialize)]
struct EdgarSearchQuery {
    /// Search query string (required)
    q: String,
    /// Comma-separated form types (e.g., "10-K,10-Q")
    forms: Option<String>,
    /// Start date in YYYY-MM-DD format
    start_date: Option<String>,
    /// End date in YYYY-MM-DD format
    end_date: Option<String>,
    /// Pagination offset (default: 0)
    from: Option<usize>,
    /// Page size (default: 100, max: 100)
    size: Option<usize>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct EdgarFieldsQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/crypto/coins
#[derive(Deserialize)]
struct CryptoCoinsQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    vs_currency: String,
    /// Number of coins to return (default: 50)
    #[serde(default = "default_crypto_count")]
    count: usize,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn default_vs_currency() -> String {
    "usd".to_string()
}

fn default_crypto_count() -> usize {
    50
}

/// Query parameters for /v2/crypto/coins/{id}
#[derive(Deserialize)]
struct CryptoCoinQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    vs_currency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/feeds
#[derive(Deserialize)]
struct FeedsQuery {
    /// Comma-separated source slugs (see `FeedSourceName` for valid values)
    sources: Option<String>,
    /// SEC form type for sec-filings source (e.g., "10-K", "8-K", default: "10-K")
    form_type: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Canonical slug identifiers accepted by the `/v2/feeds` `sources` query parameter.
///
/// Multiple aliases are accepted (e.g. `"ft"`, `"financial-times"`, `"financialtimes"`), but the
/// primary slug listed here is what appears in OpenAPI documentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeedSourceName {
    FederalReserve,
    Sec,
    SecFilings,
    MarketWatch,
    Cnbc,
    Bloomberg,
    FinancialTimes,
    Nyt,
    Guardian,
    Investing,
    Bea,
    Ecb,
    Cfpb,
    Wsj,
    Fortune,
    BusinessWire,
    CoinDesk,
    CoinTelegraph,
    TechCrunch,
    HackerNews,
    OilPrice,
    CalculatedRisk,
    Scmp,
    NikkeiAsia,
    BankOfEngland,
    VentureBeat,
    YCombinator,
    TheEconomist,
    FinancialPost,
    FtLex,
    RitholtzBigPicture,
}

impl FeedSourceName {
    fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "federal-reserve" | "federalreserve" => Some(Self::FederalReserve),
            "sec" => Some(Self::Sec),
            "sec-filings" | "secfilings" => Some(Self::SecFilings),
            "marketwatch" => Some(Self::MarketWatch),
            "cnbc" => Some(Self::Cnbc),
            "bloomberg" => Some(Self::Bloomberg),
            "ft" | "financial-times" | "financialtimes" => Some(Self::FinancialTimes),
            "nyt" | "nyt-business" | "nytbusiness" => Some(Self::Nyt),
            "guardian" | "guardian-business" | "guardianbusiness" => Some(Self::Guardian),
            "investing" | "investing-com" | "investingcom" => Some(Self::Investing),
            "bea" => Some(Self::Bea),
            "ecb" => Some(Self::Ecb),
            "cfpb" => Some(Self::Cfpb),
            "wsj" | "wsj-markets" | "wsjmarkets" => Some(Self::Wsj),
            "fortune" => Some(Self::Fortune),
            "businesswire" | "business-wire" => Some(Self::BusinessWire),
            "coindesk" | "coin-desk" => Some(Self::CoinDesk),
            "cointelegraph" | "coin-telegraph" => Some(Self::CoinTelegraph),
            "techcrunch" | "tech-crunch" => Some(Self::TechCrunch),
            "hackernews" | "hacker-news" | "hn" => Some(Self::HackerNews),
            "oilprice" | "oil-price" => Some(Self::OilPrice),
            "calculated-risk" | "calculatedrisk" => Some(Self::CalculatedRisk),
            "scmp" | "south-china-morning-post" => Some(Self::Scmp),
            "nikkei" | "nikkei-asia" | "nikkeiasia" => Some(Self::NikkeiAsia),
            "boe" | "bank-of-england" | "bankofengland" => Some(Self::BankOfEngland),
            "venturebeat" | "venture-beat" => Some(Self::VentureBeat),
            "yc" | "ycombinator" | "y-combinator" => Some(Self::YCombinator),
            "economist" | "the-economist" => Some(Self::TheEconomist),
            "financial-post" | "financialpost" => Some(Self::FinancialPost),
            "ft-lex" | "ftlex" | "lex" => Some(Self::FtLex),
            "ritholtz" | "big-picture" | "bigpicture" => Some(Self::RitholtzBigPicture),
            _ => None,
        }
    }

    fn into_feed_source(self, form_type: Option<&str>) -> FeedSource {
        match self {
            Self::FederalReserve => FeedSource::FederalReserve,
            Self::Sec => FeedSource::SecPressReleases,
            Self::SecFilings => FeedSource::SecFilings(form_type.unwrap_or("10-K").to_string()),
            Self::MarketWatch => FeedSource::MarketWatch,
            Self::Cnbc => FeedSource::Cnbc,
            Self::Bloomberg => FeedSource::Bloomberg,
            Self::FinancialTimes => FeedSource::FinancialTimes,
            Self::Nyt => FeedSource::NytBusiness,
            Self::Guardian => FeedSource::GuardianBusiness,
            Self::Investing => FeedSource::Investing,
            Self::Bea => FeedSource::Bea,
            Self::Ecb => FeedSource::Ecb,
            Self::Cfpb => FeedSource::Cfpb,
            Self::Wsj => FeedSource::WsjMarkets,
            Self::Fortune => FeedSource::Fortune,
            Self::BusinessWire => FeedSource::BusinessWire,
            Self::CoinDesk => FeedSource::CoinDesk,
            Self::CoinTelegraph => FeedSource::CoinTelegraph,
            Self::TechCrunch => FeedSource::TechCrunch,
            Self::HackerNews => FeedSource::HackerNews,
            Self::OilPrice => FeedSource::OilPrice,
            Self::CalculatedRisk => FeedSource::CalculatedRisk,
            Self::Scmp => FeedSource::Scmp,
            Self::NikkeiAsia => FeedSource::NikkeiAsia,
            Self::BankOfEngland => FeedSource::BankOfEngland,
            Self::VentureBeat => FeedSource::VentureBeat,
            Self::YCombinator => FeedSource::YCombinator,
            Self::TheEconomist => FeedSource::TheEconomist,
            Self::FinancialPost => FeedSource::FinancialPost,
            Self::FtLex => FeedSource::FtLex,
            Self::RitholtzBigPicture => FeedSource::RitholtzBigPicture,
        }
    }

    const ALL_SLUGS: &'static [&'static str] = &[
        "federal-reserve",
        "sec",
        "sec-filings",
        "marketwatch",
        "cnbc",
        "bloomberg",
        "ft",
        "nyt",
        "guardian",
        "investing",
        "bea",
        "ecb",
        "cfpb",
        "wsj",
        "fortune",
        "businesswire",
        "coindesk",
        "cointelegraph",
        "techcrunch",
        "hackernews",
        "oilprice",
        "calculated-risk",
        "scmp",
        "nikkei",
        "boe",
        "venturebeat",
        "yc",
        "economist",
        "financial-post",
        "ft-lex",
        "ritholtz",
    ];
}

/// Query parameters for /v2/fred/treasury-yields
#[derive(Deserialize)]
struct TreasuryYieldsQuery {
    /// Calendar year (default: current year)
    year: Option<u32>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/risk/{symbol}
#[derive(Deserialize)]
struct RiskQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Optional benchmark symbol for beta calculation (e.g., "SPY")
    benchmark: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

fn parse_statement_type(s: &str) -> Option<StatementType> {
    match s.to_lowercase().as_str() {
        "income" => Some(StatementType::Income),
        "balance" => Some(StatementType::Balance),
        "cashflow" | "cash-flow" => Some(StatementType::CashFlow),
        _ => None,
    }
}

// HolderType and AnalysisType are defined in the service layer
use services::analysis::AnalysisType;
use services::holders::HolderType;

#[derive(Deserialize)]
struct ScreenersQuery {
    #[serde(default = "default_screeners_count")]
    count: u32,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct SectorQuery {
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
struct EarningsTranscriptQuery {
    /// Fiscal quarter (Q1, Q2, Q3, Q4). If not provided, returns latest.
    quarter: Option<String>,
    /// Fiscal year. If not provided with quarter, returns latest.
    year: Option<i32>,
}

#[derive(Deserialize)]
struct EarningsTranscriptsQuery {
    /// Maximum number of transcripts to return. If not provided, returns all.
    limit: Option<usize>,
}

fn default_screeners_count() -> u32 {
    std::env::var("SCREENERS_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::SCREENERS_COUNT)
}

fn default_limit() -> u32 {
    std::env::var("RECOMMENDATIONS_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::SIMILAR_STOCKS_LIMIT)
}

fn default_hits() -> u32 {
    std::env::var("SEARCH_HITS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::SEARCH_HITS)
}

fn default_logo() -> bool {
    true
}

fn default_interval() -> String {
    std::env::var("DEFAULT_INTERVAL").unwrap_or_else(|_| defaults::DEFAULT_INTERVAL.to_string())
}

fn default_range() -> String {
    std::env::var("DEFAULT_RANGE").unwrap_or_else(|_| defaults::DEFAULT_RANGE.to_string())
}

// ===== BATCH ENDPOINTS =====

/// GET /v2/charts?symbols=<csv>&interval=<str>&range=<str>&patterns=<bool>
async fn get_batch_charts(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchChartsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch charts for {} symbols (interval={}, range={}, patterns={})",
        symbols.len(),
        params.interval,
        params.range,
        params.patterns,
    );

    match services::chart::get_batch_charts(&state.cache, symbols, interval, range, params.patterns)
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch charts error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/dividends?symbols=<csv>&range=<str>
async fn get_batch_dividends(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchDividendsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch dividends for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    match services::events::get_batch_dividends(&state.cache, symbols, range, &params.range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch dividends error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/splits?symbols=<csv>&range=<str>
async fn get_batch_splits(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchSplitsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch splits for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    match services::events::get_batch_splits(&state.cache, symbols, range, &params.range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch splits error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/capital-gains?symbols=<csv>&range=<str>
async fn get_batch_capital_gains(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchCapitalGainsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch capital gains for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    match services::events::get_batch_capital_gains(&state.cache, symbols, range, &params.range)
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch capital gains error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/financials?symbols=<csv>&statement=<str>&frequency=<str>
async fn get_batch_financials(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchFinancialsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();

    let statement_type = match parse_statement_type(&params.statement) {
        Some(st) => st,
        None => {
            let error = serde_json::json!({
                "error": format!("Invalid statement type: '{}'. Valid types: income, balance, cashflow", params.statement),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    let frequency = parse_frequency(&params.frequency);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch financials for {} symbols (statement={}, frequency={})",
        symbols.len(),
        params.statement,
        params.frequency
    );

    match services::financials::get_batch_financials(
        &state.cache,
        symbols,
        statement_type,
        &params.statement,
        frequency,
        &params.frequency,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch financials error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/recommendations?symbols=<csv>&limit=<u32>
async fn get_batch_recommendations(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchRecommendationsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch recommendations for {} symbols (limit={})",
        symbols.len(),
        params.limit
    );

    match services::analysis::get_batch_recommendations(&state.cache, symbols, params.limit).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch recommendations error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/options?symbols=<csv>&date=<i64>
async fn get_batch_options(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchOptionsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch options for {} symbols (date={:?})",
        symbols.len(),
        params.date
    );

    match services::options::get_batch_options(&state.cache, symbols, params.date).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch options error: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/indicators?symbols=<csv>&interval=<str>&range=<str>
async fn get_batch_indicators(
    Extension(state): Extension<AppState>,
    Query(params): Query<BatchIndicatorsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching batch indicators for {} symbols (interval={}, range={})",
        symbols.len(),
        params.interval,
        params.range
    );

    match services::indicators::get_batch_indicators(
        &state.cache,
        symbols,
        interval,
        &params.interval,
        range,
        &params.range,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Batch indicators error: {}", e);
            into_error_response(e)
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    init_tracing();

    // Initialize metrics
    metrics::init();

    info!("Finance Query server initializing...");

    // Build application with routes
    let app = create_app().await;

    // Determine server address
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(defaults::SERVER_PORT);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("🚀 Starting Finance Query on {}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn create_app() -> Router {
    // Initialize Redis cache (optional - falls back gracefully if not configured)
    let redis_url = std::env::var("REDIS_URL").ok();
    let cache = Cache::new(redis_url.as_deref()).await;

    // Initialize EDGAR client (optional - requires contact email)
    if let Ok(email) = std::env::var("EDGAR_EMAIL") {
        match finance_query::edgar::init_with_config(
            email,
            "finance-query-server",
            std::time::Duration::from_secs(30),
        ) {
            Ok(_) => info!("EDGAR client initialized"),
            Err(e) => warn!("Failed to initialize EDGAR client: {}", e),
        }
    } else {
        info!("EDGAR client not configured (set EDGAR_EMAIL to enable)");
    }

    // Initialize FRED client (optional - requires API key from stlouisfed.org)
    if let Ok(key) = std::env::var("FRED_API_KEY") {
        match finance_query::fred::init(key) {
            Ok(_) => info!("FRED client initialized"),
            Err(e) => warn!("Failed to initialize FRED client: {}", e),
        }
    } else {
        info!("FRED client not configured (set FRED_API_KEY to enable)");
    }

    // Configure rate limiting
    let rate_limit_config = RateLimitConfig::from_env();
    let rate_limiter = RateLimiterState::new(rate_limit_config.clone());
    info!(
        "Rate limiting enabled: {} requests/minute",
        rate_limit_config.requests_per_minute
    );

    let state = AppState {
        cache,
        stream_hub: StreamHub::new(),
    };

    // Build GraphQL schema (shares AppState with REST handlers).
    let schema = graphql::build_schema(state.clone());

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    // Build router with routes
    Router::new()
        // Nest all API routes under /v2
        .nest("/v2", api_routes())
        // GraphQL endpoints at root (not under /v2 — different versioning story)
        .merge(graphql::graphql_routes(schema.clone()))
        .layer(Extension(schema))
        .layer(Extension(state))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            rate_limit::rate_limit_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// Metrics middleware to track request counts and latencies
async fn metrics_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let timer = metrics::RequestTimer::new(method, path);

    let response = next.run(request).await;
    let status = response.status().as_u16();

    timer.observe(status);

    response
}

/// API routes
fn api_routes() -> Router {
    Router::new()
        // Routes are sorted alphabetically by path.
        // GET /v2/analysis/{symbol}/{analysis_type}
        .route("/analysis/{symbol}/{analysis_type}", get(get_analysis))
        // GET /v2/capital-gains/{symbol}?range=<str>
        .route("/capital-gains/{symbol}", get(get_capital_gains))
        // GET /v2/capital-gains?symbols=<csv>&range=<str>
        .route("/capital-gains", get(get_batch_capital_gains))
        // GET /v2/chart/{symbol}?interval=<str>&range=<str>&events=<bool>&patterns=<bool>
        .route("/chart/{symbol}", get(get_chart))
        // GET /v2/charts?symbols=<csv>&interval=<str>&range=<str>&patterns=<bool>
        .route("/charts", get(get_batch_charts))
        // GET /v2/crypto/coins?vs_currency=<str>&count=<u32>
        .route("/crypto/coins", get(get_crypto_coins))
        // GET /v2/crypto/coins/{id}?vs_currency=<str>
        .route("/crypto/coins/{id}", get(get_crypto_coin))
        // GET /v2/currencies
        .route("/currencies", get(get_currencies))
        // GET /v2/dividends/{symbol}?range=<str>
        .route("/dividends/{symbol}", get(get_dividends))
        // GET /v2/dividends?symbols=<csv>&range=<str>
        .route("/dividends", get(get_batch_dividends))
        // GET /v2/edgar/cik/{symbol}
        .route("/edgar/cik/{symbol}", get(get_edgar_cik))
        // GET /v2/edgar/facts/{symbol}
        .route("/edgar/facts/{symbol}", get(get_edgar_facts))
        // GET /v2/edgar/search?q=<string>&forms=<csv>&start_date=<date>&end_date=<date>
        .route("/edgar/search", get(get_edgar_search))
        // GET /v2/edgar/submissions/{symbol}
        .route("/edgar/submissions/{symbol}", get(get_edgar_submissions))
        // GET /v2/exchanges
        .route("/exchanges", get(get_exchanges))
        // GET /v2/fear-and-greed
        .route("/fear-and-greed", get(get_fear_and_greed))
        // GET /v2/feeds?sources=<csv>&form_type=<str>
        .route("/feeds", get(get_feeds))
        // GET /v2/financials/{symbol}/{statement}?frequency=<annual|quarterly>
        .route("/financials/{symbol}/{statement}", get(get_financials))
        // GET /v2/financials?symbols=<csv>&statement=<str>&frequency=<str>
        .route("/financials", get(get_batch_financials))
        // GET /v2/fred/series/{id}
        .route("/fred/series/{id}", get(get_fred_series))
        // GET /v2/fred/treasury-yields?year=<u32>
        .route("/fred/treasury-yields", get(get_fred_treasury_yields))
        // GET /v2/health - version-prefixed health check
        .route("/health", get(health_check))
        // GET /v2/holders/{symbol}/{holder_type}
        .route("/holders/{symbol}/{holder_type}", get(get_holders))
        // GET /v2/hours
        .route("/hours", get(get_hours))
        // GET /v2/indicators/{symbol}?interval=<str>&range=<str>
        .route("/indicators/{symbol}", get(get_indicators))
        // GET /v2/indicators?symbols=<csv>&interval=<str>&range=<str>
        .route("/indicators", get(get_batch_indicators))
        // GET /v2/indices?format=<raw|pretty|both>
        .route("/indices", get(get_indices))
        // GET /v2/industries/{industry}
        .route("/industries/{industry}", get(get_industry))
        // GET /v2/lookup?q=<string>&type=<string>&count=<u32>&logo=<bool>
        .route("/lookup", get(lookup))
        // GET /v2/market-summary
        .route("/market-summary", get(get_market_summary))
        // GET /v2/news?count=<u32>
        .route("/news", get(get_general_news))
        // GET /v2/news/{symbol}?count=<u32>
        .route("/news/{symbol}", get(get_news))
        // GET /v2/options/{symbol}?date=<i64>
        .route("/options/{symbol}", get(get_options))
        // GET /v2/options?symbols=<csv>&date=<i64>
        .route("/options", get(get_batch_options))
        // GET /v2/ping - version-prefixed ping
        .route("/ping", get(ping))
        // GET /v2/quote/{symbol}?logo=<bool>
        .route("/quote/{symbol}", get(get_quote))
        // GET /v2/quote-type/{symbol}
        .route("/quote-type/{symbol}", get(get_quote_type))
        // GET /v2/quotes?symbols=<csv>&logo=<bool>
        .route("/quotes", get(get_quotes))
        // GET /v2/recommendations/{symbol}?limit=<u32>
        .route("/recommendations/{symbol}", get(get_recommendations))
        // GET /v2/recommendations?symbols=<csv>&limit=<u32>
        .route("/recommendations", get(get_batch_recommendations))
        // GET /v2/risk/{symbol}?interval=<str>&range=<str>&benchmark=<str>
        .route("/risk/{symbol}", get(get_risk))
        // GET /v2/screeners/{screener}?count=<u32>
        .route("/screeners/{screener}", get(get_screeners))
        // POST /v2/screeners/custom
        .route("/screeners/custom", post(post_custom_screener))
        // GET /v2/search?q=<string>&hits=<u32>
        .route("/search", get(search))
        // GET /v2/sectors/{sector}
        .route("/sectors/{sector}", get(get_sector))
        // GET /v2/spark?symbols=<csv>&interval=<str>&range=<str>
        .route("/spark", get(get_spark))
        // GET /v2/splits/{symbol}?range=<str>
        .route("/splits/{symbol}", get(get_splits))
        // GET /v2/splits?symbols=<csv>&range=<str>
        .route("/splits", get(get_batch_splits))
        // GET /v2/stream - WebSocket real-time price streaming
        .route("/stream", get(ws_stream_handler))
        // GET /v2/transcripts/{symbol}?quarter=<str>&year=<i32>
        .route("/transcripts/{symbol}", get(get_transcript))
        // GET /v2/transcripts/{symbol}/all?limit=<usize>
        .route("/transcripts/{symbol}/all", get(get_transcripts))
        // GET /v2/trending?region=<str>
        .route("/trending", get(get_trending))
        // GET /v2/metrics - Prometheus metrics endpoint
        .route("/metrics", get(get_metrics))
}

/// GET /health
///
/// Query: (none)
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        notices: &[
            "Market data provided by Yahoo Finance. This product is not affiliated with or endorsed by Yahoo Finance or its parent company.",
            "SEC filing data sourced from the U.S. Securities and Exchange Commission EDGAR system (https://www.sec.gov). This product is not affiliated with the SEC.",
            "This product uses the FRED\u{ae} API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.",
            "U.S. Treasury yield data sourced from the U.S. Department of the Treasury (https://home.treasury.gov).",
            "Cryptocurrency data provided by CoinGecko (https://www.coingecko.com). This product is not affiliated with CoinGecko.",
            "Fear & Greed Index data provided by alternative.me (https://alternative.me).",
            "News feeds sourced from the Federal Reserve, SEC, MarketWatch, Bloomberg, Financial Times, The Guardian, NYT, Investing.com, BEA, ECB, and CFPB. Content remains the property of respective publishers.",
        ],
    };

    Json(response)
}

/// GET /ping
///
/// Query: (none)
async fn ping() -> impl IntoResponse {
    let response = PingResponse {
        message: "pong".to_string(),
    };

    Json(response)
}

/// GET /metrics
///
/// Prometheus metrics endpoint in text format
async fn get_metrics() -> impl IntoResponse {
    let metrics = metrics::gather();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics,
    )
}

/// GET /v2/quote/{symbol}
///
/// Query: `logo` (bool, default: false), `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_quote(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<QuoteQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Received quote request for symbol: {} (logo={}, format={}, fields={:?})",
        symbol,
        params.logo,
        format.as_str(),
        params.fields
    );

    match services::quote::get_quote(&state.cache, &symbol, params.logo).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch quote for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

// Helper to convert error to response
fn error_response(e: FinanceError) -> impl IntoResponse {
    let status = match e {
        FinanceError::SymbolNotFound { .. } => StatusCode::NOT_FOUND,
        FinanceError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
        FinanceError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        FinanceError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
        FinanceError::ServerError { status, .. } => {
            StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let error_response = serde_json::json!({
        "error": e.to_string(),
        "status": status.as_u16()
    });
    (status, Json(error_response))
}

/// Converts generic errors from get_or_fetch into HTTP responses
/// Attempts to downcast to FinanceError to preserve HTTP status code mapping
fn into_error_response(e: Box<dyn std::error::Error + Send + Sync>) -> axum::response::Response {
    // Try to downcast to FinanceError first to get proper HTTP status codes
    if let Some(yahoo_err) = e.downcast_ref::<FinanceError>() {
        // Track error by type
        let error_type = match yahoo_err {
            FinanceError::SymbolNotFound { .. } => "symbol_not_found",
            FinanceError::AuthenticationFailed { .. } => "authentication_failed",
            FinanceError::RateLimited { .. } => "rate_limited",
            FinanceError::Timeout { .. } => "timeout",
            FinanceError::ServerError { .. } => "server_error",
            _ => "other",
        };
        metrics::ERRORS_TOTAL.with_label_values(&[error_type]).inc();

        // Map FinanceError to appropriate HTTP status codes
        let status = match yahoo_err {
            FinanceError::SymbolNotFound { .. } => StatusCode::NOT_FOUND,
            FinanceError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
            FinanceError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            FinanceError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            FinanceError::ServerError { status, .. } => {
                StatusCode::from_u16(*status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        return (
            status,
            Json(serde_json::json!({
                "error": yahoo_err.to_string(),
                "status": status.as_u16()
            })),
        )
            .into_response();
    }

    // Fallback for other errors (e.g., serialization errors)
    metrics::ERRORS_TOTAL
        .with_label_values(&["serialization"])
        .inc();
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": e.to_string(),
            "status": 500
        })),
    )
        .into_response()
}

// Re-export parse helpers from services (shared with GraphQL)
use services::{parse_interval, parse_range};

/// GET /v2/quotes
///
/// Query: `symbols` (comma-separated, required), `logo` (bool, default: false),
///        `format` (raw|pretty|both), `fields` (comma-separated)
///
/// Uses batch fetching via Tickers for optimal performance (single API call).
async fn get_quotes(
    Extension(state): Extension<AppState>,
    Query(params): Query<QuotesQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching batch quotes for {} symbols (logo={}, format={}, fields={:?})",
        symbols.len(),
        params.logo,
        format.as_str(),
        params.fields
    );

    match services::quote::get_quotes(&state.cache, symbols, params.logo).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch batch quotes: {}", e);
            into_error_response(e)
        }
    }
}

/// Query parameters for /v2/indices
#[derive(Deserialize)]
struct IndicesQuery {
    /// Region filter: americas, europe, asia-pacific, middle-east-africa, currencies
    region: Option<String>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/indices
///
/// Returns quotes for world market indices, optionally filtered by region.
async fn get_indices(
    Extension(state): Extension<AppState>,
    Query(params): Query<IndicesQuery>,
) -> impl IntoResponse {
    use finance_query::IndicesRegion;

    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    let region = params.region.as_deref().and_then(IndicesRegion::parse);

    info!(
        "Fetching indices (region={}, format={}, fields={:?})",
        region.map(|r| r.as_str()).unwrap_or("all"),
        format.as_str(),
        params.fields
    );

    match services::market::get_indices(&state.cache, region).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch indices: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/recommendations/{symbol}
///
/// Query: `limit` (u32, default via `RECOMMENDATIONS_LIMIT` or server default)
async fn get_recommendations(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<RecommendationsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    let limit = params.limit;
    info!(
        "Fetching recommendations for {} (limit={}, fields={:?})",
        symbol, limit, params.fields
    );

    match services::analysis::get_recommendations(&state.cache, &symbol, limit).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch recommendations for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/chart/{symbol}
///
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`), `events` (bool, default false)
async fn get_chart(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching chart data for {} (events={}, patterns={}, fields={:?})",
        symbol, params.events, params.patterns, params.fields
    );

    match services::chart::get_chart(
        &state.cache,
        &symbol,
        interval,
        range,
        params.events,
        params.patterns,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch chart data for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/spark
///
/// Batch fetch sparkline data for multiple symbols in a single request.
/// Optimized for rendering sparkline charts with only close prices.
///
/// Query: `symbols` (comma-separated, required), `interval` (default "1d"), `range` (default "1mo")
async fn get_spark(
    Extension(state): Extension<AppState>,
    Query(params): Query<SparkQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching spark data for {} symbols (interval={}, range={})",
        symbols.len(),
        params.interval,
        params.range
    );

    match services::chart::get_spark(&state.cache, symbols, interval, range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch spark data: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/dividends/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_dividends(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching dividends for {} (range={:?})", symbol, range);

    match services::events::get_dividends(&state.cache, &symbol, range, &params.range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch dividends for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/splits/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_splits(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching splits for {} (range={:?})", symbol, range);

    match services::events::get_splits(&state.cache, &symbol, range, &params.range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch splits for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/capital-gains/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_capital_gains(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching capital gains for {} (range={:?})", symbol, range);

    match services::events::get_capital_gains(&state.cache, &symbol, range, &params.range).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch capital gains for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/indicators/{symbol}
///
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`)
async fn get_indicators(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Calculating indicators for {} with interval={:?}, range={:?} (fields={:?})",
        symbol, interval, range, params.fields
    );

    match services::indicators::get_indicators(
        &state.cache,
        &symbol,
        interval,
        &params.interval,
        range,
        &params.range,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to calculate indicators for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/search
///
/// Search for quotes, news, and research reports
///
/// Query parameters:
/// - `q` (string, required): Search query
/// - `quotes` (u32, default: 6): Maximum quote results
/// - `news` (u32, default: 0): Maximum news results
/// - `fuzzy` (bool, default: false): Enable fuzzy matching for typos
/// - `logo` (bool, default: true): Include logo URLs
/// - `research` (bool, default: false): Include research reports
/// - `cultural` (bool, default: false): Include cultural assets (NFT indices)
/// - `region` (string, optional): Region code for lang/localization (e.g., "US", "JP")
async fn search(
    Extension(state): Extension<AppState>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Searching for: {} (quotes={}, news={}, logo={}, research={}, cultural={}, region={:?})",
        params.q,
        params.quotes,
        params.news,
        params.logo,
        params.research,
        params.cultural,
        params.region
    );

    let region = params.region.as_deref().and_then(parse_region);

    match services::search::search(
        &state.cache,
        &params.q,
        params.quotes,
        params.news,
        services::search::SearchFlags {
            fuzzy: params.fuzzy,
            logo: params.logo,
            research: params.research,
            cultural: params.cultural,
        },
        region,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Search failed for {}: {}", params.q, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/lookup
///
/// Type-filtered symbol lookup. Unlike search, lookup specializes in discovering tickers
/// filtered by asset type (equity, ETF, mutual fund, index, future, currency, cryptocurrency).
///
/// Query parameters:
/// - `q` (string, required): Lookup query
/// - `type` (string, default: "all"): Asset type filter
/// - `count` (u32, default: 25): Maximum results
/// - `logo` (bool, default: false): Include logo URLs (requires extra API call)
/// - `region` (string, optional): Region code for lang/localization (e.g., "US", "JP")
async fn lookup(
    Extension(state): Extension<AppState>,
    Query(params): Query<LookupQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Looking up: {} (type={}, count={}, logo={}, region={:?})",
        params.q, params.lookup_type, params.count, params.logo, params.region
    );

    // Parse lookup type
    let lookup_type = match params.lookup_type.to_lowercase().as_str() {
        "all" => finance::LookupType::All,
        "equity" => finance::LookupType::Equity,
        "mutualfund" => finance::LookupType::MutualFund,
        "etf" => finance::LookupType::Etf,
        "index" => finance::LookupType::Index,
        "future" => finance::LookupType::Future,
        "currency" => finance::LookupType::Currency,
        "cryptocurrency" => finance::LookupType::Cryptocurrency,
        _ => finance::LookupType::All,
    };

    let region = params.region.as_deref().and_then(parse_region);

    match services::search::lookup(
        &state.cache,
        &params.q,
        lookup_type,
        params.count,
        params.logo,
        region,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Lookup failed for {}: {}", params.q, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/news
///
/// Returns general market news
async fn get_general_news(
    Extension(state): Extension<AppState>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching general market news (fields={:?})", params.fields);

    match services::news::get_general_news(&state.cache, params.count as usize).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch general news: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/news/{symbol}
///
/// Returns news for a specific symbol
async fn get_news(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching news for {} (fields={:?})", symbol, params.fields);

    match services::news::get_news(&state.cache, &symbol, params.count as usize).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch news for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/options/{symbol}
///
/// Query: `date` (i64, optional expiration timestamp)
async fn get_options(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<OptionsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching options for {} (fields={:?})",
        symbol, params.fields
    );

    match services::options::get_options(&state.cache, &symbol, params.date).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch options for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/financials/{symbol}/{statement}
///
/// Path params:
/// - `statement`: income, balance, or cashflow
///
/// Query: `frequency` (annual|quarterly, default: annual)
async fn get_financials(
    Extension(state): Extension<AppState>,
    Path((symbol, statement)): Path<(String, String)>,
    Query(params): Query<FinancialsQuery>,
) -> impl IntoResponse {
    let frequency = parse_frequency(&params.frequency);
    let fields = parse_fields(params.fields.as_deref());

    let statement_type = match parse_statement_type(&statement) {
        Some(st) => st,
        None => {
            let error = serde_json::json!({
                "error": format!("Invalid statement type: '{}'. Valid types: income, balance, cashflow", statement),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!(
        "Fetching {} {} financials for {} (fields={:?})",
        params.frequency, statement, symbol, params.fields
    );

    match services::financials::get_financials(
        &state.cache,
        &symbol,
        statement_type,
        &statement,
        frequency,
        &params.frequency,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to fetch financials for {} {}: {}",
                symbol, statement, e
            );
            into_error_response(e)
        }
    }
}

/// Query parameters for /v2/hours
#[derive(Deserialize)]
struct HoursQuery {
    /// Region code (e.g., "US", "JP", "GB"). Defaults to US if not specified.
    region: Option<String>,
}

/// GET /v2/hours
///
/// Query: `region` (string, optional - e.g., "US", "JP", "GB")
async fn get_hours(
    Extension(state): Extension<AppState>,
    Query(params): Query<HoursQuery>,
) -> impl IntoResponse {
    let region_display = params.region.as_deref().unwrap_or("US");
    info!("Fetching market hours for region: {}", region_display);

    match services::metadata::get_hours(&state.cache, params.region.as_deref()).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch market hours for {}: {}", region_display, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/quote-type/{symbol}
///
/// Query: (none)
async fn get_quote_type(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
) -> impl IntoResponse {
    info!("Fetching quote type for {}", symbol);

    match services::metadata::get_quote_type(&state.cache, &symbol).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch quote type for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/holders/{symbol}/{holder_type}
///
/// Path params:
/// - `holder_type`: major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster
///
/// Query: `fields` (comma-separated, optional)
async fn get_holders(
    Extension(state): Extension<AppState>,
    Path((symbol, holder_type)): Path<(String, String)>,
    Query(params): Query<HoldersQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    let ht = match HolderType::from_str(&holder_type) {
        Some(t) => t,
        None => {
            let error = serde_json::json!({
                "error": format!("Invalid holder type: '{}'. Valid types: {}", holder_type, HolderType::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!(
        "Fetching {} holders for {} (fields={:?})",
        holder_type, symbol, params.fields
    );

    match services::holders::get_holders(&state.cache, &symbol, ht, &holder_type).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to fetch {} holders for {}: {}",
                holder_type, symbol, e
            );
            into_error_response(e)
        }
    }
}

/// GET /v2/analysis/{symbol}/{analysis_type}
///
/// Path params:
/// - `analysis_type`: recommendations, upgrades-downgrades, earnings-estimate, earnings-history
///
/// Query: (none)
async fn get_analysis(
    Extension(state): Extension<AppState>,
    Path((symbol, analysis_type)): Path<(String, String)>,
    Query(params): Query<AnalysisQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    let at = match AnalysisType::from_str(&analysis_type) {
        Some(t) => t,
        None => {
            let error = serde_json::json!({
                "error": format!("Invalid analysis type: '{}'. Valid types: {}", analysis_type, AnalysisType::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!(
        "Fetching {} analysis for {} (fields={:?})",
        analysis_type, symbol, params.fields
    );

    match services::analysis::get_analysis(&state.cache, &symbol, at, &analysis_type).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to fetch {} analysis for {}: {}",
                analysis_type, symbol, e
            );
            into_error_response(e)
        }
    }
}

/// GET /v2/screeners/{screener}
///
/// Path params:
/// - `screener`: One of 15 predefined screener identifiers (kebab-case)
///   - Equity: aggressive-small-caps, day-gainers, day-losers, growth-technology-stocks,
///     most-actives, most-shorted-stocks, small-cap-gainers, undervalued-growth-stocks,
///     undervalued-large-caps
///   - Fund: conservative-foreign-funds, high-yield-bond, portfolio-anchors,
///     solid-large-growth-funds, solid-midcap-growth-funds, top-mutual-funds
///
/// Query: `count` (u32, default 25, max 250), `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_screeners(
    Extension(state): Extension<AppState>,
    Path(screener): Path<String>,
    Query(params): Query<ScreenersQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    let st = match screener.parse::<Screener>() {
        Ok(t) => t,
        Err(_) => {
            let error = serde_json::json!({
                "error": format!("Invalid screener: '{}'. Valid types: {}", screener, Screener::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!(
        "Fetching {} screener (count={}, format={:?}, fields={:?})",
        screener, params.count, params.format, params.fields
    );

    match services::market::get_screener(&state.cache, st, &screener, params.count).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} screener: {}", screener, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/sectors/{sector}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_sector(
    Extension(state): Extension<AppState>,
    Path(sector): Path<String>,
    Query(params): Query<SectorQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    let st = match sector.parse::<Sector>() {
        Ok(t) => t,
        Err(_) => {
            let error = serde_json::json!({
                "error": format!("Invalid sector: '{}'. Valid types: {}", sector, Sector::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!(
        "Fetching {} sector (format={:?}, fields={:?})",
        sector, params.format, params.fields
    );

    match services::market::get_sector(&state.cache, st, &sector).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} sector: {}", sector, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/industries/{industry}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_industry(
    Extension(state): Extension<AppState>,
    Path(industry): Path<String>,
    Query(params): Query<SectorQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching {} industry (format={:?}, fields={:?})",
        industry, params.format, params.fields
    );

    match services::market::get_industry(&state.cache, &industry).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} industry: {}", industry, e);
            into_error_response(e)
        }
    }
}

/// Request body for custom screener endpoint
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CustomScreenerRequest {
    /// Number of results (default: 25, max: 250)
    #[serde(default = "default_screeners_count")]
    size: u32,
    /// Pagination offset (default: 0)
    #[serde(default)]
    offset: u32,
    /// Sort direction: "ASC" or "DESC" (default: DESC)
    #[serde(default)]
    sort_type: Option<String>,
    /// Field to sort by (default: intradaymarketcap)
    sort_field: Option<String>,
    /// Quote type: EQUITY, ETF, MUTUALFUND, etc. (default: EQUITY)
    quote_type: Option<String>,
    /// Filter conditions
    #[serde(default)]
    filters: Vec<FilterCondition>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// A single filter condition
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilterCondition {
    /// Field name (e.g., "region", "avgdailyvol3m", "intradaymarketcap")
    field: String,
    /// Operator: eq, gt, gte, lt, lte, btwn
    operator: String,
    /// Value(s) for the condition
    value: serde_json::Value,
}

/// POST /v2/screeners/custom
///
/// Execute a custom screener query with flexible filtering.
///
/// Request body:
/// ```json
/// {
///   "size": 25,
///   "offset": 0,
///   "sortType": "DESC",
///   "sortField": "intradaymarketcap",
///   "quoteType": "EQUITY",
///   "filters": [
///     {"field": "region", "operator": "eq", "value": "us"},
///     {"field": "avgdailyvol3m", "operator": "gt", "value": 200000}
///   ],
///   "format": "raw",
///   "fields": "symbol,shortName,regularMarketPrice"
/// }
/// ```
async fn post_custom_screener(Json(body): Json<CustomScreenerRequest>) -> impl IntoResponse {
    let format = parse_format(body.format.as_deref());
    let fields = parse_fields(body.fields.as_deref());

    let quote_type = body
        .quote_type
        .as_deref()
        .and_then(|s| s.parse::<QuoteType>().ok())
        .unwrap_or_default();

    let sort_ascending = body
        .sort_type
        .as_deref()
        .map(|s| s.to_lowercase() == "asc")
        .unwrap_or(false);

    let filter_count = body.filters.len();

    match quote_type {
        QuoteType::Equity => {
            let result = build_and_run_equity_screener(body, sort_ascending, filter_count).await;
            match result {
                Ok(data) => {
                    let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
                    let response = apply_transforms(json, format, fields.as_ref());
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(ServerScreenerError::InvalidField(msg)) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": msg, "status": 400})),
                )
                    .into_response(),
                Err(ServerScreenerError::InvalidOperator(msg)) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": msg, "status": 400})),
                )
                    .into_response(),
                Err(ServerScreenerError::Finance(e)) => {
                    error!("Failed to execute custom screener: {}", e);
                    error_response(e).into_response()
                }
            }
        }
        QuoteType::MutualFund => {
            let result = build_and_run_fund_screener(body, sort_ascending, filter_count).await;
            match result {
                Ok(data) => {
                    let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
                    let response = apply_transforms(json, format, fields.as_ref());
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(ServerScreenerError::InvalidField(msg)) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": msg, "status": 400})),
                )
                    .into_response(),
                Err(ServerScreenerError::InvalidOperator(msg)) => (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": msg, "status": 400})),
                )
                    .into_response(),
                Err(ServerScreenerError::Finance(e)) => {
                    error!("Failed to execute custom screener: {}", e);
                    error_response(e).into_response()
                }
            }
        }
    }
}

enum ServerScreenerError {
    InvalidField(String),
    InvalidOperator(String),
    Finance(FinanceError),
}

impl From<FinanceError> for ServerScreenerError {
    fn from(e: FinanceError) -> Self {
        ServerScreenerError::Finance(e)
    }
}

async fn build_and_run_equity_screener(
    body: CustomScreenerRequest,
    sort_ascending: bool,
    filter_count: usize,
) -> Result<finance_query::ScreenerResults, ServerScreenerError> {
    let mut query = EquityScreenerQuery::new()
        .size(body.size)
        .offset(body.offset);

    if let Some(sort_field_str) = body.sort_field {
        match sort_field_str.parse::<EquityField>() {
            Ok(field) => {
                query = query.sort_by(field, sort_ascending);
            }
            Err(_) => {
                return Err(ServerScreenerError::InvalidField(format!(
                    "Unknown equity sort field: '{}'. Use EquityField enum values.",
                    sort_field_str
                )));
            }
        }
    }

    for filter in body.filters {
        let condition = build_equity_condition(&filter)?;
        query = query.add_condition(condition);
    }

    info!(
        "Executing custom equity screener (size={}, filters={})",
        body.size, filter_count
    );

    Ok(finance::custom_screener(query).await?)
}

async fn build_and_run_fund_screener(
    body: CustomScreenerRequest,
    sort_ascending: bool,
    filter_count: usize,
) -> Result<finance_query::ScreenerResults, ServerScreenerError> {
    let mut query = FundScreenerQuery::new().size(body.size).offset(body.offset);

    if let Some(sort_field_str) = body.sort_field {
        match sort_field_str.parse::<FundField>() {
            Ok(field) => {
                query = query.sort_by(field, sort_ascending);
            }
            Err(_) => {
                return Err(ServerScreenerError::InvalidField(format!(
                    "Unknown fund sort field: '{}'. Use FundField enum values.",
                    sort_field_str
                )));
            }
        }
    }

    for filter in body.filters {
        let condition = build_fund_condition(&filter)?;
        query = query.add_condition(condition);
    }

    info!(
        "Executing custom fund screener (size={}, filters={})",
        body.size, filter_count
    );

    Ok(finance::custom_screener(query).await?)
}

fn build_equity_condition(
    filter: &FilterCondition,
) -> Result<finance_query::QueryCondition<EquityField>, ServerScreenerError> {
    let field = filter.field.parse::<EquityField>().map_err(|_| {
        ServerScreenerError::InvalidField(format!(
            "Unknown equity field: '{}'. See EquityField for valid values.",
            filter.field
        ))
    })?;
    build_condition_from_filter(field, filter)
}

fn build_fund_condition(
    filter: &FilterCondition,
) -> Result<finance_query::QueryCondition<FundField>, ServerScreenerError> {
    let field = filter.field.parse::<FundField>().map_err(|_| {
        ServerScreenerError::InvalidField(format!(
            "Unknown fund field: '{}'. See FundField for valid values.",
            filter.field
        ))
    })?;
    build_condition_from_filter(field, filter)
}

fn build_condition_from_filter<
    F: finance_query::ScreenerField + finance_query::ScreenerFieldExt,
>(
    field: F,
    filter: &FilterCondition,
) -> Result<finance_query::QueryCondition<F>, ServerScreenerError> {
    let op = filter.operator.to_lowercase();
    match op.as_str() {
        "eq" | "=" | "==" => match &filter.value {
            serde_json::Value::String(s) => Ok(field.eq_str(s.clone())),
            serde_json::Value::Number(n) => Ok(field.eq_num(n.as_f64().unwrap_or(0.0))),
            _ => Ok(field.eq_str(filter.value.to_string())),
        },
        "gt" | ">" => {
            let v = numeric_value(&filter.value)?;
            Ok(field.gt(v))
        }
        "gte" | ">=" => {
            let v = numeric_value(&filter.value)?;
            Ok(field.gte(v))
        }
        "lt" | "<" => {
            let v = numeric_value(&filter.value)?;
            Ok(field.lt(v))
        }
        "lte" | "<=" => {
            let v = numeric_value(&filter.value)?;
            Ok(field.lte(v))
        }
        "btwn" | "between" => {
            let (min, max) = between_values(&filter.value)?;
            Ok(field.between(min, max))
        }
        _ => Err(ServerScreenerError::InvalidOperator(format!(
            "Invalid operator: '{}'. Valid: eq, gt, gte, lt, lte, btwn",
            filter.operator
        ))),
    }
}

fn numeric_value(v: &serde_json::Value) -> Result<f64, ServerScreenerError> {
    v.as_f64().ok_or_else(|| {
        ServerScreenerError::InvalidField(format!("Expected a numeric value, got: {}", v))
    })
}

fn between_values(v: &serde_json::Value) -> Result<(f64, f64), ServerScreenerError> {
    match v {
        serde_json::Value::Array(arr) if arr.len() == 2 => {
            let min = arr[0].as_f64().ok_or_else(|| {
                ServerScreenerError::InvalidField("BTWN first value must be numeric".to_string())
            })?;
            let max = arr[1].as_f64().ok_or_else(|| {
                ServerScreenerError::InvalidField("BTWN second value must be numeric".to_string())
            })?;
            Ok((min, max))
        }
        _ => Err(ServerScreenerError::InvalidField(
            "BTWN operator requires an array of exactly 2 numeric values: [min, max]".to_string(),
        )),
    }
}

/// Initialize tracing/logging with JSON or text format
fn init_tracing() {
    let log_level = std::env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase();

    let log_format = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "json".to_string())
        .to_lowercase();

    let filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!(
            "finance_query={},server={},tower_http=debug,axum::rejection=trace",
            log_level, log_level
        )
        .into()
    });

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

/// GET /v2/transcripts/{symbol}
///
/// Returns earnings transcript for a symbol.
/// Query params:
/// - `quarter` (optional): Fiscal quarter (Q1, Q2, Q3, Q4). Defaults to latest.
/// - `year` (optional): Fiscal year. Defaults to latest.
async fn get_transcript(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptQuery>,
) -> impl IntoResponse {
    info!(
        "Fetching transcript for {} (quarter={:?}, year={:?})",
        symbol, params.quarter, params.year
    );

    match services::transcripts::get_transcript(
        &state.cache,
        &symbol,
        params.quarter.as_deref(),
        params.year,
    )
    .await
    {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch transcript for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/transcripts/{symbol}/all
///
/// Returns all earnings transcripts for a symbol.
/// Query params:
/// - `limit` (optional): Maximum number of transcripts to return.
async fn get_transcripts(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptsQuery>,
) -> impl IntoResponse {
    info!(
        "Fetching all transcripts for {} (limit={:?})",
        symbol, params.limit
    );

    match services::transcripts::get_transcripts(&state.cache, &symbol, params.limit).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch transcripts for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/currencies
///
/// Returns available currencies from Yahoo Finance.
async fn get_currencies(Extension(state): Extension<AppState>) -> impl IntoResponse {
    info!("Fetching currencies");

    match services::metadata::get_currencies(&state.cache).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch currencies: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/exchanges
///
/// Returns list of supported exchanges with their suffixes and data providers.
async fn get_exchanges(Extension(state): Extension<AppState>) -> impl IntoResponse {
    info!("Fetching exchanges");

    match services::metadata::get_exchanges(&state.cache).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch exchanges: {}", e);
            into_error_response(e)
        }
    }
}

/// Query parameters for /v2/market-summary
#[derive(Deserialize)]
struct MarketSummaryQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    region: Option<String>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/market-summary
///
/// Returns market summary with major indices, currencies, and commodities.
async fn get_market_summary(
    Extension(state): Extension<AppState>,
    Query(params): Query<MarketSummaryQuery>,
) -> impl IntoResponse {
    let region = params.region.as_deref().and_then(parse_region);
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching market summary (region={:?}, format={}, fields={:?})",
        region,
        format.as_str(),
        params.fields
    );

    match services::market::get_market_summary(&state.cache, region).await {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch market summary: {}", e);
            into_error_response(e)
        }
    }
}

/// Query parameters for /v2/trending
#[derive(Deserialize)]
struct TrendingQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    region: Option<String>,
}

/// GET /v2/trending
///
/// Returns trending tickers for a region.
async fn get_trending(
    Extension(state): Extension<AppState>,
    Query(params): Query<TrendingQuery>,
) -> impl IntoResponse {
    let region = params.region.as_deref().and_then(parse_region);

    info!("Fetching trending tickers (region={:?})", region);

    match services::market::get_trending(&state.cache, region).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch trending tickers: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/edgar/cik/{symbol}
///
/// Resolve a ticker symbol to its SEC CIK number.
/// Requires EDGAR_EMAIL environment variable to be set.
async fn get_edgar_cik(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Resolving CIK for symbol: {}", symbol);

    match services::edgar::get_cik(&state.cache, &symbol).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to resolve CIK for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/edgar/submissions/{symbol}
///
/// Fetch SEC filing history and company metadata.
/// Requires EDGAR_EMAIL environment variable to be set.
async fn get_edgar_submissions(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching EDGAR submissions for symbol: {}", symbol);

    match services::edgar::get_submissions(&state.cache, &symbol).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch EDGAR submissions for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/edgar/facts/{symbol}
///
/// Fetch structured XBRL financial data from SEC.
/// Requires EDGAR_EMAIL environment variable to be set.
async fn get_edgar_facts(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching EDGAR company facts for symbol: {}", symbol);

    match services::edgar::get_facts(&state.cache, &symbol).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch EDGAR company facts for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/edgar/search
///
/// Search SEC EDGAR filings by text content.
/// Requires EDGAR_EMAIL environment variable to be set.
///
/// Query parameters:
/// - `q`: Search query string (required)
/// - `forms`: Comma-separated form types (e.g., "10-K,10-Q")
/// - `start_date`: Start date in YYYY-MM-DD format
/// - `end_date`: End date in YYYY-MM-DD format
/// - `from`: Pagination offset (default: 0)
/// - `size`: Page size (default: 100, max: 100)
async fn get_edgar_search(
    Extension(state): Extension<AppState>,
    Query(params): Query<EdgarSearchQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Searching EDGAR: query={}, forms={:?}, start={:?}, end={:?}, from={:?}, size={:?}",
        params.q, params.forms, params.start_date, params.end_date, params.from, params.size
    );

    match services::edgar::search_edgar(
        &state.cache,
        &params.q,
        params.forms.as_deref(),
        params.start_date.as_deref(),
        params.end_date.as_deref(),
        params.from,
        params.size,
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to search EDGAR: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/fear-and-greed
///
/// Returns the CNN Fear & Greed index from alternative.me.
async fn get_fear_and_greed(Extension(state): Extension<AppState>) -> impl IntoResponse {
    info!("Fetching Fear & Greed index");

    match services::market::get_fear_and_greed(&state.cache).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch Fear & Greed index: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/fred/series/{id}
///
/// Fetch observations for a FRED data series. Requires `FRED_API_KEY` to be set.
async fn get_fred_series(
    Extension(state): Extension<AppState>,
    Path(series_id): Path<String>,
    Query(params): Query<EdgarFieldsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching FRED series: {}", series_id);

    match services::fred::get_series(&state.cache, &series_id).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch FRED series {}: {}", series_id, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/fred/treasury-yields
///
/// Query: `year` (u32, default: current year)
async fn get_fred_treasury_yields(
    Extension(state): Extension<AppState>,
    Query(params): Query<TreasuryYieldsQuery>,
) -> impl IntoResponse {
    let year = params.year.unwrap_or_else(|| {
        chrono::Utc::now()
            .format("%Y")
            .to_string()
            .parse()
            .unwrap_or(2025)
    });
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching Treasury yields for year {}", year);

    match services::fred::get_treasury_yields(&state.cache, year).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch Treasury yields for {}: {}", year, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/crypto/coins
///
/// Query: `vs_currency` (str, default "usd"), `count` (u32, default 50)
async fn get_crypto_coins(
    Extension(state): Extension<AppState>,
    Query(params): Query<CryptoCoinsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching top {} crypto coins (vs {})",
        params.count, params.vs_currency
    );

    match services::crypto::get_coins(&state.cache, &params.vs_currency, params.count).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch crypto coins: {}", e);
            into_error_response(e)
        }
    }
}

/// GET /v2/crypto/coins/{id}
///
/// Query: `vs_currency` (str, default "usd")
async fn get_crypto_coin(
    Extension(state): Extension<AppState>,
    Path(coin_id): Path<String>,
    Query(params): Query<CryptoCoinQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching crypto coin: {} (vs {})",
        coin_id, params.vs_currency
    );

    match services::crypto::get_coin(&state.cache, &coin_id, &params.vs_currency).await {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch crypto coin {}: {}", coin_id, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/feeds
///
/// Query: `sources` (csv, default: all built-in), `form_type` (str, for sec-filings source)
async fn get_feeds(
    Extension(state): Extension<AppState>,
    Query(params): Query<FeedsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    let source_list = params.sources.as_deref().unwrap_or("all");
    info!("Fetching feeds (sources={})", source_list);

    let sources = match parse_feed_sources(params.sources.as_deref(), params.form_type.as_deref()) {
        Ok(s) => s,
        Err(msg) => {
            let error = serde_json::json!({ "error": msg, "status": 400 });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    match services::feeds::get_feeds(
        &state.cache,
        &sources,
        source_list,
        params.form_type.as_deref().unwrap_or(""),
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch feeds: {}", e);
            into_error_response(e)
        }
    }
}

/// Parse comma-separated source slugs into a `Vec<FeedSource>`.
///
/// Returns `Err` with a descriptive message if any slug is unrecognized (caller should 400).
/// Falls back to default sources when `sources` is `None`.
fn parse_feed_sources(
    sources: Option<&str>,
    form_type: Option<&str>,
) -> Result<Vec<FeedSource>, String> {
    let default_sources = || {
        vec![
            FeedSource::FederalReserve,
            FeedSource::SecPressReleases,
            FeedSource::MarketWatch,
            FeedSource::Bloomberg,
        ]
    };

    let Some(sources_str) = sources else {
        return Ok(default_sources());
    };

    let mut parsed = Vec::new();
    for slug in sources_str
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        match FeedSourceName::parse(slug) {
            Some(name) => parsed.push(name.into_feed_source(form_type)),
            None => {
                return Err(format!(
                    "Unknown feed source: '{}'. Valid sources: {}",
                    slug,
                    FeedSourceName::ALL_SLUGS.join(", ")
                ));
            }
        }
    }

    if parsed.is_empty() {
        Ok(default_sources())
    } else {
        Ok(parsed)
    }
}

/// GET /v2/risk/{symbol}
///
/// Query: `interval` (str, default "1d"), `range` (str, default "1y"), `benchmark` (str, optional)
async fn get_risk(
    Extension(state): Extension<AppState>,
    Path(symbol): Path<String>,
    Query(params): Query<RiskQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching risk analytics for {} (interval={:?}, range={:?}, benchmark={:?})",
        symbol, interval, range, params.benchmark
    );

    match services::risk::get_risk(
        &state.cache,
        &symbol,
        interval,
        &params.interval,
        range,
        &params.range,
        params.benchmark.as_deref(),
    )
    .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch risk analytics for {}: {}", symbol, e);
            into_error_response(e)
        }
    }
}

/// WebSocket /v2/stream
///
/// Real-time price streaming via WebSocket.
///
/// # Protocol
///
/// **Subscribe to symbols:**
/// ```json
/// {"subscribe": ["AAPL", "NVDA", "TSLA"]}
/// ```
///
/// **Unsubscribe from symbols:**
/// ```json
/// {"unsubscribe": ["AAPL"]}
/// ```
///
/// **Receive price updates:**
/// ```json
/// {
///   "id": "AAPL",
///   "price": 178.52,
///   "change": 2.34,
///   "changePercent": 1.33,
///   "time": 1703123456000,
///   "exchange": "NMS",
///   "marketHours": 2
/// }
/// ```
async fn ws_stream_handler(
    Extension(state): Extension<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Track new WebSocket connection
    metrics::WEBSOCKET_CONNECTIONS.inc();
    ws.on_upgrade(move |socket| handle_stream_socket(state, socket))
}

/// RAII guard to decrement WebSocket connection count on drop
struct ConnectionGuard;

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        metrics::WEBSOCKET_CONNECTIONS.dec();
    }
}

/// Handle the WebSocket connection for streaming
async fn handle_stream_socket(state: AppState, mut socket: WebSocket) {
    let _guard = ConnectionGuard; // Ensures connection count is decremented on exit
    info!("New streaming WebSocket connection");

    // Wait for initial subscription message
    let symbols = match wait_for_subscription(&mut socket).await {
        Some(symbols) => {
            metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
            symbols
        }
        None => {
            warn!("WebSocket closed before subscription");
            return;
        }
    };

    info!("Starting stream for symbols: {:?}", symbols);
    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(symbols.len() as f64);

    // Ref-counted subscribe (shared upstream stream).
    if let Err(e) = state.stream_hub.subscribe_symbols(&symbols).await {
        error!("Failed to create shared price stream: {}", e);
        let _ = socket
            .send(Message::Text(
                serde_json::json!({"error": e.to_string()})
                    .to_string()
                    .into(),
            ))
            .await;
        return;
    }

    let mut hub_stream = match state.stream_hub.resubscribe().await {
        Some(s) => s,
        None => {
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": "stream unavailable"})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };

    let subscriptions = Arc::new(tokio::sync::RwLock::new(
        symbols.iter().cloned().collect::<HashSet<String>>(),
    ));

    let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Message>(32);

    // Split socket for concurrent read/write
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward filtered price updates + outbound messages to client
    let subscriptions_for_send = Arc::clone(&subscriptions);
    let mut send_task = tokio::spawn(async move {
        use futures_util::stream::StreamExt;
        loop {
            tokio::select! {
                msg = out_rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if sender.send(msg).await.is_err() {
                                break;
                            }
                        }
                        None => {
                            // Control channel closed.
                            break;
                        }
                    }
                }

                maybe_price = hub_stream.next() => {
                    match maybe_price {
                        Some(price) => {
                            let should_send = {
                                let subs = subscriptions_for_send.read().await;
                                subs.contains(&price.id)
                            };

                            if should_send {
                                let json = serde_json::to_string(&price).unwrap_or_default();
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                                metrics::WEBSOCKET_MESSAGES_SENT.inc();
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    // Handle incoming messages (subscribe/unsubscribe)
    let subscriptions_for_recv = Arc::clone(&subscriptions);
    let stream_hub = state.stream_hub.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text) {
                        metrics::WEBSOCKET_MESSAGES_RECEIVED.inc();
                        info!("Received stream command: {:?}", cmd);

                        if let Some(symbols) = cmd.subscribe {
                            let mut newly_added: Vec<String> = Vec::new();
                            {
                                let mut subs = subscriptions_for_recv.write().await;
                                for s in symbols {
                                    if subs.insert(s.clone()) {
                                        newly_added.push(s);
                                    }
                                }
                            }

                            if !newly_added.is_empty() {
                                if let Err(e) = stream_hub.subscribe_symbols(&newly_added).await {
                                    error!("Failed to subscribe symbols: {}", e);
                                    {
                                        let mut subs = subscriptions_for_recv.write().await;
                                        for s in &newly_added {
                                            subs.remove(s);
                                        }
                                    }
                                    let _ = out_tx
                                        .send(Message::Text(
                                            serde_json::json!({"error": e.to_string()})
                                                .to_string()
                                                .into(),
                                        ))
                                        .await;
                                } else {
                                    // Update symbol count on successful subscription
                                    let count = {
                                        let subs = subscriptions_for_recv.read().await;
                                        subs.len()
                                    };
                                    metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(count as f64);
                                }
                            }
                        }

                        if let Some(symbols) = cmd.unsubscribe {
                            let mut removed: Vec<String> = Vec::new();
                            {
                                let mut subs = subscriptions_for_recv.write().await;
                                for s in symbols {
                                    if subs.remove(&s) {
                                        removed.push(s);
                                    }
                                }
                            }

                            if !removed.is_empty() {
                                stream_hub.unsubscribe_symbols(&removed).await;
                                // Update symbol count after unsubscription
                                let count = {
                                    let subs = subscriptions_for_recv.read().await;
                                    subs.len()
                                };
                                metrics::WEBSOCKET_SYMBOLS_SUBSCRIBED.set(count as f64);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket closed by client");
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete, then ensure per-client resources are torn down.
    tokio::select! {
        _ = &mut send_task => info!("Send task completed"),
        _ = &mut recv_task => info!("Receive task completed"),
    }

    // Ensure tasks stop promptly.
    send_task.abort();
    recv_task.abort();

    // Release this client's active subscriptions from the global hub.
    let symbols_to_release: Vec<String> = {
        let subs = subscriptions.read().await;
        subs.iter().cloned().collect()
    };
    state
        .stream_hub
        .unsubscribe_symbols(&symbols_to_release)
        .await;

    info!("WebSocket stream connection closed");
}

/// Wait for initial subscription message
async fn wait_for_subscription(socket: &mut WebSocket) -> Option<Vec<String>> {
    while let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg
            && let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text)
            && let Some(symbols) = cmd.subscribe
        {
            return Some(symbols);
        }
    }
    None
}

/// Stream command from client
#[derive(Debug, Deserialize)]
struct StreamCommand {
    subscribe: Option<Vec<String>>,
    unsubscribe: Option<Vec<String>>,
}

/// Graceful shutdown handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal");
        },
        _ = terminate => {
            warn!("Received terminate signal");
        },
    }

    info!("Shutting down gracefully...");
}
