//! REST and WebSocket inbound parameter/message types, one struct per
//! endpoint or channel.
//!
//! These live in the lib (not `handlers/`, which only the bin target
//! compiles) so `cargo soothfast spec gen` can resolve them from the lib's
//! rustdoc JSON: each `#[soothfast::route]` marker names its struct via
//! `params = "..."` (query parameters) or `request = "..."` (POST body /
//! WS publish message).

use finance_query::streaming::{AlertConditionKind, AlertRule};
use finance_query::{
    Frequency, IndicesRegion, Interval, LookupType, QuoteType, Region, SortType, StatementType,
    TimeRange, ValueFormat,
};
use serde::{Deserialize, Deserializer};

// -- REST-only routing enums -------------------------------------------------
//
// These have no equivalent in the base `finance_query` library — they exist
// purely to route a REST path segment to a GraphQL field, so they live here
// rather than as a library concept.

/// Fiscal quarter for `GET /v2/transcripts/{symbol}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Quarter {
    Q1,
    Q2,
    Q3,
    Q4,
}

impl Quarter {
    pub fn as_str(&self) -> &'static str {
        match self {
            Quarter::Q1 => "Q1",
            Quarter::Q2 => "Q2",
            Quarter::Q3 => "Q3",
            Quarter::Q4 => "Q4",
        }
    }
}

/// Analysis type for `GET /v2/analysis/{symbol}/{type}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisType {
    Recommendations,
    UpgradesDowngrades,
    EarningsEstimate,
    EarningsHistory,
}

impl AnalysisType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalysisType::Recommendations => "recommendations",
            AnalysisType::UpgradesDowngrades => "upgrades-downgrades",
            AnalysisType::EarningsEstimate => "earnings-estimate",
            AnalysisType::EarningsHistory => "earnings-history",
        }
    }
}

/// Holder type for `GET /v2/holders/{symbol}/{type}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HolderType {
    Major,
    Institutional,
    #[serde(rename = "mutualfund")]
    MutualFund,
    InsiderTransactions,
    InsiderPurchases,
    InsiderRoster,
}

impl HolderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            HolderType::Major => "major",
            HolderType::Institutional => "institutional",
            HolderType::MutualFund => "mutualfund",
            HolderType::InsiderTransactions => "insider-transactions",
            HolderType::InsiderPurchases => "insider-purchases",
            HolderType::InsiderRoster => "insider-roster",
        }
    }
}

/// Deserialize `format` through [`ValueFormat`]'s `FromStr` rather than serde.
///
/// Typing the field is what puts the enum in the generated spec, but `FromStr`
/// is what the endpoint has always accepted: case-insensitive, with the `fmt`/
/// `full` aliases, and an unrecognized value falling back to the default
/// instead of rejecting the request. Going through it keeps that contract.
fn lenient_value_format<'de, D>(de: D) -> Result<Option<ValueFormat>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(de)?
        .as_deref()
        .and_then(ValueFormat::parse))
}

