mod error;
mod tools;

use anyhow::Result;
use clap::Parser;
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

    let handler = FinanceTools::new();

    match cli.transport.as_str() {
        "http" => start_http(cli.addr, handler).await,
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

async fn start_http(addr: String, _handler: FinanceTools) -> Result<()> {
    use axum::{Router, routing::get};
    use rmcp::transport::streamable_http_server::{
        StreamableHttpService, StreamableHttpServerConfig,
        session::local::LocalSessionManager,
    };

    info!("Starting MCP server (HTTP transport) on {addr}");

    let service = StreamableHttpService::new(
        || Ok(FinanceTools::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig {
            stateful_mode: false,
            json_response: true,
            ..Default::default()
        },
    );

    let router = Router::new()
        .nest_service("/mcp", service)
        .route("/health", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on http://{addr}/mcp");

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
