use crate::adapters::yahoo::client::YahooClient;
use crate::constants::sectors::Sector;
use crate::error::Result;
use crate::models::market::sectors::SectorData;

/// Fetch detailed sector data from Yahoo Finance
///
/// Returns comprehensive sector information including overview, performance,
/// top companies, ETFs, mutual funds, industries, and research reports.
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `sector_type` - The sector to fetch data for
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::Sector;
/// let sector = client.get_sector(Sector::Technology).await?;
/// println!("Sector: {} ({} companies)", sector.name,
///     sector.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
/// for company in sector.top_companies.iter().take(5) {
///     println!("  {} - {:?}", company.symbol, company.name);
/// }
/// # Ok(())
/// # }
/// ```
/// Delegates to [`YahooClient::get_sector`] for the typed result.
pub async fn fetch(client: &YahooClient, sector_type: Sector) -> Result<SectorData> {
    client.get_sector(sector_type).await
}