fn default_recommendations_env_limit() -> u32 {
    std::env::var("RECOMMENDATIONS_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
}

fn default_recommendations_limit() -> u32 {
    10
}

fn default_calendar_range() -> TimeRange {
    TimeRange::OneMonth
}

/// Default chart interval, overridable via `DEFAULT_INTERVAL` env var.
pub fn default_interval() -> Interval {
    std::env::var("DEFAULT_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(Interval::OneDay)
}

/// Default chart range, overridable via `DEFAULT_RANGE` env var.
pub fn default_range() -> TimeRange {
    std::env::var("DEFAULT_RANGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(TimeRange::OneMonth)
}

fn default_vs_currency() -> String {
    "usd".to_string()
}

fn default_crypto_count() -> usize {
    50
}

fn default_crypto_search_limit() -> u32 {
    25
}

fn default_max_range() -> TimeRange {
    TimeRange::Max
}

fn default_frequency() -> Frequency {
    Frequency::Annual
}

fn default_news_count() -> u32 {
    10
}

fn default_screeners_count() -> u32 {
    std::env::var("SCREENERS_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(25)
}

fn default_hits() -> u32 {
    std::env::var("SEARCH_HITS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn default_logo() -> bool {
    true
}

fn default_lookup_type() -> LookupType {
    LookupType::All
}

fn default_lookup_count() -> u32 {
    25
}

// -- analysis --------------------------------------------------------------

/// Query parameters for `GET /v2/recommendations/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsQuery {
    /// Maximum number of similar symbols to return
    #[serde(default = "default_recommendations_env_limit")]
    pub limit: u32,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/recommendations`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRecommendationsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Maximum number of similar symbols per requested symbol
    #[serde(default = "default_recommendations_limit")]
    pub limit: u32,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max symbols per page; omitted (with page_cursor also omitted) = every
    /// requested symbol's recommendations as a bare array, unchanged from
    /// pre-pagination behavior
    pub page_limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub page_cursor: Option<String>,
}

/// Query parameters for `GET /v2/analysis/{symbol}/{type}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- calendar --------------------------------------------------------------

/// Query parameters for `GET /v2/calendar`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Forward window: 1wk/5d, 1mo, 3mo, etc. (default: 1mo)
    #[serde(default = "default_calendar_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- chart -----------------------------------------------------------------

/// Query parameters for `GET /v2/chart/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartQuery {
    /// Candle interval (e.g. 1m, 5m, 1h, 1d; default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (e.g. 1d, 5d, 1mo, 1y, max; default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Start date as Unix timestamp (seconds). When provided together with `end`,
    /// overrides `range` and uses absolute date boundaries.
    pub start: Option<i64>,
    /// End date as Unix timestamp (seconds). Defaults to now when `start` is set.
    pub end: Option<i64>,
    /// Include events (dividends, splits, capital gains) in response
    #[serde(default)]
    pub events: bool,
    /// Detect candlestick patterns and include per-candle signals in response.
    /// The `patterns` array aligns 1:1 with the `candles` array; `null` means
    /// no pattern was detected on that bar.
    #[serde(default)]
    pub patterns: bool,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max candles per page; omitted (with cursor also omitted) = every matching
    /// candle as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/spark`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparkQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Candle interval (default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/charts`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchChartsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Candle interval (default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Detect candlestick patterns and include per-candle signals in response.
    #[serde(default)]
    pub patterns: bool,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's chart as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- crypto ----------------------------------------------------------------

/// Query parameters for `GET /v2/crypto/coins`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoCoinsQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    pub vs_currency: String,
    /// Number of coins to return (default: 50)
    #[serde(default = "default_crypto_count")]
    pub count: usize,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max coins per page; omitted (with cursor also omitted) = every fetched
    /// coin (up to `count`) as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/crypto/coins/{id}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoCoinQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    pub vs_currency: String,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/crypto/trending`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoTrendingQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max coins per page; omitted (with cursor also omitted) = bare array
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/crypto/global`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoGlobalQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/filings/{symbol}/congressional-trades` and
/// `GET /v2/filings/{symbol}/fails-to-deliver`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilingsQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max entries per page; omitted (with cursor also omitted) = every
    /// fetched entry as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/crypto/search`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptoSearchQuery {
    /// Free-text query (coin name, symbol, or CoinGecko id)
    pub query: String,
    /// Max matches to fetch (default: 25)
    #[serde(default = "default_crypto_search_limit")]
    pub limit_results: u32,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max matches per page; omitted (with cursor also omitted) = bare array
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- edgar -----------------------------------------------------------------

/// Query parameters for `GET /v2/edgar/search`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgarSearchQuery {
    /// Search query string (required)
    pub q: String,
    /// Comma-separated form types (e.g., "10-K,10-Q")
    pub forms: Option<String>,
    /// Start date in YYYY-MM-DD format
    pub start_date: Option<String>,
    /// End date in YYYY-MM-DD format
    pub end_date: Option<String>,
    /// Pagination offset (default: 0)
    pub from: Option<usize>,
    /// Page size (default: 100, max: 100)
    pub size: Option<usize>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/edgar/cik/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgarFieldsQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/edgar/submissions/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgarSubmissionsQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max filings per page; omitted (with cursor also omitted) = every
    /// matching filing as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/edgar/facts/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgarFactsQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// XBRL taxonomy (default: "us-gaap"); also try "ifrs-full" or "dei"
    pub taxonomy: Option<String>,
    /// Comma-separated XBRL concept names to filter to; omitted = curated defaults
    pub concepts: Option<String>,
    /// Max data points per concept per page; omitted (with cursor also omitted) =
    /// every matching data point as a bare array, unchanged from pre-pagination
    /// behavior. Applied uniformly across every returned concept.
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- events ----------------------------------------------------------------

/// Query parameters for range-scoped event endpoints
/// (`GET /v2/splits/{symbol}`, `GET /v2/capital-gains/{symbol}`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeQuery {
    /// Time range (default: max)
    #[serde(default = "default_max_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/dividends/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DividendsQuery {
    /// Time range (default: max)
    #[serde(default = "default_max_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max dividend payments per page; omitted (with cursor also omitted) = every
    /// matching payment as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/dividends`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDividendsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Time range (default: max)
    #[serde(default = "default_max_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's dividend history as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/splits`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchSplitsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Time range (default: max)
    #[serde(default = "default_max_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/capital-gains`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCapitalGainsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Time range (default: max)
    #[serde(default = "default_max_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- feeds -----------------------------------------------------------------

/// Query parameters for `GET /v2/feeds`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedsQuery {
    /// Comma-separated source slugs (default: federal-reserve, sec, marketwatch, bloomberg)
    pub sources: Option<String>,
    /// SEC form type for sec-filings source (e.g., "10-K", "8-K", default: "10-K")
    pub form_type: Option<String>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max entries to return; omitted (with cursor also omitted) = every matching
    /// entry as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- financials ------------------------------------------------------------

/// Query parameters for `GET /v2/financials/{symbol}/{statement}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialsQuery {
    /// Frequency: annual or quarterly (default: annual)
    #[serde(default = "default_frequency")]
    pub frequency: Frequency,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Comma-separated list of metric names to include in the statement (e.g. "TotalRevenue,NetIncome")
    pub metrics: Option<String>,
}

/// Query parameters for `GET /v2/financials`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFinancialsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Statement type (required): income, balance, cashflow
    pub statement: StatementType,
    /// Frequency: annual or quarterly (default: annual)
    #[serde(default = "default_frequency")]
    pub frequency: Frequency,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Comma-separated list of metric names to include in the statement (e.g. "TotalRevenue,NetIncome")
    pub metrics: Option<String>,
}

// -- fred ------------------------------------------------------------------

/// Query parameters for `GET /v2/fred/series/{id}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FredSeriesQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max observations per page; omitted (with cursor also omitted) = every
    /// matching observation as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/fred/treasury-yields`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TreasuryYieldsQuery {
    /// Calendar year (default: current year)
    pub year: Option<u32>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max rows per page; omitted (with cursor also omitted) = every matching
    /// row as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- holders ---------------------------------------------------------------

/// Query parameters for `GET /v2/holders/{symbol}/{type}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldersQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max entries per page for holder types with a list (institutional,
    /// mutualfund, insider-transactions, insider-roster); omitted (with cursor
    /// also omitted) = every matching entry as a bare array, unchanged from
    /// pre-pagination behavior. No-op for major/insider-purchases (no list).
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- indicators ------------------------------------------------------------

/// Query parameters for `GET /v2/indicators`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchIndicatorsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Candle interval (default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's indicators as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/indicators/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorsQuery {
    /// Candle interval (default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- keyless (gdelt / cftc) --------------------------------------------------

/// Query parameters for `GET /v2/gdelt/news/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GdeltNewsQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max articles per page; omitted (with cursor also omitted) = bare array
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/cftc/cot/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CotQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max weekly observations per page; omitted (with cursor also omitted) =
    /// every observation as a bare array
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- market ----------------------------------------------------------------

/// Query parameters for `GET /v2/indices`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicesQuery {
    /// Region filter: americas, europe, asia-pacific, middle-east-africa, currencies
    pub region: Option<IndicesRegion>,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/market-summary`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSummaryQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    pub region: Option<Region>,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}

/// Query parameters for `GET /v2/trending`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingQuery {
    /// Region code for localization (e.g., "US", "JP", "GB")
    pub region: Option<Region>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/fear-and-greed`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FearAndGreedQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- metadata --------------------------------------------------------------

/// Query parameters for `GET /v2/hours`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoursQuery {
    /// Region code (e.g., "US", "JP", "GB"). Defaults to US if not specified.
    pub region: Option<Region>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/quote-type/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTypeQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/currencies`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrenciesQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/exchanges`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangesQuery {
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- news ------------------------------------------------------------------

/// Query parameters for `GET /v2/news` and `GET /v2/news/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsQuery {
    /// Maximum number of articles to return (default: 10)
    #[serde(default = "default_news_count")]
    pub count: u32,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
    /// Max articles per page; omitted (with cursor also omitted) = every fetched
    /// article (up to `count`) as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/crypto/news` and `GET /v2/forex/news`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketNewsQuery {
    /// Maximum number of articles to fetch (default: 20)
    #[serde(default = "default_market_news_limit")]
    pub limit: u32,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max articles per page; omitted (with pageCursor also omitted) = every
    /// fetched article (up to `limit`) as a bare array, unchanged from
    /// pre-pagination behavior
    pub page_limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub page_cursor: Option<String>,
}

fn default_market_news_limit() -> u32 {
    20
}

// -- options ---------------------------------------------------------------

/// Query parameters for `GET /v2/options/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsQuery {
    /// Optional expiration date (Unix timestamp)
    pub date: Option<i64>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max contracts per side per page; omitted (with cursor also omitted) =
    /// every matching contract as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    /// (applied to both `calls` and `puts`)
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/options`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchOptionsQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Expiration date (Unix timestamp, optional)
    pub date: Option<i64>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's options chain as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- quote -----------------------------------------------------------------

/// Query parameters for `GET /v2/quote/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteQuery {
    /// Whether to include company logo URL (default: false)
    #[serde(default)]
    pub logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}

/// Query parameters for `GET /v2/quotes`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotesQuery {
    /// Comma-separated symbols (required)
    pub symbols: String,
    /// Whether to include company logo URLs (default: false)
    #[serde(default)]
    pub logo: bool,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's quote as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

// -- risk ------------------------------------------------------------------

/// Query parameters for `GET /v2/risk/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskQuery {
    /// Candle interval (default: 1d)
    #[serde(default = "default_interval")]
    pub interval: Interval,
    /// Time range (default: 1mo)
    #[serde(default = "default_range")]
    pub range: TimeRange,
    /// Optional benchmark symbol for beta calculation (e.g., "SPY")
    pub benchmark: Option<String>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

// -- screener --------------------------------------------------------------

/// Query parameters for `GET /v2/screeners/{screener}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenersQuery {
    /// Number of results (default: 25, max: 250)
    #[serde(default = "default_screeners_count")]
    pub count: u32,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Request body for `POST /v2/screeners/custom`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomScreenerRequest {
    /// Number of results (default: 25, max: 250)
    #[serde(default = "default_screeners_count")]
    pub size: u32,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: u32,
    /// Sort direction: "ASC" or "DESC" (default: DESC)
    #[serde(default)]
    pub sort_type: Option<SortType>,
    /// Field to sort by (default: intradaymarketcap)
    pub sort_field: Option<String>,
    /// Quote type: EQUITY, ETF, MUTUALFUND, etc. (default: EQUITY)
    pub quote_type: Option<QuoteType>,
    /// Filter conditions
    #[serde(default)]
    pub filters: Vec<FilterCondition>,
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// A single custom-screener filter condition.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterCondition {
    /// Field name (e.g., "region", "avgdailyvol3m", "intradaymarketcap")
    pub field: String,
    /// Operator: eq, gt, gte, lt, lte, btwn
    pub operator: String,
    /// Value(s) for the condition
    pub value: serde_json::Value,
}

// -- search ----------------------------------------------------------------

/// Query parameters for `GET /v2/search`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    /// Search query string (required)
    pub q: String,
    /// Maximum number of quote results (default: 6)
    #[serde(default = "default_hits")]
    pub quotes: u32,
    /// Maximum number of news results (default: 0 = disabled)
    #[serde(default)]
    pub news: u32,
    /// Enable fuzzy matching for typos (default: false)
    #[serde(default)]
    pub fuzzy: bool,
    /// Enable logo URLs in results (default: true)
    #[serde(default = "default_logo")]
    pub logo: bool,
    /// Enable research reports (default: false)
    #[serde(default)]
    pub research: bool,
    /// Enable cultural assets/NFT indices (default: false)
    #[serde(default)]
    pub cultural: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    pub region: Option<Region>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
    /// Max quotes per page; omitted (with cursor also omitted) = every fetched
    /// quote (up to `quotes`) as a bare array, unchanged from pre-pagination behavior
    pub limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    pub cursor: Option<String>,
}

/// Query parameters for `GET /v2/lookup`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupQuery {
    /// Lookup query string (required)
    pub q: String,
    /// Asset type filter: all, equity, mutualfund, etf, index, future, currency, cryptocurrency
    #[serde(default = "default_lookup_type")]
    #[serde(rename = "type")]
    pub lookup_type: LookupType,
    /// Maximum number of results (default: 25)
    #[serde(default = "default_lookup_count")]
    pub count: u32,
    /// Include logo URLs (requires additional API call, default: false)
    #[serde(default)]
    pub logo: bool,
    /// Region code for lang/region settings (e.g., "US", "JP", "GB")
    pub region: Option<Region>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}

// -- sector / industry -----------------------------------------------------

/// Query parameters for `GET /v2/sectors/{sector}` and `GET /v2/industries/{industry}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorQuery {
    /// Value format: raw, pretty, or both (default: raw)
    #[serde(default, deserialize_with = "lenient_value_format")]
    pub format: Option<ValueFormat>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}

