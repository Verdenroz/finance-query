//! ETF Global partner data: analytics, constituents, fund flows, profiles, taxonomies.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::PaginatedResponseDTO;

/// ETF analytics data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfAnalyticsDTO {
    /// Ticker.
    pub composite_ticker: Option<String>,
    /// Name.
    pub name: Option<String>,
    /// Performance data.
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// ETF constituent/holding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfConstituentDTO {
    /// Holding ticker.
    pub constituent_ticker: Option<String>,
    /// Holding name.
    pub constituent_name: Option<String>,
    /// Weight in portfolio.
    pub weight: Option<f64>,
    /// Market value.
    pub market_value: Option<f64>,
    /// Share count.
    pub shares_held: Option<f64>,
    /// ETF ticker holding the constituent.
    pub composite_ticker: Option<String>,
}

/// ETF fund flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfFundFlowDTO {
    /// Ticker.
    pub composite_ticker: Option<String>,
    /// Date.
    pub effective_date: Option<String>,
    /// Flow amount.
    pub fund_flow: Option<f64>,
    /// Net asset value per share.
    pub nav: Option<f64>,
    /// Shares outstanding.
    pub shares_outstanding: Option<f64>,
}

/// ETF profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfProfileDataDTO {
    /// Ticker.
    pub composite_ticker: Option<String>,
    /// Name.
    pub description: Option<String>,
    /// Issuer.
    pub issuer: Option<String>,
    /// Expense ratio.
    pub total_expenses: Option<f64>,
    /// Inception date.
    pub inception_date: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Remaining profile and exposure fields.
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// ETF taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfTaxonomyDTO {
    /// Category.
    pub category: Option<String>,
    /// ETF ticker.
    pub composite_ticker: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Investment objective.
    pub objective: Option<String>,
    /// Geographic region.
    pub region: Option<String>,
    /// Remaining taxonomy fields.
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// Fetch ETF analytics.
pub async fn etf_analytics(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EtfAnalyticsDTO>> {
    let client = build_client()?;
    client.get("/etf-global/v1/analytics", params).await
}

/// Fetch ETF constituents/holdings.
pub async fn etf_constituents(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EtfConstituentDTO>> {
    let client = build_client()?;
    let mut query = vec![("composite_ticker", ticker)];
    query.extend_from_slice(params);
    client.get("/etf-global/v1/constituents", &query).await
}

/// Fetch ETF fund flows.
pub async fn etf_fund_flows(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EtfFundFlowDTO>> {
    let client = build_client()?;
    client.get("/etf-global/v1/fund-flows", params).await
}

/// Fetch ETF profiles/exposure.
pub async fn etf_profiles(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EtfProfileDataDTO>> {
    let client = build_client()?;
    client.get("/etf-global/v1/profiles", params).await
}

/// Fetch ETF taxonomies.
pub async fn etf_taxonomies(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<EtfTaxonomyDTO>> {
    let client = build_client()?;
    client.get("/etf-global/v1/taxonomies", params).await
}
