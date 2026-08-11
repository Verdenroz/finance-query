use axum::{
    Router,
    extract::Extension,
    http::{HeaderValue, Method},
    middleware,
};
use finance_query_server::{
    AppState, FeedHub, StreamHub,
    cache::Cache,
    graphql, metrics,
    rate_limit::{RateLimitConfig, RateLimiterState, rate_limit_middleware},
};
use std::net::{IpAddr, SocketAddr};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod handlers;

/// Default server port, overridable via the `PORT` env var.
const DEFAULT_SERVER_PORT: u16 = 8000;

/// Default bind address, overridable via the `HOST` env var. A deployment
/// serves the world; an SDK-spawned copy sets `HOST=127.0.0.1` so the
/// consumer's machine is the only thing that can reach it.
const DEFAULT_SERVER_HOST: &str = "0.0.0.0";

/// Prefix of the embedded-server readiness line.
///
/// An SDK that bundles this binary spawns it and has to learn where it
/// bound — guessing a port races, so we bind first and say so afterwards.
/// Deliberately plain text on stdout: launchers scan for this prefix and
/// ignore every other line, which is what lets the server log freely and
/// costs us no dependency to participate.
const READY_PREFIX: &str = "soothfast-ready ";

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    init_tracing();

    // Initialize metrics
    metrics::init();

    info!("Finance Query server initializing...");

    // Warm the offline translation model in the background so the first
    // translated request doesn't pay the one-time download/load cost. Runs
    // concurrently with serving: health checks pass while the model loads.
    #[cfg(feature = "translation-offline")]
    tokio::spawn(async {
        match finance_query::translation::preload().await {
            Ok(()) => info!("Offline translation model preloaded"),
            Err(e) => warn!("Translation model preload failed: {}", e),
        }
    });

    // Build application with routes
    let app = create_app().await;

    // Determine server address. Port 0 is how an embedding SDK asks for
    // whatever port is free, which is why the announcement below reports
    // the bound address rather than this one.
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_SERVER_PORT);
    let host: IpAddr = std::env::var("HOST")
        .unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_string())
        .parse()
        .expect("HOST must be an IP address");
    let addr = SocketAddr::new(host, port);

    info!("🚀 Starting Finance Query on {}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    let bound = listener.local_addr().expect("bound address");
    announce(&reachable_url(bound));

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");
}

/// A URL a client on this machine can actually reach.
///
/// `0.0.0.0` means "every interface", which is not an address anything can
/// connect *to* — an SDK handed that would fail on Windows and on any host
/// with no default route.
fn reachable_url(addr: SocketAddr) -> String {
    match addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => format!("http://127.0.0.1:{}", addr.port()),
        IpAddr::V6(ip) if ip.is_unspecified() => format!("http://[::1]:{}", addr.port()),
        IpAddr::V4(ip) => format!("http://{ip}:{}", addr.port()),
        IpAddr::V6(ip) => format!("http://[{ip}]:{}", addr.port()),
    }
}

/// Tell a waiting SDK launcher where we bound.
///
/// Harmless when nobody is listening — one line on stdout — so it is
/// unconditional rather than gated on an "am I embedded?" flag we would
/// have to plumb through and keep honest.
fn announce(base_url: &str) {
    use std::io::Write;

    let mut stdout = std::io::stdout().lock();
    let _ = writeln!(stdout, "{READY_PREFIX}{{\"base_url\":\"{base_url}\"}}");
    let _ = stdout.flush();
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
        "Rate limiting enabled: {} requests/minute per IP",
        rate_limit_config.requests_per_minute
    );

    let state = AppState {
        cache,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
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
        .nest("/v2", handlers::api_routes())
        // GraphQL endpoints at root (not under /v2 — different versioning story)
        .merge(graphql::graphql_routes(schema.clone()))
        .layer(Extension(schema))
        .layer(Extension(state))
        .route_layer(middleware::from_fn(handlers::metrics_middleware))
        .layer(middleware::from_fn_with_state(
            rate_limiter,
            rate_limit_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
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
            "finance_query={log_level},server={log_level},tower_http={log_level},axum::rejection=trace"
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The wildcard bind is the whole reason this function exists: an SDK
    /// handed `http://0.0.0.0:…` cannot connect to it on Windows.
    #[test]
    fn a_wildcard_bind_is_announced_as_loopback() {
        let addr: SocketAddr = "0.0.0.0:8000".parse().expect("parses");
        assert_eq!(reachable_url(addr), "http://127.0.0.1:8000");

        let addr: SocketAddr = "[::]:8000".parse().expect("parses");
        assert_eq!(reachable_url(addr), "http://[::1]:8000");
    }

    #[test]
    fn a_specific_bind_is_announced_as_itself() {
        let addr: SocketAddr = "127.0.0.1:54321".parse().expect("parses");
        assert_eq!(reachable_url(addr), "http://127.0.0.1:54321");

        let addr: SocketAddr = "[::1]:54321".parse().expect("parses");
        assert_eq!(reachable_url(addr), "http://[::1]:54321");
    }
}
