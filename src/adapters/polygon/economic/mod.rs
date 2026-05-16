//! Economic indicator endpoints: inflation, labor market, treasury yields.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::Result;

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
