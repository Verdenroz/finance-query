mod cache;

use axum::{
    Router,
    extract::{
        Extension, Path, Query, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
};
use cache::Cache;
use finance_query::{
    Frequency, Interval, Region, ScreenerQuery, ScreenerType, SectorType, StatementType, Ticker,
    Tickers, TimeRange, ValueFormat, YahooError, finance, screener_query, streaming::PriceStream,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    cache: Cache,
    stream_hub: StreamHub,
}

/// Process-wide hub that maintains a single upstream Yahoo Finance stream.
///
/// Multiple downstream WebSocket clients can subscribe/unsubscribe to symbols.
/// Symbol subscriptions are ref-counted so each symbol is only subscribed once upstream.
#[derive(Clone, Default)]
struct StreamHub {
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

    async fn resubscribe(&self) -> Option<PriceStream> {
        let inner = self.inner.lock().await;
        inner.upstream.as_ref().map(|s| s.resubscribe())
    }

    async fn subscribe_symbols(&self, symbols: &[String]) -> Result<(), YahooError> {
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

    async fn unsubscribe_symbols(&self, symbols: &[String]) {
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
    pub const SEARCH_HITS: u32 = 6;
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

/// Filter a JSON value to only include specified fields (top-level only)
fn filter_fields(
    value: serde_json::Value,
    fields: &std::collections::HashSet<String>,
) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let filtered: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(k, _)| fields.contains(k))
                .collect();
            serde_json::Value::Object(filtered)
        }
        serde_json::Value::Array(arr) => {
            // Filter each object in the array
            serde_json::Value::Array(arr.into_iter().map(|v| filter_fields(v, fields)).collect())
        }
        // Non-object/array values pass through unchanged
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
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
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

fn parse_statement_type(s: &str) -> Option<StatementType> {
    match s.to_lowercase().as_str() {
        "income" => Some(StatementType::Income),
        "balance" => Some(StatementType::Balance),
        "cashflow" | "cash-flow" => Some(StatementType::CashFlow),
        _ => None,
    }
}

/// Holder types for /holders/{symbol}/{type}
#[derive(Debug, Clone, Copy)]
enum HolderType {
    Major,
    Institutional,
    Mutualfund,
    InsiderTransactions,
    InsiderPurchases,
    InsiderRoster,
}

impl HolderType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "major" => Some(Self::Major),
            "institutional" => Some(Self::Institutional),
            "mutualfund" | "mutual-fund" => Some(Self::Mutualfund),
            "insider-transactions" => Some(Self::InsiderTransactions),
            "insider-purchases" => Some(Self::InsiderPurchases),
            "insider-roster" => Some(Self::InsiderRoster),
            _ => None,
        }
    }

    fn valid_types() -> &'static str {
        "major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster"
    }
}

/// Analysis types for /analysis/{symbol}/{type}
#[derive(Debug, Clone, Copy)]
enum AnalysisType {
    Recommendations,
    UpgradesDowngrades,
    EarningsEstimate,
    EarningsHistory,
}

impl AnalysisType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "recommendations" => Some(Self::Recommendations),
            "upgrades-downgrades" => Some(Self::UpgradesDowngrades),
            "earnings-estimate" => Some(Self::EarningsEstimate),
            "earnings-history" => Some(Self::EarningsHistory),
            _ => None,
        }
    }

    fn valid_types() -> &'static str {
        "recommendations, upgrades-downgrades, earnings-estimate, earnings-history"
    }
}

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

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    init_tracing();

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

    let state = AppState {
        cache,
        stream_hub: StreamHub::new(),
    };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    // Build router with routes
    Router::new()
        // Nest all API routes under /v2
        .nest("/v2", api_routes())
        .layer(Extension(state))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// API routes
