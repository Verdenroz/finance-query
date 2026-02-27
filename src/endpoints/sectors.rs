use super::urls::builders;
use crate::client::YahooClient;
use crate::constants::sectors::Sector;
use crate::error::Result;
use crate::models::sectors::SectorData;

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
pub async fn fetch(client: &YahooClient, sector_type: Sector) -> Result<SectorData> {
    let url = builders::sector(sector_type.as_api_path());
    let response = client.request_with_crumb(&url).await?;
    let json: serde_json::Value = response.json().await?;

    parse_sector_response(&json)
}

/// Parse Yahoo Finance sector response into clean SectorData
fn parse_sector_response(json: &serde_json::Value) -> Result<SectorData> {
    SectorData::from_response(json).map_err(|e| {
        crate::error::FinanceError::ResponseStructureError {
            field: "sector".to_string(),
            context: e,
        }
    })
}
