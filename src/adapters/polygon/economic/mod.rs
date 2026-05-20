//! Economic indicator endpoints: inflation, labor market, treasury yields.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::economic::{EconomicSeries, MacroObservation};

use super::build_client;
use super::models::PaginatedResponseDTO;

/// Economic data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicDataPointDTO {
    /// Date.
    pub date: Option<String>,
    /// Value.
    pub value: Option<f64>,
    /// Period.
    pub period: Option<String>,
}

/// Fetch inflation data.
pub async fn inflation(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client.get("/v1/indicators/economy/inflation", params).await
}

/// Fetch inflation expectations.
pub async fn inflation_expectations(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/inflation-expectations", params)
        .await
}

/// Fetch labor market data (unemployment, participation, earnings, job openings).
pub async fn labor_market(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/labor-market", params)
        .await
}

/// Fetch US Treasury yield data.
pub async fn treasury_yields(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EconomicDataPointDTO>> {
    let client = build_client()?;
    client
        .get("/v1/indicators/economy/treasury-yields", params)
        .await
}

/// Fetch economic series (canonical) by series ID.
pub async fn fetch_economic_series_response(series_id: &str) -> Result<EconomicSeries> {
    use crate::error::FinanceError;
    let params: &[(&str, &str)] = &[];
    let paginated = match series_id {
        "inflation" => inflation(params).await?,
        "inflation_expectations" => inflation_expectations(params).await?,
        "labor_market" => labor_market(params).await?,
        "treasury_yields" => treasury_yields(params).await?,
        other => {
            return Err(FinanceError::InvalidParameter {
                param: "series_id".to_string(),
                reason: format!("Unknown economic series: {other}"),
            });
        }
    };
    Ok(EconomicSeries {
        series_id: series_id.to_string(),
        title: None,
        units: None,
        frequency: None,
        observations: paginated
            .results
            .unwrap_or_default()
            .into_iter()
            .map(|d| MacroObservation {
                date: d.date.unwrap_or_default(),
                value: d.value,
            })
            .collect(),
    })
}
