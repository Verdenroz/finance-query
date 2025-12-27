use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
};
use finance_query::{
    AsyncTicker, Frequency, Interval, StatementType, Tickers, TimeRange, YahooError, finance,
};
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
    /// Default movers count
    pub const MOVERS_COUNT: u32 = 10;
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
#[derive(Deserialize)]
struct QuoteQuery {
    /// Whether to include company logo URL (default: false)
    #[serde(default)]
    logo: bool,
}

#[derive(Deserialize)]
struct QuotesQuery {
    symbols: String, // Comma-separated symbols
    /// Whether to include company logo URLs (default: false)
    #[serde(default)]
    logo: bool,
}

#[derive(Deserialize)]
struct RecommendationsQuery {
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Deserialize)]
struct ChartQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_hits")]
    hits: u32,
}

#[derive(Deserialize)]
struct OptionsQuery {
    date: Option<i64>, // Optional expiration timestamp
}

#[derive(Deserialize)]
struct FinancialsQuery {
    /// Frequency: annual or quarterly (default: annual)
    #[serde(default = "default_frequency")]
    frequency: String,
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

/// Mover types for /movers/{type}
#[derive(Debug, Clone, Copy)]
enum MoverType {
    Gainers,
    Losers,
    Actives,
}

impl MoverType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gainers" => Some(Self::Gainers),
            "losers" => Some(Self::Losers),
            "actives" => Some(Self::Actives),
            _ => None,
        }
    }

    fn valid_types() -> &'static str {
        "gainers, losers, actives"
    }
}

#[derive(Deserialize)]
struct MoversQuery {
    #[serde(default = "default_movers_count")]
    count: u32,
}

#[derive(Deserialize)]
struct EarningsTranscriptQuery {
    event_id: String,
    company_id: String,
}

