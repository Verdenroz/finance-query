mod error;
mod lang;
mod metrics;
mod tools;

use anyhow::Result;
use clap::Parser;
use finance_query_server::{AppState, FeedHub, StreamHub, cache::Cache, graphql};
use tools::FinanceTools;
use tracing::info;

#[derive(Parser)]
#[command(name = "fq-mcp", about = "finance-query MCP server")]
struct Cli {
    /// Transport mode: "stdio" (default, for local development) or "http" (for VPS deployment)
    #[arg(long, default_value = "stdio", env = "MCP_TRANSPORT")]
    transport: String,

    /// HTTP bind address (only used when transport=http)
    #[arg(long, default_value = "0.0.0.0:3000", env = "MCP_ADDR")]
    addr: String,

    /// Comma-separated `Host` header values to accept on the HTTP transport
    /// (DNS-rebinding protection). Loopback addresses are always allowed;
    /// public deployments must add their domain (e.g. "finance-query.com").
    #[arg(long, env = "MCP_ALLOWED_HOSTS", value_delimiter = ',')]
    allowed_hosts: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // When run from workspace root (e.g. `cargo run -p finance-query-mcp`),
    // try the crate-local .env first, then fall back to the standard search.
    dotenvy::from_path("finance-query-mcp/.env").ok();
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fq_mcp=info".parse()?)
                .add_directive("rmcp=warn".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    init_edgar();
    init_fred();
    // Both transports record tool metrics; only http exports them via /metrics.
    metrics::init();

    // Warm the offline translation model in the background so the first
    // tool call with a `lang` param doesn't pay the one-time load cost.
    #[cfg(feature = "translation-offline")]
    tokio::spawn(async {
        match finance_query::translation::preload().await {
            Ok(()) => info!("Offline translation model preloaded"),
            Err(e) => tracing::warn!("Translation model preload failed: {e}"),
        }
    });

    // MCP is stateless — never connect to Redis, always fetch fresh.
    let cache = Cache::new(None).await;
    let state = AppState {
        cache,
        stream_hub: StreamHub::new(),
        feed_hub: FeedHub::new(),
    };
    let schema = graphql::build_schema(state);
    let handler = FinanceTools::new(schema);

    match cli.transport.as_str() {
        "http" => start_http(cli.addr, cli.allowed_hosts, handler).await,
        _ => start_stdio(handler).await,
    }
}

async fn start_stdio(handler: FinanceTools) -> Result<()> {
    use rmcp::ServiceExt;

    info!("Starting MCP server (stdio transport)");
    let (stdin, stdout) = rmcp::transport::io::stdio();
    let service = handler.serve((stdin, stdout)).await?;
    service.waiting().await?;
    Ok(())
}

async fn start_http(
    addr: String,
    extra_allowed_hosts: Vec<String>,
    handler: FinanceTools,
) -> Result<()> {
    use axum::{Router, routing::get};
    use rmcp::transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    };

    info!("Starting MCP server (HTTP transport) on {addr}");

    // rmcp defaults `allowed_hosts` to loopback only, rejecting any other
    // `Host` header as a DNS-rebinding protection. Behind a reverse proxy
    // (Caddy) the original public Host header is forwarded through, so the
    // public domain must be added explicitly or every request 403s.
    let mut allowed_hosts = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    allowed_hosts.extend(extra_allowed_hosts);
    if allowed_hosts.len() > 3 {
        info!(hosts = ?allowed_hosts, "HTTP transport allowed hosts configured");
    }

    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default()
            .with_stateful_mode(false)
            .with_json_response(true)
            .with_allowed_hosts(allowed_hosts),
    );

    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        // Unauthenticated by design: Caddy blocks /mcp/metrics at the edge,
        // so this route is only reachable from inside the docker network.
        .route("/metrics", get(|| async { metrics::gather() }))
        .fallback_service(service);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on http://{addr}/");

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap_or_default();
        })
        .await?;

    Ok(())
}

fn init_edgar() {
    if let Ok(email) = std::env::var("EDGAR_EMAIL") {
        match finance_query::edgar::init_with_config(
            email,
            "finance-query-mcp",
            std::time::Duration::from_secs(30),
        ) {
            Ok(_) => info!("EDGAR client initialized"),
            Err(e) => tracing::warn!("Failed to initialize EDGAR client: {e}"),
        }
    } else {
        info!("EDGAR client not configured (set EDGAR_EMAIL to enable)");
    }
}

fn init_fred() {
    if let Ok(key) = std::env::var("FRED_API_KEY") {
        match finance_query::fred::init(key) {
            Ok(_) => info!("FRED client initialized"),
            Err(e) => tracing::warn!("Failed to initialize FRED client: {e}"),
        }
    } else {
        info!("FRED client not configured (set FRED_API_KEY to enable)");
    }
}
