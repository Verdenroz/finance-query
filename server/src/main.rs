use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
};
use finance_query::{
    ClientConfig, Error as YahooError, Interval, Ticker, TimeRange, YahooClient, defaults,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
struct SimilarQuery {
    symbol: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Deserialize)]
struct HistoricalQuery {
    symbol: String,
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

fn default_limit() -> u32 {
    defaults::SIMILAR_STOCKS_LIMIT
}
fn default_hits() -> u32 {
    defaults::SEARCH_HITS
}
fn default_interval() -> String {
    "1d".to_string()
}
fn default_range() -> String {
    "1mo".to_string()
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
        .route("/similar", get(get_similar))
        .route("/historical", get(get_historical))
        .route("/search", get(search))
        // Financials
        .route("/financials/{symbol}", get(get_financials))
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
            get(get_recommendations),
        )
        .route(
            "/analysis/{symbol}/upgrades-downgrades",
            get(get_upgrades_downgrades),
        )
        .route("/analysis/{symbol}/price-targets", get(get_price_targets))
        .route(
            "/analysis/{symbol}/earnings-estimate",
            get(get_earnings_estimate),
        )
        .route(
            "/analysis/{symbol}/revenue-estimate",
            get(get_revenue_estimate),
        )
        .route(
            "/analysis/{symbol}/earnings-history",
            get(get_earnings_history),
        )
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

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            info!("Successfully fetched quote for {}", symbol);
            // Convert ticker data to JSON for response
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "summaryDetail": ticker.summary_detail(),
                "financialData": ticker.financial_data(),
                "keyStats": ticker.key_stats(),
                "assetProfile": ticker.asset_profile(),
                "calendarEvents": ticker.calendar_events(),
                "earnings": ticker.earnings(),
                "earningsHistory": ticker.earnings_history(),
                "earningsTrend": ticker.earnings_trend(),
                "institutionOwnership": ticker.institution_ownership(),
                "fundOwnership": ticker.fund_ownership(),
                "majorHolders": ticker.major_holders(),
                "insiderHolders": ticker.insider_holders(),
                "insiderTransactions": ticker.insider_transactions(),
                "recommendationTrend": ticker.recommendation_trend(),
                "gradingHistory": ticker.grading_history(),
                "secFilings": ticker.sec_filings(),
                "sharePurchaseActivity": ticker.share_purchase_activity(),
                "quoteType": ticker.quote_type(),
                "summaryProfile": ticker.summary_profile(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch quote for {}: {}", symbol, e);
            let status = match e {
                YahooError::SymbolNotFound(_) => StatusCode::NOT_FOUND,
                YahooError::AuthenticationFailed => StatusCode::UNAUTHORIZED,
                YahooError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let error_response = serde_json::json!({
                "error": e.to_string(),
                "status": status.as_u16()
            });
            (status, Json(error_response)).into_response()
        }
    }
}

// Helper to convert error to response
fn error_response(e: YahooError) -> impl IntoResponse {
    let status = match e {
        YahooError::SymbolNotFound(_) => StatusCode::NOT_FOUND,
        YahooError::AuthenticationFailed => StatusCode::UNAUTHORIZED,
        YahooError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
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
        match Ticker::new(symbol).await {
            Ok(ticker) => {
                results.push(serde_json::json!({
                    "symbol": ticker.symbol(),
                    "summaryDetail": ticker.summary_detail(),
                    "financialData": ticker.financial_data(),
                    "keyStats": ticker.key_stats(),
                    "assetProfile": ticker.asset_profile(),
                    "calendarEvents": ticker.calendar_events(),
                    "earnings": ticker.earnings(),
                    "recommendationTrend": ticker.recommendation_trend(),
                }));
            }
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

/// Get similar quotes for a symbol
async fn get_similar(Query(params): Query<SimilarQuery>) -> impl IntoResponse {
    info!("Fetching similar quotes for {}", params.symbol);

    match YahooClient::new(ClientConfig::default()).await {
        Ok(client) => match client
            .get_similar_quotes(&params.symbol, params.limit)
            .await
        {
            Ok(json) => (StatusCode::OK, Json(json)).into_response(),
            Err(e) => {
                error!("Failed to fetch similar quotes: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create client: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get historical chart data
async fn get_historical(Query(params): Query<HistoricalQuery>) -> impl IntoResponse {
    let interval = parse_interval(&params.interval);
    let range = parse_range(&params.range);
    info!("Fetching historical data for {}", params.symbol);

    match YahooClient::new(ClientConfig::default()).await {
        Ok(client) => match client.get_chart(&params.symbol, interval, range).await {
            Ok(json) => (StatusCode::OK, Json(json)).into_response(),
            Err(e) => {
                error!("Failed to fetch historical data: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create client: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Search for symbols
async fn search(Query(params): Query<SearchQuery>) -> impl IntoResponse {
    info!("Searching for: {}", params.q);

    match YahooClient::new(ClientConfig::default()).await {
        Ok(client) => match client.search(&params.q, params.hits).await {
            Ok(json) => (StatusCode::OK, Json(json)).into_response(),
            Err(e) => {
                error!("Search failed: {}", e);
                error_response(e).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create client: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get financial statements for a symbol
async fn get_financials(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching financials for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "financialData": ticker.financial_data(),
                "earnings": ticker.earnings(),
                "earningsHistory": ticker.earnings_history(),
                "earningsTrend": ticker.earnings_trend(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch financials: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get major holders breakdown
async fn get_major_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching major holders for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "majorHolders": ticker.major_holders(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch major holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get institutional holders
async fn get_institutional_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching institutional holders for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "institutionOwnership": ticker.institution_ownership(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch institutional holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get mutual fund holders
async fn get_mutualfund_holders(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching mutual fund holders for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "fundOwnership": ticker.fund_ownership(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch mutual fund holders: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider transactions
async fn get_insider_transactions(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider transactions for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "insiderTransactions": ticker.insider_transactions(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch insider transactions: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider purchases summary
async fn get_insider_purchases(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider purchases for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "sharePurchaseActivity": ticker.share_purchase_activity(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch insider purchases: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get insider roster
async fn get_insider_roster(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching insider roster for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "insiderHolders": ticker.insider_holders(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch insider roster: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get analyst recommendations
async fn get_recommendations(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching recommendations for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "recommendationTrend": ticker.recommendation_trend(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch recommendations: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get analyst upgrades and downgrades
async fn get_upgrades_downgrades(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching upgrades/downgrades for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "gradingHistory": ticker.grading_history(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch upgrades/downgrades: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get analyst price targets
async fn get_price_targets(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching price targets for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            // Price targets are part of financial data
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "financialData": ticker.financial_data(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch price targets: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get earnings estimates
async fn get_earnings_estimate(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching earnings estimate for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "earningsTrend": ticker.earnings_trend(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch earnings estimate: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get revenue estimates
async fn get_revenue_estimate(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching revenue estimate for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "earningsTrend": ticker.earnings_trend(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch revenue estimate: {}", e);
            error_response(e).into_response()
        }
    }
}

/// Get historical earnings
async fn get_earnings_history(Path(symbol): Path<String>) -> impl IntoResponse {
    info!("Fetching earnings history for {}", symbol);

    match Ticker::new(&symbol).await {
        Ok(ticker) => {
            let response = serde_json::json!({
                "symbol": ticker.symbol(),
                "earningsHistory": ticker.earnings_history(),
            });
            (StatusCode::OK, Json(response)).into_response()
        }
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