// -- stream ----------------------------------------------------------------

/// Inbound control message for `/v2/stream`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamCommand {
    /// Ticker symbols to subscribe to.
    pub subscribe: Option<Vec<String>>,
    /// Ticker symbols to unsubscribe from.
    pub unsubscribe: Option<Vec<String>>,
    /// Replace the connection's alert rule set. While a non-empty rule set is
    /// active the server sends alert events instead of raw price updates;
    /// send an empty array to revert to raw ticks. Symbols named by rules
    /// are subscribed automatically.
    pub alerts: Option<Vec<WsAlertRule>>,
}

/// One alert rule as sent over the wire.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsAlertRule {
    /// Ticker symbol to watch.
    pub symbol: String,
    /// Predicate to evaluate. Crossing conditions compare against the
    /// previous tick and never fire on a symbol's first tick.
    pub condition: AlertConditionKind,
    /// Threshold the predicate compares against.
    pub value: f64,
    /// Fire every time the condition becomes true again, instead of once.
    #[serde(default)]
    pub repeat: bool,
}

impl WsAlertRule {
    pub fn parse(&self) -> Result<AlertRule, String> {
        let rule = AlertRule::new(self.symbol.clone(), self.condition.with_value(self.value));
        Ok(if self.repeat { rule.repeating() } else { rule })
    }
}

