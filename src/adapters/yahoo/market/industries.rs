use crate::adapters::yahoo::client::YahooClient;
use crate::error::Result;
use crate::models::market::industries::IndustryData;

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
/// ```ignore
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
/// Delegates to [`YahooClient::get_industry`] for the typed result.
pub async fn fetch(client: &YahooClient, industry_key: &str) -> Result<IndustryData> {
    client.get_industry(industry_key).await
}
