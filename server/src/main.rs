use axum::{
    Router,
    extract::Path,
    http::{HeaderValue, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::get,
};
use finance_query::{Error as YahooError, Ticker};
use serde::Serialize;
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
        .unwrap_or(8000);
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
        .route("/health", get(health_check))
        .route("/ping", get(ping))
        .route("/quote/{symbol}", get(get_quote))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
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
