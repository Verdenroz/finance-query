//! Fund holdings endpoints: ETF sector weightings, country weightings, and holdings.

use serde::{Deserialize, Serialize};

use crate::adapters::common::percent::strip_percent;
use crate::error::Result;
use crate::models::fundamentals::{
    EtfCountryWeighting, EtfHolding, EtfProfile, EtfSectorWeighting,
};

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// ETF sector weighting entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfSectorWeightingDTO {
    /// Sector name.
    pub sector: Option<String>,
    /// Weight percentage (e.g., `7.23`).
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<f64>,
}

/// ETF country weighting entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfCountryWeightingDTO {
    /// Country name.
    pub country: Option<String>,
    /// Weight percentage (e.g., "62.15%").
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<String>,
}

/// ETF holding entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfHoldingDTO {
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
pub async fn etf_sector_weightings(symbol: &str) -> Result<Vec<EtfSectorWeightingDTO>> {
    let client = build_client()?;
    client
        .get("/stable/etf/sector-weightings", &[("symbol", symbol)])
        .await
}

/// Fetch ETF country weightings.
pub async fn etf_country_weightings(symbol: &str) -> Result<Vec<EtfCountryWeightingDTO>> {
    let client = build_client()?;
    client
        .get("/stable/etf/country-weightings", &[("symbol", symbol)])
        .await
}

/// Fetch ETF holdings (same endpoint as ETF holder).
pub async fn etf_holdings(symbol: &str) -> Result<Vec<EtfHoldingDTO>> {
    let client = build_client()?;
    client
        .get("/stable/etf/holdings", &[("symbol", symbol)])
        .await
}

// ============================================================================
// Canonical conversions
// ============================================================================

/// Convert a holding entry into a canonical [`EtfHolding`]. FMP reports
/// weight as a percentage (e.g. `12.5`); the canonical field is a fraction.
fn to_etf_holding(dto: EtfHoldingDTO) -> EtfHolding {
    EtfHolding {
        symbol: dto.asset.clone(),
        description: dto.asset,
        weight: dto.weight_percentage.map(|w| w / 100.0),
    }
}

fn to_sector_weighting(dto: EtfSectorWeightingDTO) -> EtfSectorWeighting {
    EtfSectorWeighting {
        sector: dto.sector,
        weight: dto.weight_percentage.map(|w| w / 100.0),
    }
}

fn to_country_weighting(dto: EtfCountryWeightingDTO) -> EtfCountryWeighting {
    EtfCountryWeighting {
        country: dto.country,
        weight: dto
            .weight_percentage
            .as_deref()
            .and_then(strip_percent)
            .map(|w| w / 100.0),
    }
}

/// Fetch the canonical ETF profile for a symbol. FMP has no dedicated
/// ETF-profile endpoint, so `name` comes from the plain quote and the
/// remaining profile-level fields (`net_assets`, `net_expense_ratio`,
/// `portfolio_turnover`, `dividend_yield`, `inception_date`) stay `None`.
pub async fn fetch_etf_profile_response(symbol: &str) -> Result<EtfProfile> {
    let (quotes, holdings, sectors, countries) = tokio::try_join!(
        crate::adapters::fmp::quote::quote(symbol),
        etf_holdings(symbol),
        etf_sector_weightings(symbol),
        etf_country_weightings(symbol)
    )?;
    let name = quotes.into_iter().next().and_then(|q| q.name);
    Ok(EtfProfile {
        symbol: Some(symbol.to_string()),
        name,
        holdings: holdings.into_iter().map(to_etf_holding).collect(),
        sector_weightings: sectors.into_iter().map(to_sector_weighting).collect(),
        country_weightings: countries.into_iter().map(to_country_weighting).collect(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_etf_holding_weight_percent_to_fraction() {
        let dto: EtfHoldingDTO = serde_json::from_value(serde_json::json!({
            "asset": "AAPL",
            "sharesNumber": 170000000.0,
            "weightPercentage": 7.2,
            "marketValue": 31450000000.0
        }))
        .unwrap();

        let out = to_etf_holding(dto);
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert!((out.weight.unwrap() - 0.072).abs() < 1e-9);
    }

    #[test]
    fn maps_sector_and_country_weightings() {
        let sector: EtfSectorWeightingDTO = serde_json::from_value(serde_json::json!({
            "sector": "Technology",
            "weightPercentage": 29.50
        }))
        .unwrap();
        let out = to_sector_weighting(sector);
        assert_eq!(out.sector.as_deref(), Some("Technology"));
        assert!((out.weight.unwrap() - 0.295).abs() < 1e-9);

        let country: EtfCountryWeightingDTO = serde_json::from_value(serde_json::json!({
            "country": "Japan",
            "weightPercentage": "15.80%"
        }))
        .unwrap();
        let out = to_country_weighting(country);
        assert_eq!(out.country.as_deref(), Some("Japan"));
        assert!((out.weight.unwrap() - 0.158).abs() < 1e-9);
    }

    #[tokio::test]
    async fn test_etf_sector_weightings_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/etf/sector-weightings")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "sector": "Technology",
                        "weightPercentage": 29.50
                    },
                    {
                        "sector": "Healthcare",
                        "weightPercentage": 13.20
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfSectorWeightingDTO> = client
            .get("/stable/etf/sector-weightings", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 2);
        assert_eq!(resp[0].sector.as_deref(), Some("Technology"));
        assert_eq!(resp[0].weight_percentage, Some(29.50));
    }

    #[tokio::test]
    async fn test_etf_country_weightings_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/etf/country-weightings")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfCountryWeightingDTO> = client
            .get("/stable/etf/country-weightings", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].country.as_deref(), Some("Japan"));
    }
}