fn default_movers_count() -> u32 {
    std::env::var("MOVERS_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::MOVERS_COUNT)
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
        // GET /v2/chart/{symbol}?interval=<str>&range=<str>
        .route("/chart/{symbol}", get(get_chart))
        // GET /v2/earnings-transcript?event_id=<str>&company_id=<str>
        .route("/earnings-transcript", get(get_earnings_transcript))
        // GET /v2/financials/{symbol}/{statement}?frequency=<annual|quarterly>
        .route("/financials/{symbol}/{statement}", get(get_financials))
        // GET /v2/hours
        .route("/hours", get(get_hours))
        // GET /v2/holders/{symbol}/{holder_type}
        .route("/holders/{symbol}/{holder_type}", get(get_holders))
        // GET /v2/indicators/{symbol}?interval=<str>&range=<str>
        .route("/indicators/{symbol}", get(get_indicators))
        // GET /v2/movers/{mover_type}?count=<u32>
        .route("/movers/{mover_type}", get(get_movers))
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
        // GET /v2/search?q=<string>&hits=<u32>
        .route("/search", get(search))
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
/// Query: `logo` (bool, default: false)
async fn get_quote(
    Path(symbol): Path<String>,
    Query(params): Query<QuoteQuery>,
) -> impl IntoResponse {
    info!(
        "Received quote request for symbol: {} (logo={})",
        symbol, params.logo
    );

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.quote(params.logo).await {
            Ok(quote) => {
                info!("Successfully fetched quote for {}", symbol);
                (StatusCode::OK, Json(quote)).into_response()
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
/// Query: `symbols` (comma-separated, required), `logo` (bool, default: false)
///
/// Uses batch fetching via Tickers for optimal performance (single API call).
async fn get_quotes(Query(params): Query<QuotesQuery>) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    info!(
        "Fetching batch quotes for {} symbols (logo={})",
        symbols.len(),
        params.logo
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
            (StatusCode::OK, Json(batch_response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch batch quotes: {}", e);
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
    info!("Fetching recommendations for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.recommendations(params.limit).await {
            Ok(recommendation) => (StatusCode::OK, Json(recommendation)).into_response(),
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
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`)
async fn get_chart(
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    info!("Fetching chart data for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.chart(interval, range).await {
            Ok(chart) => (StatusCode::OK, Json(chart)).into_response(),
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

/// GET /v2/indicators/{symbol}
///
/// Query: `interval` (str, default via `DEFAULT_INTERVAL`), `range` (str, default via `DEFAULT_RANGE`)
async fn get_indicators(
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    info!(
        "Calculating indicators for {} with interval={:?}, range={:?}",
        symbol, interval, range
    );

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.indicators(interval, range).await {
            Ok(indicators) => (StatusCode::OK, Json(indicators)).into_response(),
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
/// Query: `q` (string, required), `hits` (u32, default via `SEARCH_HITS` or server default)
async fn search(Query(params): Query<SearchQuery>) -> impl IntoResponse {
    info!("Searching for: {}", params.q);

    match finance::search(&params.q, params.hits).await {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            error!("Search failed: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/news
///
/// Returns general market news
async fn get_general_news() -> impl IntoResponse {
    info!("Fetching general market news");

    match finance::news().await {
        Ok(news) => (StatusCode::OK, Json(news)).into_response(),
        Err(e) => {
            error!("Failed to fetch general news: {}", e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/news/{symbol}
///
/// Returns news for a specific symbol
async fn get_news(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching news for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.news().await {
            Ok(news) => (StatusCode::OK, Json(news)).into_response(),
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
    info!("Fetching options for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.options(params.date).await {
            Ok(options_response) => (StatusCode::OK, Json(options_response)).into_response(),
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
        "Fetching {} {} financials for {}",
        params.frequency, statement, symbol
    );

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.financials(statement_type, frequency).await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
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

    match AsyncTicker::new(&symbol).await {
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
/// Query: (none)
async fn get_holders(Path((symbol, holder_type)): Path<(String, String)>) -> impl IntoResponse {
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

    info!("Fetching {} holders for {}", holder_type, symbol);

    let ticker = match AsyncTicker::new(&symbol).await {
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
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
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
async fn get_analysis(Path((symbol, analysis_type)): Path<(String, String)>) -> impl IntoResponse {
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

    info!("Fetching {} analysis for {}", analysis_type, symbol);

    let ticker = match AsyncTicker::new(&symbol).await {
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
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch {} analysis: {}", analysis_type, e);
            error_response(e).into_response()
        }
    }
}

/// GET /v2/movers/{mover_type}
///
/// Path params:
/// - `mover_type`: gainers, losers, actives
///
/// Query: `count` (u32, default via `MOVERS_COUNT` or server default)
async fn get_movers(
    Path(mover_type): Path<String>,
    Query(params): Query<MoversQuery>,
) -> impl IntoResponse {
    let mt = match MoverType::from_str(&mover_type) {
        Some(t) => t,
        None => {
            let error = serde_json::json!({
                "error": format!("Invalid mover type: '{}'. Valid types: {}", mover_type, MoverType::valid_types()),
                "status": 400
            });
            return (StatusCode::BAD_REQUEST, Json(error)).into_response();
        }
    };

    info!("Fetching {} movers (count={})", mover_type, params.count);

    let result = match mt {
        MoverType::Gainers => finance::gainers(params.count).await,
        MoverType::Losers => finance::losers(params.count).await,
        MoverType::Actives => finance::actives(params.count).await,
    };

    match result {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch {} movers: {}", mover_type, e);
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

/// GET /v2/earnings-transcript
///
/// Query: `event_id` (string, required), `company_id` (string, required)
async fn get_earnings_transcript(
    Query(params): Query<EarningsTranscriptQuery>,
) -> impl IntoResponse {
    info!("Fetching earnings transcript for event {}", params.event_id);

    match finance::earnings_transcript(&params.event_id, &params.company_id).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch earnings transcript: {}", e);
            error_response(e).into_response()
        }
    }
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