fn api_routes() -> Router {
    Router::new()
        // Routes are sorted alphabetically by path.
        // GET /v2/analysis/{symbol}/{analysis_type}
        .route("/analysis/{symbol}/{analysis_type}", get(get_analysis))
        // GET /v2/capital-gains/{symbol}?range=<str>
        .route("/capital-gains/{symbol}", get(get_capital_gains))
        // GET /v2/chart/{symbol}?interval=<str>&range=<str>&events=<bool>
        .route("/chart/{symbol}", get(get_chart))
        // GET /v2/currencies
        .route("/currencies", get(get_currencies))
        // GET /v2/dividends/{symbol}?range=<str>
        .route("/dividends/{symbol}", get(get_dividends))
        // GET /v2/exchanges
        .route("/exchanges", get(get_exchanges))
        // GET /v2/financials/{symbol}/{statement}?frequency=<annual|quarterly>
        .route("/financials/{symbol}/{statement}", get(get_financials))
        // GET /v2/health - version-prefixed health check
        .route("/health", get(health_check))
        // GET /v2/holders/{symbol}/{holder_type}
        .route("/holders/{symbol}/{holder_type}", get(get_holders))
        // GET /v2/hours
        .route("/hours", get(get_hours))
        // GET /v2/indicators/{symbol}?interval=<str>&range=<str>
        .route("/indicators/{symbol}", get(get_indicators))
        // GET /v2/indices?format=<raw|pretty|both>
        .route("/indices", get(get_indices))
        // GET /v2/industries/{industry_key}
        .route("/industries/{industry_key}", get(get_industry))
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
        // GET /v2/screeners/{screener_type}?count=<u32>
        .route("/screeners/{screener_type}", get(get_screeners))
        // POST /v2/screeners/custom
        .route("/screeners/custom", post(post_custom_screener))
        // GET /v2/search?q=<string>&hits=<u32>
        .route("/search", get(search))
        // GET /v2/sectors/{sector_type}
        .route("/sectors/{sector_type}", get(get_sector))
        // GET /v2/spark?symbols=<csv>&interval=<str>&range=<str>
        .route("/spark", get(get_spark))
        // GET /v2/splits/{symbol}?range=<str>
        .route("/splits/{symbol}", get(get_splits))
        // GET /v2/stream - WebSocket real-time price streaming
        .route("/stream", get(ws_stream_handler))
        // GET /v2/transcripts/{symbol}?quarter=<str>&year=<i32>
        .route("/transcripts/{symbol}", get(get_transcript))
        // GET /v2/transcripts/{symbol}/all?limit=<usize>
        .route("/transcripts/{symbol}/all", get(get_transcripts))
        // GET /v2/trending?region=<str>
        .route("/trending", get(get_trending))
}

/// GET /health
///
/// Query: (none)
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
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

    // Cache key includes symbol and logo flag
    let logo_str = if params.logo { "1" } else { "0" };
    let cache_key = Cache::key("quote", &[&symbol.to_uppercase(), logo_str]);
    let logo = params.logo;
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let quote = ticker.quote(logo).await?;
                info!("Successfully fetched quote for {}", symbol_clone);
                let json = serde_json::to_value(&quote)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
