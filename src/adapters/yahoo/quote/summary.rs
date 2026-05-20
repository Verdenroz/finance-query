/// Quote summary endpoint
///
/// Fetches full quote summary data for a single symbol.
/// Uses the /v10/finance/quoteSummary endpoint with all available modules.
use crate::adapters::yahoo::client::YahooClient;
use crate::adapters::yahoo::endpoints::api;
use crate::error::Result;
use crate::models::quote::QuoteSummaryResponse;
use tracing::info;

/// Fetch full quote summary for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
pub(crate) async fn fetch_summary(
    client: &YahooClient,
    symbol: &str,
) -> Result<QuoteSummaryResponse> {
    info!("Fetching quote summary for: {}", symbol);

    let base_url = api::quote_summary(symbol);
    let modules = crate::models::quote::Module::all()
        .iter()
        .map(|m| m.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let url = format!("{base_url}?modules={modules}");
    let resp = client.request_with_crumb(&url).await?;
    let json: serde_json::Value = resp.json().await?;
    QuoteSummaryResponse::from_json(json, symbol)
}
