use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
};
use finance_query::{AsyncTicker, Interval, TimeRange, YahooError, finance};
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
    /// Default number of news articles to return
    pub const NEWS_COUNT: u32 = 10;
    /// Default chart interval
    pub const DEFAULT_INTERVAL: &str = "1d";
    /// Default chart range
    pub const DEFAULT_RANGE: &str = "1mo";
    /// Default start period for timeseries (Unix timestamp)
    pub const DEFAULT_PERIOD1: i64 = 0;
    /// Default end period for timeseries (Unix timestamp)
    pub const DEFAULT_PERIOD2: i64 = 9999999999;
    /// Default server port
    pub const SERVER_PORT: u16 = 8000;
    /// Default movers count
    pub const MOVERS_COUNT: u32 = 10;
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct PingResponse {
    message: String,
}

// Query parameter structs
#[derive(Deserialize)]
struct QuotesQuery {
    symbols: String, // Comma-separated symbols
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
struct NewsQuery {
    #[serde(default = "default_news_count")]
    count: u32,
}

#[derive(Deserialize)]
struct OptionsQuery {
    date: Option<i64>, // Optional expiration timestamp
}

#[derive(Deserialize)]
struct TimeseriesQuery {
    #[serde(default = "default_period1")]
    period1: i64,
    #[serde(default = "default_period2")]
    period2: i64,
    types: String, // Comma-separated fundamental types
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

fn default_news_count() -> u32 {
    std::env::var("NEWS_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::NEWS_COUNT)
}

fn default_interval() -> String {
    std::env::var("DEFAULT_INTERVAL").unwrap_or_else(|_| defaults::DEFAULT_INTERVAL.to_string())
}

fn default_range() -> String {
    std::env::var("DEFAULT_RANGE").unwrap_or_else(|_| defaults::DEFAULT_RANGE.to_string())
}

fn default_period1() -> i64 {
    std::env::var("DEFAULT_PERIOD1")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::DEFAULT_PERIOD1)
}

fn default_period2() -> i64 {
    std::env::var("DEFAULT_PERIOD2")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(defaults::DEFAULT_PERIOD2)
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
        .route("/health", get(health_check))
        .route("/ping", get(ping))
        // Nest all API routes under /v2
        .nest("/v2", api_routes())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

/// API routes
fn api_routes() -> Router {
    Router::new()
        // Core quotes
        .route("/quote/{symbol}", get(get_quote))
        .route("/quotes", get(get_quotes))
        .route("/recommendations/{symbol}", get(get_recommendations))
        .route("/chart/{symbol}", get(get_chart))
        .route("/indicators/{symbol}", get(get_indicators))
        .route("/search", get(search))
        // News & Options
        .route("/news/{symbol}", get(get_news))
        .route("/options/{symbol}", get(get_options))
        // Financials
        .route("/financials/{symbol}", get(get_financials))
        .route("/timeseries/{symbol}", get(get_timeseries))
        .route("/quote-type/{symbol}", get(get_quote_type))
        // Holders
        .route("/holders/{symbol}/major", get(get_major_holders))
        .route(
            "/holders/{symbol}/institutional",
            get(get_institutional_holders),
        )
        .route("/holders/{symbol}/mutualfund", get(get_mutualfund_holders))
        .route(
            "/holders/{symbol}/insider-transactions",
            get(get_insider_transactions),
        )
        .route(
            "/holders/{symbol}/insider-purchases",
            get(get_insider_purchases),
        )
        .route("/holders/{symbol}/insider-roster", get(get_insider_roster))
        // Analysis
        .route(
            "/analysis/{symbol}/recommendations",
            get(get_recommendation_trend),
        )
        .route(
            "/analysis/{symbol}/upgrades-downgrades",
            get(get_upgrades_downgrades),
        )
        .route(
            "/analysis/{symbol}/earnings-estimate",
            get(get_earnings_estimate),
        )
        .route(
            "/analysis/{symbol}/earnings-history",
            get(get_earnings_history),
        )
        // Market Movers
        .route("/movers/gainers", get(get_gainers))
        .route("/movers/losers", get(get_losers))
        .route("/movers/actives", get(get_actives))
        // Earnings Transcript
        .route("/earnings-transcript", get(get_earnings_transcript))
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(response)
}

/// Ping endpoint
async fn ping() -> impl IntoResponse {
    let response = PingResponse {
        message: "pong".to_string(),
    };

    Json(response)
}

/// Get quote for a symbol
async fn get_quote(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Received quote request for symbol: {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.quote().await {
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

/// Get detailed quotes for multiple symbols
async fn get_quotes(Query(params): Query<QuotesQuery>) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    info!("Fetching detailed quotes for {} symbols", symbols.len());

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for symbol in symbols {
        match AsyncTicker::new(symbol).await {
            Ok(ticker) => match ticker.quote().await {
                Ok(quote) => {
                    results.push(quote);
                }
                Err(e) => {
                    error!("Failed to fetch quote for {}: {}", symbol, e);
                    errors.push(serde_json::json!({
                        "symbol": symbol,
                        "error": e.to_string()
                    }));
                }
            },
            Err(e) => {
                error!("Failed to fetch quote for {}: {}", symbol, e);
                errors.push(serde_json::json!({
                    "symbol": symbol,
                    "error": e.to_string()
                }));
            }
        }
    }

    let response = serde_json::json!({
        "quotes": results,
        "errors": errors
    });
    (StatusCode::OK, Json(response)).into_response()
}

/// Get recommended/similar quotes for a symbol
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

/// Get chart data
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

/// Get technical indicators
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

/// Search for symbols
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

/// Get news for a symbol
async fn get_news(
    Path(symbol): Path<String>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    info!("Fetching news for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.news(params.count).await {
            Ok(news_response) => (StatusCode::OK, Json(news_response)).into_response(),
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

/// Get options chain for a symbol
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

/// Get financial statements for a symbol
async fn get_financials(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching financials for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.financial_data().await {
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

/// Get fundamentals timeseries data
async fn get_timeseries(
    Path(symbol): Path<String>,
    Query(params): Query<TimeseriesQuery>,
) -> impl IntoResponse {
    info!(
        "Fetching timeseries for {} with types: {}",
        symbol, params.types
    );

    // Parse types from comma-separated string
    let types: Vec<&str> = params.types.split(',').map(|s| s.trim()).collect();

    if types.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "No types provided for timeseries"
            })),
        )
            .into_response();
    }

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => {
            match ticker
                .timeseries(&types, params.period1, params.period2)
                .await
            {
                Ok(response) => (StatusCode::OK, Json(response)).into_response(),
                Err(e) => {
                    error!("Failed to fetch timeseries: {}", e);
                    error_response(e).into_response()
                }
            }
        }
        Err(e) => {
            error!("Failed to create ticker: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get quote type metadata for a symbol
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

/// Get major holders breakdown
async fn get_major_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching major holders for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.major_holders().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch major holders: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch major holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get institutional holders
async fn get_institutional_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching institutional holders for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.institution_ownership().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch institutional holders: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch institutional holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get mutual fund holders
async fn get_mutualfund_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching mutual fund holders for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.fund_ownership().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch mutual fund holders: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch mutual fund holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider transactions
async fn get_insider_transactions(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider transactions for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.insider_transactions().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch insider transactions: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch insider transactions: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider purchases summary
async fn get_insider_purchases(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider purchases for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.share_purchase_activity().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch insider purchases: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch insider purchases: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider roster
async fn get_insider_roster(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider roster for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.insider_holders().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch insider roster: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch insider roster: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get analyst recommendations trend
async fn get_recommendation_trend(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching recommendation trend for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.recommendation_trend().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch recommendation trend: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch recommendation trend: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get analyst upgrades and downgrades
async fn get_upgrades_downgrades(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching upgrades/downgrades for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.grading_history().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch upgrades/downgrades: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch upgrades/downgrades: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get earnings estimates
async fn get_earnings_estimate(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching earnings estimate for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.earnings_trend().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch earnings estimate: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch earnings estimate: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get historical earnings
async fn get_earnings_history(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching earnings history for {}", symbol);

    match AsyncTicker::new(&symbol).await {
        Ok(ticker) => match ticker.earnings_history().await {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to fetch earnings history: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to fetch earnings history: {}", e);
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

/// Get top gaining stocks
async fn get_gainers(Query(params): Query<MoversQuery>) -> impl IntoResponse {
    info!("Fetching top {} gainers", params.count);

    match finance::gainers(params.count).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch gainers: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get top losing stocks
async fn get_losers(Query(params): Query<MoversQuery>) -> impl IntoResponse {
    info!("Fetching top {} losers", params.count);

    match finance::losers(params.count).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch losers: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get most active stocks
async fn get_actives(Query(params): Query<MoversQuery>) -> impl IntoResponse {
    info!("Fetching top {} most active stocks", params.count);

    match finance::actives(params.count).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => {
            error!("Failed to fetch most active stocks: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get earnings call transcript
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