/// Inbound control message for `/v2/feeds/stream`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedStreamCommand {
    /// Feed source slugs to subscribe to (same slugs as `GET /v2/feeds`'s
    /// `sources` param).
    pub subscribe: Option<Vec<String>>,
    /// Feed source slugs to unsubscribe from.
    pub unsubscribe: Option<Vec<String>>,
    /// SEC filing form type, used when `sec-filings` is in `subscribe`.
    pub form_type: Option<String>,
}

// -- transcripts -----------------------------------------------------------

/// Query parameters for `GET /v2/transcripts/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptQuery {
    /// Fiscal quarter (Q1, Q2, Q3, Q4). If not provided, returns latest.
    pub quarter: Option<Quarter>,
    /// Fiscal year. If not provided with quarter, returns latest.
    pub year: Option<i32>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}

/// Query parameters for `GET /v2/earnings-transcript/{symbol}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptV2Query {
    /// Fiscal quarter (Q1, Q2, Q3, Q4). Required alongside `year` for
    /// providers with no "latest" shortcut (Alpha Vantage); Yahoo defaults
    /// to latest when omitted.
    pub quarter: Option<Quarter>,
    /// Fiscal year. See `quarter`.
    pub year: Option<i32>,
    /// Comma-separated list of fields to include in response
    pub fields: Option<String>,
}

/// Query parameters for `GET /v2/transcripts/{symbol}/all`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptsQuery {
    /// Maximum number of transcripts to return. If not provided, returns all.
    pub limit: Option<usize>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    pub lang: Option<String>,
}
