use super::urls::builders;
use crate::client::YahooClient;
use crate::error::Result;
use crate::models::industries::Industry;

/// Fetch detailed industry data from Yahoo Finance
///
/// Returns comprehensive industry information including overview, performance,
/// top companies, top performing companies, top growth companies, and research reports.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `industry_key` - The industry key/slug (e.g., "semiconductors", "software-infrastructure")
///
/// # Example
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// let industry = client.get_industry("semiconductors").await?;
/// println!("Industry: {} ({} companies)", industry.name,
///     industry.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
/// for company in industry.top_companies.iter().take(5) {
///     println!("  {} - {:?}", company.symbol, company.name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn fetch(client: &YahooClient, industry_key: &str) -> Result<Industry> {
    let url = builders::industry(industry_key);
    let response = client.request_with_crumb(&url).await?;
    let json: serde_json::Value = response.json().await?;

    parse_industry_response(&json)
}

/// Parse Yahoo Finance industry response into clean Industry
fn parse_industry_response(json: &serde_json::Value) -> Result<Industry> {
    Industry::from_response(json).map_err(|e| crate::error::YahooError::ResponseStructureError {
        field: "industry".to_string(),
        context: e,
    })
}
