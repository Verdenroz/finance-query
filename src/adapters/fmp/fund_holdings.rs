//! Fund holdings endpoints: ETF sector weightings, country weightings, and holdings.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// ETF sector weighting entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfSectorWeighting {
    /// Sector name.
    pub sector: Option<String>,
    /// Weight percentage (e.g., "7.23%").
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<String>,
}

/// ETF country weighting entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfCountryWeighting {
    /// Country name.
    pub country: Option<String>,
    /// Weight percentage (e.g., "62.15%").
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<String>,
}

/// ETF holding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfHolding {
    /// Asset name / ticker.
    pub asset: Option<String>,
    /// Number of shares held.
    #[serde(rename = "sharesNumber")]
    pub shares_number: Option<f64>,
    /// Weight in ETF as a percentage.
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<f64>,
    /// Market value.
    #[serde(rename = "marketValue")]
    pub market_value: Option<f64>,
    /// Updated date.
    pub updated: Option<String>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch ETF sector weightings.
pub async fn etf_sector_weightings(symbol: &str) -> Result<Vec<EtfSectorWeighting>> {
    let client = build_client()?;
    let path = format!("/api/v3/etf-sector-weightings/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch ETF country weightings.
pub async fn etf_country_weightings(symbol: &str) -> Result<Vec<EtfCountryWeighting>> {
    let client = build_client()?;
    let path = format!("/api/v3/etf-country-weightings/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch ETF holdings (same endpoint as ETF holder).
pub async fn etf_holdings(symbol: &str) -> Result<Vec<EtfHolding>> {
    let client = build_client()?;
    let path = format!("/api/v3/etf-holder/{symbol}");
    client.get(&path, &[]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_etf_sector_weightings_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/etf-sector-weightings/SPY")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "sector": "Technology",
                        "weightPercentage": "29.50%"
                    },
                    {
                        "sector": "Healthcare",
                        "weightPercentage": "13.20%"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfSectorWeighting> = client
            .get("/api/v3/etf-sector-weightings/SPY", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].sector.as_deref(), Some("Technology"));
        assert_eq!(resp[0].weight_percentage.as_deref(), Some("29.50%"));
    }

    #[tokio::test]
    async fn test_etf_country_weightings_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/etf-country-weightings/VEU")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "country": "Japan",
                        "weightPercentage": "15.80%"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfCountryWeighting> = client
            .get("/api/v3/etf-country-weightings/VEU", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].country.as_deref(), Some("Japan"));
    }
}
