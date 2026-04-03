//! Market performance endpoints: sector/industry PE, sector performance, gainers/losers/actives.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Sector price-to-earnings ratio entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectorPe {
    /// Date.
    pub date: Option<String>,
    /// Sector name.
    pub sector: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// PE ratio.
    pub pe: Option<f64>,
}

/// Industry price-to-earnings ratio entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndustryPe {
    /// Date.
    pub date: Option<String>,
    /// Industry name.
    pub industry: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// PE ratio.
    pub pe: Option<f64>,
}

/// Sector performance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SectorPerformance {
    /// Sector name.
    pub sector: Option<String>,
    /// Changes percentage.
    #[serde(rename = "changesPercentage")]
    pub changes_percentage: Option<String>,
}

/// Historical sector performance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalSectorPerformance {
    /// Date.
    pub date: Option<String>,
    /// Utilities sector performance.
    #[serde(rename = "utilitiesChangesPercentage")]
    pub utilities_changes_percentage: Option<f64>,
    /// Basic materials sector performance.
    #[serde(rename = "basicMaterialsChangesPercentage")]
    pub basic_materials_changes_percentage: Option<f64>,
    /// Communication services sector performance.
    #[serde(rename = "communicationServicesChangesPercentage")]
    pub communication_services_changes_percentage: Option<f64>,
    /// Consumer cyclical sector performance.
    #[serde(rename = "consumerCyclicalChangesPercentage")]
    pub consumer_cyclical_changes_percentage: Option<f64>,
    /// Consumer defensive sector performance.
    #[serde(rename = "consumerDefensiveChangesPercentage")]
    pub consumer_defensive_changes_percentage: Option<f64>,
    /// Energy sector performance.
    #[serde(rename = "energyChangesPercentage")]
    pub energy_changes_percentage: Option<f64>,
    /// Financial services sector performance.
    #[serde(rename = "financialServicesChangesPercentage")]
    pub financial_services_changes_percentage: Option<f64>,
    /// Healthcare sector performance.
    #[serde(rename = "healthcareChangesPercentage")]
    pub healthcare_changes_percentage: Option<f64>,
    /// Industrials sector performance.
    #[serde(rename = "industrialsChangesPercentage")]
    pub industrials_changes_percentage: Option<f64>,
    /// Real estate sector performance.
    #[serde(rename = "realEstateChangesPercentage")]
    pub real_estate_changes_percentage: Option<f64>,
    /// Technology sector performance.
    #[serde(rename = "technologyChangesPercentage")]
    pub technology_changes_percentage: Option<f64>,
}

/// Market mover entry (gainers, losers, most active).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketMover {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Price change.
    pub change: Option<f64>,
    /// Price.
    pub price: Option<f64>,
    /// Change percentage.
    #[serde(rename = "changesPercentage")]
    pub changes_percentage: Option<f64>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch sector PE ratios.
pub async fn sectors_pe() -> Result<Vec<SectorPe>> {
    let client = build_client()?;
    client
        .get("/api/v4/sector_price_earning_ratio", &[])
        .await
}

/// Fetch industry PE ratios.
pub async fn industries_pe() -> Result<Vec<IndustryPe>> {
    let client = build_client()?;
    client
        .get("/api/v4/industry_price_earning_ratio", &[])
        .await
}

/// Fetch current sector performance.
pub async fn sector_performance() -> Result<Vec<SectorPerformance>> {
    let client = build_client()?;
    client.get("/api/v3/sector-performance", &[]).await
}

/// Fetch historical sector performance.
pub async fn historical_sector_performance(
    limit: u32,
) -> Result<Vec<HistoricalSectorPerformance>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/api/v3/historical-sectors-performance",
            &[("limit", &*limit_str)],
        )
        .await
}

/// Fetch top stock market gainers.
pub async fn stock_market_gainers() -> Result<Vec<MarketMover>> {
    let client = build_client()?;
    client.get("/api/v3/stock_market/gainers", &[]).await
}

/// Fetch top stock market losers.
pub async fn stock_market_losers() -> Result<Vec<MarketMover>> {
    let client = build_client()?;
    client.get("/api/v3/stock_market/losers", &[]).await
}

/// Fetch most active stocks.
pub async fn stock_market_most_active() -> Result<Vec<MarketMover>> {
    let client = build_client()?;
    client.get("/api/v3/stock_market/actives", &[]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sector_performance_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/sector-performance")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "sector": "Technology",
                        "changesPercentage": "1.25%"
                    },
                    {
                        "sector": "Healthcare",
                        "changesPercentage": "-0.45%"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<SectorPerformance> = client
            .get("/api/v3/sector-performance", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].sector.as_deref(), Some("Technology"));
        assert_eq!(resp[0].changes_percentage.as_deref(), Some("1.25%"));
    }

    #[tokio::test]
    async fn test_stock_market_gainers_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/stock_market/gainers")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "XYZ",
                        "name": "XYZ Corp",
                        "change": 5.20,
                        "price": 42.50,
                        "changesPercentage": 13.93
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<MarketMover> = client
            .get("/api/v3/stock_market/gainers", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].symbol.as_deref(), Some("XYZ"));
        assert!((resp[0].change.unwrap() - 5.20).abs() < 0.01);
    }
}
