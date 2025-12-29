use axum::{
    Router,
    extract::{
        Path, Query, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
};
use finance_query::{
    Country, Frequency, Interval, ScreenerQuery, ScreenerType, SectorType, StatementType, Ticker,
    Tickers, TimeRange, ValueFormat, YahooError, finance, screener_query, streaming::PriceStream,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

/// Parse country code string into Country enum
fn parse_country(s: &str) -> Option<Country> {
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
    /// Country code for lang/region settings (e.g., "US", "JP", "GB")
    country: Option<String>,
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
    /// Country code for lang/region settings (e.g., "US", "JP", "GB")
    country: Option<String>,
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
    let app = create_app();

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

fn create_app() -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    // Build router with routes
    Router::new()
        // Health & utility (not versioned)
        // GET /health
        .route("/health", get(health_check))
        // GET /ping
        .route("/ping", get(ping))
        // Nest all API routes under /v2
        .nest("/v2", api_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// API routes
fn api_routes() -> Router {
    Router::new()
        // Routes are sorted alphabetically by path for consistency.
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
        // GET /v2/transcripts/{symbol}?quarter=<str>&year=<i32>
        .route("/transcripts/{symbol}", get(get_transcript))
        // GET /v2/transcripts/{symbol}/all?limit=<usize>
        .route("/transcripts/{symbol}/all", get(get_transcripts))
        // GET /v2/financials/{symbol}/{statement}?frequency=<annual|quarterly>
        .route("/financials/{symbol}/{statement}", get(get_financials))
        // GET /v2/hours
        .route("/hours", get(get_hours))
        // GET /v2/holders/{symbol}/{holder_type}
        .route("/holders/{symbol}/{holder_type}", get(get_holders))
        // GET /v2/indicators/{symbol}?interval=<str>&range=<str>
        .route("/indicators/{symbol}", get(get_indicators))
        // GET /v2/indices?format=<raw|pretty|both>
        .route("/indices", get(get_indices))
        // GET /v2/industries/{industry_key}
        .route("/industries/{industry_key}", get(get_industry))
        // GET /v2/market-summary
        .route("/market-summary", get(get_market_summary))
        // GET /v2/news?count=<u32>
        .route("/news", get(get_general_news))
        // GET /v2/news/{symbol}?count=<u32>
        .route("/news/{symbol}", get(get_news))
        // GET /v2/options/{symbol}?date=<i64>
        .route("/options/{symbol}", get(get_options))
        // GET /v2/quote-type/{symbol}
        .route("/quote-type/{symbol}", get(get_quote_type))
        // GET /v2/quote/{symbol}?logo=<bool>
        .route("/quote/{symbol}", get(get_quote))
        // GET /v2/quotes?symbols=<csv>&logo=<bool>
        .route("/quotes", get(get_quotes))
        // GET /v2/recommendations/{symbol}?limit=<u32>
        .route("/recommendations/{symbol}", get(get_recommendations))
        // POST /v2/screeners/custom
        .route("/screeners/custom", post(post_custom_screener))
        // GET /v2/screeners/{screener_type}?count=<u32>
        .route("/screeners/{screener_type}", get(get_screeners))
        // GET /v2/search?q=<string>&hits=<u32>
        .route("/search", get(search))
        // GET /v2/lookup?q=<string>&type=<string>&count=<u32>&logo=<bool>
        .route("/lookup", get(lookup))
        // GET /v2/sectors/{sector_type}
        .route("/sectors/{sector_type}", get(get_sector))
        // GET /v2/splits/{symbol}?range=<str>
        .route("/splits/{symbol}", get(get_splits))
        // GET /v2/trending?region=<str>
        .route("/trending", get(get_trending))
        // WebSocket /v2/stream - Real-time price streaming
        .route("/stream", get(ws_stream_handler))
}

/// GET /health
///
/// Query: (none)
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
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

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.quote(params.logo).await {
            Ok(quote) => {
                info!("Successfully fetched quote for {}", symbol);
                let json = serde_json::to_value(quote).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, format, fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch quote for {}: {}", symbol, e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch quote for {}: {}", symbol, e);
            error_response(e).into_response()
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
async fn get_quotes(Query(params): Query<QuotesQuery>) -> impl IntoResponse {
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

    // Use Tickers for batch fetching (single API call)
    let tickers = match Tickers::new(symbols).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create Tickers: {}", e);
            return error_response(e).into_response();
        }
    };

    match tickers.quotes(params.logo).await {
        Ok(batch_response) => {
            info!(
                "Batch fetch complete: {} success, {} errors",
                batch_response.success_count(),
                batch_response.error_count()
            );
            let json = serde_json::to_value(batch_response).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch batch quotes: {}", e);
            error_response(e).into_response()
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
async fn get_indices(Query(params): Query<IndicesQuery>) -> impl IntoResponse {
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

    match finance::indices(region).await {
        Ok(batch_response) => {
            info!(
                "Indices fetch complete: {} success, {} errors",
                batch_response.success_count(),
                batch_response.error_count()
            );
            let json = serde_json::to_value(batch_response).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch indices: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/recommendations/{symbol}
///
/// Query: `limit` (u32, default via `RECOMMENDATIONS_LIMIT` or server default)
async fn get_recommendations(
    Path(symbol): Path<String>,
    Query(params): Query<RecommendationsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching recommendations for {} (fields={:?})",
        symbol, params.fields
    );

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.recommendations(params.limit).await {
            Ok(recommendation) => {
                let json = serde_json::to_value(recommendation).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch recommendations: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/chart/{symbol}
///
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`), `events` (bool, default false)
async fn get_chart(
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching chart data for {} (events={}, fields={:?})",
        symbol, params.events, params.fields
    );

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.chart(interval, range).await {
            Ok(chart) => {
                let mut json = serde_json::to_value(chart).unwrap_or(serde_json::Value::Null);

                // Include events if requested
                if params.events
                    && let serde_json::Value::Object(ref mut map) = json
                {
                    // Fetch events using the same range (events are already cached from chart fetch)
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

                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch chart data: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/dividends/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_dividends(
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching dividends for {} (range={:?})", symbol, range);

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.dividends(range).await {
            Ok(dividends) => {
                let json = serde_json::to_value(dividends).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch dividends: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/splits/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_splits(
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching splits for {} (range={:?})", symbol, range);

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.splits(range).await {
            Ok(splits) => {
                let json = serde_json::to_value(splits).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch splits: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/capital-gains/{symbol}
///
/// Query: `range` (str, default "max")
async fn get_capital_gains(
    Path(symbol): Path<String>,
    Query(params): Query<RangeQuery>,
) -> impl IntoResponse {
    let range = parse_range(&params.range);
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching capital gains for {} (range={:?})", symbol, range);

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.capital_gains(range).await {
            Ok(gains) => {
                let json = serde_json::to_value(gains).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch capital gains: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/indicators/{symbol}
///
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`)
async fn get_indicators(
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

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.indicators(interval, range).await {
            Ok(indicators) => {
                let json = serde_json::to_value(indicators).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to calculate indicators: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
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
/// - `country` (string, optional): Country code for lang/region (e.g., "US", "JP")
async fn search(Query(params): Query<SearchQuery>) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Searching for: {} (quotes={}, news={}, logo={}, research={}, cultural={}, country={:?})",
        params.q,
        params.quotes,
        params.news,
        params.logo,
        params.research,
        params.cultural,
        params.country
    );

    let mut options = finance::SearchOptions::new()
        .quotes_count(params.quotes)
        .news_count(params.news)
        .enable_fuzzy_query(params.fuzzy)
        .enable_logo_url(params.logo)
        .enable_research_reports(params.research)
        .enable_cultural_assets(params.cultural);

    // Apply optional country override
    if let Some(country_str) = params.country
        && let Some(country) = parse_country(&country_str)
    {
        options = options.country(country);
    }

    match finance::search(&params.q, &options).await {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Search failed: {}", e);
            error_response(e).into_response()
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
/// - `country` (string, optional): Country code for lang/region
async fn lookup(Query(params): Query<LookupQuery>) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Looking up: {} (type={}, count={}, logo={}, country={:?})",
        params.q, params.lookup_type, params.count, params.logo, params.country
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

    let mut options = finance::LookupOptions::new()
        .lookup_type(lookup_type)
        .count(params.count)
        .include_logo(params.logo);

    // Apply optional country override
    if let Some(country_str) = params.country
        && let Some(country) = parse_country(&country_str)
    {
        options = options.country(country);
    }

    match finance::lookup(&params.q, &options).await {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Lookup failed: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/news
///
/// Returns general market news
async fn get_general_news(Query(params): Query<NewsQuery>) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching general market news (fields={:?})", params.fields);

    match finance::news().await {
        Ok(news) => {
            let json = serde_json::to_value(news).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch general news: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/news/{symbol}
///
/// Returns news for a specific symbol
async fn get_news(
    Path(symbol): Path<String>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!("Fetching news for {} (fields={:?})", symbol, params.fields);

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.news().await {
            Ok(news) => {
                let json = serde_json::to_value(news).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch news: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/options/{symbol}
///
/// Query: `date` (i64, optional expiration timestamp)
async fn get_options(
    Path(symbol): Path<String>,
    Query(params): Query<OptionsQuery>,
) -> impl IntoResponse {
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching options for {} (fields={:?})",
        symbol, params.fields
    );

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.options(params.date).await {
            Ok(options_response) => {
                let json =
                    serde_json::to_value(options_response).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch options: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
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

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.financials(statement_type, frequency).await {
            Ok(result) => {
                let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
                let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
                (StatusCode::OK, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to fetch financials: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch financials: {}", e);
            error_response(e).into_response()
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
async fn get_hours(Query(params): Query<HoursQuery>) -> impl IntoResponse {
    let region_display = params.region.as_deref().unwrap_or("US");
    info!("Fetching market hours for region: {}", region_display);

    match finance::hours(params.region.as_deref()).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch market hours: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/quote-type/{symbol}
///
/// Query: (none)
async fn get_quote_type(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching quote type for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => match ticker.quote_type().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch quote type: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
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

    let ticker = match Ticker::new(&symbol).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            return error_response(e).into_response();
        }
    };

    let result = match ht {
        HolderType::Major => ticker
            .major_holders()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        HolderType::Institutional => ticker
            .institution_ownership()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        HolderType::Mutualfund => ticker
            .fund_ownership()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        HolderType::InsiderTransactions => ticker
            .insider_transactions()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        HolderType::InsiderPurchases => ticker
            .share_purchase_activity()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        HolderType::InsiderRoster => ticker
            .insider_holders()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
    };

    match result {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} holders: {}", holder_type, e);
            error_response(e).into_response()
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

    let ticker = match Ticker::new(&symbol).await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            return error_response(e).into_response();
        }
    };

    let result = match at {
        AnalysisType::Recommendations => ticker
            .recommendation_trend()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        AnalysisType::UpgradesDowngrades => ticker
            .grading_history()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        AnalysisType::EarningsEstimate => ticker
            .earnings_trend()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
        AnalysisType::EarningsHistory => ticker
            .earnings_history()
            .await
            .map(|r| serde_json::to_value(r).unwrap()),
    };

    match result {
        Ok(json) => {
            let response = apply_transforms(json, ValueFormat::default(), fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} analysis: {}", analysis_type, e);
            error_response(e).into_response()
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

    info!(
        "Fetching {} screener (count={}, format={:?}, fields={:?})",
        screener_type, params.count, params.format, params.fields
    );

    let result = finance::screener(st, params.count).await;

    match result {
        Ok(data) => {
            let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} screener: {}", screener_type, e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/sectors/{sector_type}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_sector(
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

    info!(
        "Fetching {} sector (format={:?}, fields={:?})",
        sector_type, params.format, params.fields
    );

    let result = finance::sector(st).await;

    match result {
        Ok(data) => {
            let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} sector: {}", sector_type, e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/industries/{industry_key}
///
/// Query: `format` (raw|pretty|both), `fields` (comma-separated)
async fn get_industry(
    Path(industry_key): Path<String>,
    Query(params): Query<SectorQuery>,
) -> impl IntoResponse {
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());

    info!(
        "Fetching {} industry (format={:?}, fields={:?})",
        industry_key, params.format, params.fields
    );

    let result = finance::industry(&industry_key).await;

    match result {
        Ok(data) => {
            let json = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch {} industry: {}", industry_key, e);
            error_response(e).into_response()
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
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptQuery>,
) -> impl IntoResponse {
    info!(
        "Fetching transcript for {} (quarter={:?}, year={:?})",
        symbol, params.quarter, params.year
    );

    match finance::earnings_transcript(&symbol, params.quarter.as_deref(), params.year).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch transcript for {}: {}", symbol, e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/transcripts/{symbol}/all
///
/// Returns all earnings transcripts for a symbol.
/// Query params:
/// - `limit` (optional): Maximum number of transcripts to return.
async fn get_transcripts(
    Path(symbol): Path<String>,
    Query(params): Query<EarningsTranscriptsQuery>,
) -> impl IntoResponse {
    info!(
        "Fetching all transcripts for {} (limit={:?})",
        symbol, params.limit
    );

    match finance::earnings_transcripts(&symbol, params.limit).await {
        Ok(transcripts) => (StatusCode::OK, Json(transcripts)).into_response(),
        Err(e) => {
            error!("Failed to fetch transcripts for {}: {}", symbol, e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/currencies
///
/// Returns available currencies from Yahoo Finance.
async fn get_currencies() -> impl IntoResponse {
    info!("Fetching currencies");

    match finance::currencies().await {
        Ok(currencies) => {
            let json = serde_json::to_value(currencies).unwrap_or(serde_json::Value::Null);
            (StatusCode::OK, Json(json)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch currencies: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Query parameters for /v2/market-summary
#[derive(Deserialize)]
struct MarketSummaryQuery {
    /// Country code for localization (e.g., "US", "JP", "GB")
    country: Option<String>,
    /// Value format: raw, pretty, or both (default: raw)
    format: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/market-summary
///
/// Returns market summary with major indices, currencies, and commodities.
async fn get_market_summary(Query(params): Query<MarketSummaryQuery>) -> impl IntoResponse {
    let country = params.country.as_deref().and_then(parse_country);
    let format = parse_format(params.format.as_deref());
    let fields = parse_fields(params.fields.as_deref());
    info!(
        "Fetching market summary (country={:?}, format={}, fields={:?})",
        country,
        format.as_str(),
        params.fields
    );

    match finance::market_summary(country).await {
        Ok(summary) => {
            let json = serde_json::to_value(summary).unwrap_or(serde_json::Value::Null);
            let response = apply_transforms(json, format, fields.as_ref());
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch market summary: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Query parameters for /v2/trending
#[derive(Deserialize)]
struct TrendingQuery {
    /// Country code for localization (e.g., "US", "JP", "GB")
    country: Option<String>,
}

/// GET /v2/trending
///
/// Returns trending tickers for a region.
async fn get_trending(Query(params): Query<TrendingQuery>) -> impl IntoResponse {
    let country = params.country.as_deref().and_then(parse_country);
    info!("Fetching trending tickers (country={:?})", country);

    match finance::trending(country).await {
        Ok(trending) => {
            let json = serde_json::to_value(trending).unwrap_or(serde_json::Value::Null);
            (StatusCode::OK, Json(json)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch trending tickers: {}", e);
            error_response(e).into_response()
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
async fn ws_stream_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_stream_socket)
}

/// Handle the WebSocket connection for streaming
async fn handle_stream_socket(mut socket: WebSocket) {
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

    // Create price stream
    let symbol_refs: Vec<&str> = symbols.iter().map(|s| s.as_str()).collect();
    let stream = match PriceStream::subscribe(&symbol_refs).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create price stream: {}", e);
            let _ = socket
                .send(Message::Text(
                    serde_json::json!({"error": e.to_string()})
                        .to_string()
                        .into(),
                ))
                .await;
            return;
        }
    };

    // Split socket for concurrent read/write
    let (mut sender, mut receiver) = socket.split();

    // Spawn task to forward price updates to client
    let mut price_stream = stream;
    let send_task = tokio::spawn(async move {
        use futures_util::stream::StreamExt;
        while let Some(price) = price_stream.next().await {
            let json = serde_json::to_string(&price).unwrap_or_default();
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages (subscribe/unsubscribe)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(cmd) = serde_json::from_str::<StreamCommand>(&text) {
                        info!("Received stream command: {:?}", cmd);
                        // Commands are handled by the price stream internally
                        // For now, we just log them
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

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => info!("Send task completed"),
        _ = recv_task => info!("Receive task completed"),
    }

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
    #[allow(dead_code)]
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
