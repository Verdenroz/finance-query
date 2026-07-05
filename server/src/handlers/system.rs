use axum::response::{IntoResponse, Json};
use finance_query_server::metrics;
use serde::Serialize;

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

/// GET /health
///
/// Query: (none)
pub(crate) async fn health_check() -> impl IntoResponse {
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
pub(crate) async fn ping() -> impl IntoResponse {
    let response = PingResponse {
        message: "pong".to_string(),
    };

    Json(response)
}

/// GET /metrics
///
/// Prometheus metrics endpoint in text format
pub(crate) async fn get_metrics() -> impl IntoResponse {
    let metrics = metrics::gather();
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics,
    )
}

/// Metrics middleware to track request counts and latencies
pub(crate) async fn metrics_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = request.method().to_string();
    // Route template, not raw URI, to keep label cardinality bounded.
    let path = request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let timer = metrics::RequestTimer::new(method, path);

    let response = next.run(request).await;
    let status = response.status().as_u16();

    timer.observe(status);

    response
}