fn error_response(e: YahooError) -> impl IntoResponse {
    let status = match e {
        YahooError::SymbolNotFound { .. } => StatusCode::NOT_FOUND,
        YahooError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
        YahooError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
        YahooError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
        YahooError::ServerError { status, .. } => {
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
/// Attempts to downcast to YahooError to preserve HTTP status code mapping
fn into_error_response(e: Box<dyn std::error::Error + Send + Sync>) -> axum::response::Response {
    // Try to downcast to YahooError first to get proper HTTP status codes
    if let Some(yahoo_err) = e.downcast_ref::<YahooError>() {
        // Map YahooError to appropriate HTTP status codes
        let status = match yahoo_err {
            YahooError::SymbolNotFound { .. } => StatusCode::NOT_FOUND,
            YahooError::AuthenticationFailed { .. } => StatusCode::UNAUTHORIZED,
            YahooError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            YahooError::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            YahooError::ServerError { status, .. } => {
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
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": e.to_string(),
            "status": 500
        })),
    )
        .into_response()
}

// Helper to parse interval string
fn parse_interval(s: &str) -> Interval {
    match s {
        "1m" => Interval::OneMinute,
        "5m" => Interval::FiveMinutes,
        "15m" => Interval::FifteenMinutes,
        "30m" => Interval::ThirtyMinutes,
        "1h" => Interval::OneHour,
        "1d" => Interval::OneDay,
        "1wk" => Interval::OneWeek,
        "1mo" => Interval::OneMonth,
        "3mo" => Interval::ThreeMonths,
        _ => Interval::OneDay,
    }
}

// Helper to parse range string
fn parse_range(s: &str) -> TimeRange {
    match s {
        "1d" => TimeRange::OneDay,
        "5d" => TimeRange::FiveDays,
        "1mo" => TimeRange::OneMonth,
        "3mo" => TimeRange::ThreeMonths,
        "6mo" => TimeRange::SixMonths,
        "1y" => TimeRange::OneYear,
        "2y" => TimeRange::TwoYears,
        "5y" => TimeRange::FiveYears,
        "10y" => TimeRange::TenYears,
        "ytd" => TimeRange::YearToDate,
        "max" => TimeRange::Max,
        _ => TimeRange::OneMonth,
    }
}

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
    let mut symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    symbols.sort(); // Sort for consistent cache key
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching batch quotes for {} symbols (logo={}, format={}, fields={:?})",
        symbols.len(),
        params.logo,
        format.as_str(),
        params.fields
    );

    // Cache key: sorted symbols + logo flag
    let logo_str = if params.logo { "1" } else { "0" };
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("quotes", &[&symbols_key, logo_str]);
    let logo = params.logo;

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::QUOTES,
            cache::is_market_open(),
            || async move {
                // Use Tickers for batch fetching (single API call)
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.quotes(logo).await?;
                info!(
                    "Batch fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                let json = serde_json::to_value(&batch_response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
    let region_str = region.map(|r| r.as_str()).unwrap_or("all");

    info!(
        "Fetching indices (region={}, format={}, fields={:?})",
        region_str,
        format.as_str(),
        params.fields
    );

    let cache_key = Cache::key("indices", &[region_str]);

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICES,
            cache::is_market_open(),
            || async move {
                let batch_response = finance::indices(region).await?;
                info!(
                    "Indices fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                let json = serde_json::to_value(&batch_response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key(
        "recommendations",
        &[&symbol.to_uppercase(), &limit.to_string()],
    );
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let recommendation = ticker.recommendations(limit).await?;
                let json = serde_json::to_value(&recommendation)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
    let events_str = if params.events { "1" } else { "0" };
    info!(
        "Fetching chart data for {} (events={}, fields={:?})",
        symbol, params.events, params.fields
    );

    let cache_key = Cache::key(
        "chart",
        &[
            &symbol.to_uppercase(),
            &params.interval,
            &params.range,
            events_str,
        ],
    );
    let symbol_clone = symbol.clone();
    let include_events = params.events;

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::CHART,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let chart = ticker.chart(interval, range).await?;
                let mut json = serde_json::to_value(&chart)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

                // Include events if requested
                if include_events && let serde_json::Value::Object(ref mut map) = json {
                    if let Ok(dividends) = ticker.dividends(range).await {
                        map.insert(
                            "dividends".to_string(),
                            serde_json::to_value(dividends).unwrap_or_default(),
                        );
                    }
                    if let Ok(splits) = ticker.splits(range).await {
                        map.insert(
                            "splits".to_string(),
                            serde_json::to_value(splits).unwrap_or_default(),
                        );
                    }
                    if let Ok(capital_gains) = ticker.capital_gains(range).await {
                        map.insert(
                            "capitalGains".to_string(),
                            serde_json::to_value(capital_gains).unwrap_or_default(),
                        );
                    }
                }
                Ok(json)
            },
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
    let mut symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    symbols.sort(); // Sort for consistent cache key
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching spark data for {} symbols (interval={}, range={})",
        symbols.len(),
        params.interval,
        params.range
    );

    // Cache key: sorted symbols + interval + range
    let symbols_key = symbols.join(",").to_uppercase();
    let cache_key = Cache::key("spark", &[&symbols_key, &params.interval, &params.range]);

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SPARK,
            cache::is_market_open(),
            || async move {
                let tickers = Tickers::new(symbols).await?;
                let batch_response = tickers.spark(interval, range).await?;
                info!(
                    "Spark fetch complete: {} success, {} errors",
                    batch_response.success_count(),
                    batch_response.error_count()
                );
                let json = serde_json::to_value(&batch_response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key("dividends", &[&symbol.to_uppercase(), &params.range]);
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let dividends = ticker.dividends(range).await?;
                let json = serde_json::to_value(&dividends)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key("splits", &[&symbol.to_uppercase(), &params.range]);
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let splits = ticker.splits(range).await?;
                let json = serde_json::to_value(&splits)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key("capital-gains", &[&symbol.to_uppercase(), &params.range]);
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HISTORICAL,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let gains = ticker.capital_gains(range).await?;
                let json = serde_json::to_value(&gains)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key(
        "indicators",
        &[&symbol.to_uppercase(), &params.interval, &params.range],
    );
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICATORS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let indicators = ticker.indicators(interval, range).await?;
                info!("Successfully calculated indicators for {}", symbol_clone);
                let json = serde_json::to_value(&indicators)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
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

    // Cache key includes query and key options
    let cache_key = Cache::key(
        "search",
        &[
            &params.q.to_lowercase(),
            &params.quotes.to_string(),
            &params.news.to_string(),
            if params.logo { "1" } else { "0" },
        ],
    );

    let query = params.q.clone();
    let mut options = finance::SearchOptions::new()
        .quotes_count(params.quotes)
        .news_count(params.news)
        .enable_fuzzy_query(params.fuzzy)
        .enable_logo_url(params.logo)
        .enable_research_reports(params.research)
        .enable_cultural_assets(params.cultural);

    // Apply optional region override
    if let Some(region_str) = params.region
        && let Some(region) = parse_region(&region_str)
    {
        options = options.region(region);
    }

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let result = finance::search(&query, &options).await?;
                let json = serde_json::to_value(&result)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
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

    let cache_key = Cache::key(
        "lookup",
        &[
            &params.q.to_lowercase(),
            &params.lookup_type.to_lowercase(),
            &params.count.to_string(),
            if params.logo { "1" } else { "0" },
        ],
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

    let query = params.q.clone();
    let mut options = finance::LookupOptions::new()
        .lookup_type(lookup_type)
        .count(params.count)
        .include_logo(params.logo);

    // Apply optional region override
    if let Some(region_str) = params.region
        && let Some(region) = parse_region(&region_str)
    {
        options = options.region(region);
    }

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SEARCH,
            cache::is_market_open(),
            || async move {
                let result = finance::lookup(&query, &options).await?;
                let json = serde_json::to_value(&result)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
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

    let cache_key = Cache::key("news", &["general"]);

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::GENERAL_NEWS,
            cache::is_market_open(),
            || async move {
                let news = finance::news().await?;
                let json = serde_json::to_value(&news)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key("news", &[&symbol.to_uppercase()]);
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::NEWS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let news = ticker.news().await?;
                let json = serde_json::to_value(&news)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let date_str = params.date.map(|d| d.to_string()).unwrap_or_default();
    let cache_key = Cache::key("options", &[&symbol.to_uppercase(), &date_str]);
    let symbol_clone = symbol.clone();
    let date = params.date;

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::OPTIONS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let options_response = ticker.options(date).await?;
                let json = serde_json::to_value(&options_response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key(
        "financials",
        &[&symbol.to_uppercase(), &statement, &params.frequency],
    );

    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::FINANCIALS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let result = ticker.financials(statement_type, frequency).await?;
                let json = serde_json::to_value(&result)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
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

    // Short cache (5 min) - market hours change throughout the day
    let cache_key = Cache::key("hours", &[region_display]);
    let region = params.region.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MARKET_HOURS,
            cache::is_market_open(),
            || async move {
                let response = finance::hours(region.as_deref()).await?;
                let json = serde_json::to_value(&response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    // Long cache - quote type rarely changes
    let cache_key = Cache::key("quote-type", &[&symbol.to_uppercase()]);
    let symbol_clone = symbol.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let response = ticker.quote_type().await?;
                let json = serde_json::to_value(&response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let cache_key = Cache::key("holders", &[&symbol.to_uppercase(), &holder_type]);
    let symbol_clone = symbol.clone();
    let holder_type_clone = holder_type.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::HOLDERS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let json: serde_json::Value = match ht {
                    HolderType::Major => {
                        let data = ticker.major_holders().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    HolderType::Institutional => {
                        let data = ticker.institution_ownership().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    HolderType::Mutualfund => {
                        let data = ticker.fund_ownership().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    HolderType::InsiderTransactions => {
                        let data = ticker.insider_transactions().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    HolderType::InsiderPurchases => {
                        let data = ticker.share_purchase_activity().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    HolderType::InsiderRoster => {
                        let data = ticker.insider_holders().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                };
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to fetch {} holders for {}: {}",
                holder_type_clone, symbol, e
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

    let cache_key = Cache::key("analysis", &[&symbol.to_uppercase(), &analysis_type]);

    info!(
        "Fetching {} analysis for {} (fields={:?})",
        analysis_type, symbol, params.fields
    );

    let symbol_clone = symbol.clone();
    let analysis_type_clone = analysis_type.clone();

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::ANALYSIS,
            cache::is_market_open(),
            || async move {
                let ticker = Ticker::new(&symbol_clone).await?;
                let json: serde_json::Value = match at {
                    AnalysisType::Recommendations => {
                        let data = ticker.recommendation_trend().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    AnalysisType::UpgradesDowngrades => {
                        let data = ticker.grading_history().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    AnalysisType::EarningsEstimate => {
                        let data = ticker.earnings_trend().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                    AnalysisType::EarningsHistory => {
                        let data = ticker.earnings_history().await?;
                        serde_json::to_value(data)
                            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                    }
                };
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!(
                "Failed to fetch {} analysis for {}: {}",
                analysis_type_clone, symbol, e
            );
            into_error_response(e)
        }
    }
}

/// GET /v2/screeners/{screener_type}
///
/// Path params:
/// - `screener_type`: One of 15 predefined screener types (kebab-case)
///   - Equity: aggressive-small-caps, day-gainers, day-losers, growth-technology-stocks,
///     most-actives, most-shorted-stocks, small-cap-gainers, undervalued-growth-stocks,
///     undervalued-large-caps
///   - Fund: conservative-foreign-funds, high-yield-bond, portfolio-anchors,
///     solid-large-growth-funds, solid-midcap-growth-funds, top-mutual-funds
///
/// Query: `count` (u32, default 25, max 250), `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_screeners(
    Extension(state): Extension<AppState>,
    Path(screener_type): Path<String>,
    Query(params): Query<ScreenersQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    let st = match screener_type.parse::<ScreenerType>() {
        Ok(t) => t,
        Err(_) => {
            let error = serde_json::json!({
                "error": format!("Invalid screener type: '{}'. Valid types: {}", screener_type, ScreenerType::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    let cache_key = Cache::key("screener", &[&screener_type, &params.count.to_string()]);
    let count = params.count;
    let screener_type_clone = screener_type.clone();

    info!(
        "Fetching {} screener (count={}, format={:?}, fields={:?})",
        screener_type, count, params.format, params.fields
    );

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let data = finance::screener(st, count).await?;
                let json = serde_json::to_value(data)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} screener: {}", screener_type_clone, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/sectors/{sector_type}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_sector(
    Extension(state): Extension<AppState>,
    Path(sector_type): Path<String>,
    Query(params): Query<SectorQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    let st = match sector_type.parse::<SectorType>() {
        Ok(t) => t,
        Err(_) => {
            let error = serde_json::json!({
                "error": format!("Invalid sector type: '{}'. Valid types: {}", sector_type, SectorType::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    let cache_key = Cache::key("sector", &[&sector_type]);
    let sector_type_clone = sector_type.clone();

    info!(
        "Fetching {} sector (format={:?}, fields={:?})",
        sector_type, params.format, params.fields
    );

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let data = finance::sector(st).await?;
                let json = serde_json::to_value(data)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} sector: {}", sector_type_clone, e);
            into_error_response(e)
        }
    }
}

/// GET /v2/industries/{industry_key}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_industry(
    Extension(state): Extension<AppState>,
    Path(industry_key): Path<String>,
    Query(params): Query<SectorQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());

    let cache_key = Cache::key("industry", &[&industry_key]);
    let industry_key_clone = industry_key.clone();

    info!(
        "Fetching {} industry (format={:?}, fields={:?})",
        industry_key, params.format, params.fields
    );

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::SECTORS,
            cache::is_market_open(),
            || async move {
                let data = finance::industry(&industry_key_clone).await?;
                let json = serde_json::to_value(data)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => {
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} industry: {}", industry_key, e);
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

    // Parse quote type
    let quote_type = body
        .quote_type
        .as_deref()
        .and_then(|s| s.parse::<screener_query::QuoteType>().ok())
        .unwrap_or_default();

    // Parse sort type
    let sort_ascending = body
        .sort_type
        .as_deref()
        .map(|s| s.to_lowercase() == "asc")
        .unwrap_or(false);

    // Build the query
    let mut query = ScreenerQuery::new()
        .size(body.size)
        .offset(body.offset)
        .quote_type(quote_type);

    // Set sort field if provided
    if let Some(sort_field) = body.sort_field {
        query = query.sort_by(sort_field, sort_ascending);
    }

    // Add filter conditions
    let filter_count = body.filters.len();
    for filter in body.filters {
        let op = match filter.operator.to_lowercase().as_str() {
            "eq" | "=" => screener_query::Operator::Eq,
            "gt" | ">" => screener_query::Operator::Gt,
            "gte" | ">=" => screener_query::Operator::Gte,
            "lt" | "<" => screener_query::Operator::Lt,
            "lte" | "<=" => screener_query::Operator::Lte,
            "btwn" | "between" => screener_query::Operator::Between,
            _ => {
                let error = serde_json::json!({
                    "error": format!("Invalid operator: '{}'. Valid: eq, gt, gte, lt, lte, btwn", filter.operator),
                    "status": 400
                });
                return (StatusCode::BAD_REQUEST, Json(error)).into_response();
            }
        };

        let mut condition = finance_query::QueryCondition::new(filter.field.clone(), op);

        // Add value(s) to the condition
        match &filter.value {
            serde_json::Value::String(s) => {
                condition = condition.value_str(s.clone());
            }
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    condition = condition.value(f);
                }
            }
            serde_json::Value::Array(arr) => {
                // For BETWEEN operator or multiple values
                for v in arr {
                    match v {
                        serde_json::Value::String(s) => {
                            condition = condition.value_str(s.clone());
                        }
                        serde_json::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                condition = condition.value(f);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }

        query = query.add_condition(condition);
    }

    info!(
        "Executing custom screener (size={}, quote_type={:?}, filters={})",
        body.size, quote_type, filter_count
    );

    let result = finance::custom_screener(query).await;

    match result {
        Ok(data) => {
            let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to execute custom screener: {}", e);
            error_response(e).into_response()
        }
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
    let quarter_str = params.quarter.as_deref().unwrap_or("latest");
    let year_str = params
        .year
        .map(|y| y.to_string())
        .unwrap_or_else(|| "latest".to_string());
    let cache_key = Cache::key(
        "transcript",
        &[&symbol.to_uppercase(), quarter_str, &year_str],
    );

    info!(
        "Fetching transcript for {} (quarter={:?}, year={:?})",
        symbol, params.quarter, params.year
    );

    let symbol_clone = symbol.clone();
    let quarter = params.quarter.clone();
    let year = params.year;

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::TRANSCRIPT,
            cache::is_market_open(),
            || async move {
                let response =
                    finance::earnings_transcript(&symbol_clone, quarter.as_deref(), year).await?;
                let json = serde_json::to_value(&response)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
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
    let limit_str = params
        .limit
        .map(|l| l.to_string())
        .unwrap_or_else(|| "all".to_string());
    let cache_key = Cache::key("transcripts", &[&symbol.to_uppercase(), &limit_str]);

    info!(
        "Fetching all transcripts for {} (limit={:?})",
        symbol, params.limit
    );

    let symbol_clone = symbol.clone();
    let limit = params.limit;

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::EARNINGS_LIST,
            cache::is_market_open(),
            || async move {
                let transcripts = finance::earnings_transcripts(&symbol_clone, limit).await?;
                let json = serde_json::to_value(&transcripts)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
    let cache_key = Cache::key("currencies", &[]);

    info!("Fetching currencies");

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async {
                let currencies = finance::currencies().await?;
                let json = serde_json::to_value(currencies)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
    let cache_key = Cache::key("exchanges", &[]);

    info!("Fetching exchanges");

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::METADATA,
            cache::is_market_open(),
            || async {
                let exchanges = finance::exchanges().await?;
                let json = serde_json::to_value(&exchanges)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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

    let region_str = params.region.as_deref().unwrap_or("US");
    let cache_key = Cache::key("market_summary", &[region_str]);

    info!(
        "Fetching market summary (region={:?}, format={}, fields={:?})",
        region,
        format.as_str(),
        params.fields
    );

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::INDICES,
            cache::is_market_open(),
            || async move {
                let summary = finance::market_summary(region).await?;
                let json = serde_json::to_value(summary)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
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
    let region_str = params.region.as_deref().unwrap_or("US");
    let cache_key = Cache::key("trending", &[region_str]);

    info!("Fetching trending tickers (region={:?})", region);

    match state
        .cache
        .get_or_fetch(
            &cache_key,
            cache::ttl::MOVERS,
            cache::is_market_open(),
            || async move {
                let trending = finance::trending(region).await?;
                let json = serde_json::to_value(trending)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
                Ok(json)
            },
        )
        .await
    {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Failed to fetch trending tickers: {}", e);
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
    ws.on_upgrade(move |socket| handle_stream_socket(state, socket))
}

/// Handle the WebSocket connection for streaming
async fn handle_stream_socket(state: AppState, mut socket: WebSocket) {
    info!("New streaming WebSocket connection");

    // Wait for initial subscription message
    let symbols = match wait_for_subscription(&mut socket).await {
        Some(symbols) => symbols,
        None => {
            warn!("WebSocket closed before subscription");
            return;
        }
    };

    info!("Starting stream for symbols: {:?}", symbols);

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

                            if !newly_added.is_empty()
                                && let Err(e) = stream_hub.subscribe_symbols(&newly_added).await
                            {
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
