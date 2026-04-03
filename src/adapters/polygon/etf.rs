//! ETF Global partner data: analytics, constituents, fund flows, profiles, taxonomies.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::PaginatedResponse;

/// ETF analytics data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfAnalytics {
    /// Ticker.
    pub ticker: Option<String>,
    /// Name.
    pub name: Option<String>,
    /// Performance data.
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

/// ETF constituent/holding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfConstituent {
    /// Holding ticker.
    pub ticker: Option<String>,
    /// Holding name.
    pub name: Option<String>,
    /// Weight in portfolio.
    pub weight: Option<f64>,
    /// Market value.
    pub market_value: Option<f64>,
    /// Share count.
    pub shares: Option<f64>,
}

/// ETF fund flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfFundFlow {
    /// Ticker.
    pub ticker: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Flow amount.
    pub flow: Option<f64>,
}

/// ETF profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfProfileData {
    /// Ticker.
    pub ticker: Option<String>,
    /// Name.
    pub name: Option<String>,
    /// Issuer.
    pub issuer: Option<String>,
    /// Expense ratio.
    pub expense_ratio: Option<f64>,
    /// Inception date.
    pub inception_date: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Region.
    pub region: Option<String>,
}

/// ETF taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfTaxonomy {
    /// Category.
    pub category: Option<String>,
    /// Subcategory.
    pub subcategory: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Fetch ETF analytics.
pub async fn etf_analytics(params: &[(&str, &str)]) -> Result<PaginatedResponse<EtfAnalytics>> {
    let client = build_client()?;
    client.get("/v3/reference/etfs/analytics", params).await
}

/// Fetch ETF constituents/holdings.
pub async fn etf_constituents(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<EtfConstituent>> {
    let client = build_client()?;
    let path = format!("/v3/reference/etfs/{}/constituents", ticker);
    client.get(&path, params).await
}

/// Fetch ETF fund flows.
pub async fn etf_fund_flows(params: &[(&str, &str)]) -> Result<PaginatedResponse<EtfFundFlow>> {
    let client = build_client()?;
    client.get("/v3/reference/etfs/fund-flows", params).await
}

/// Fetch ETF profiles/exposure.
pub async fn etf_profiles(params: &[(&str, &str)]) -> Result<PaginatedResponse<EtfProfileData>> {
    let client = build_client()?;
    client.get("/v3/reference/etfs/profiles", params).await
}

/// Fetch ETF taxonomies.
pub async fn etf_taxonomies(params: &[(&str, &str)]) -> Result<PaginatedResponse<EtfTaxonomy>> {
    let client = build_client()?;
    client.get("/v3/reference/etfs/taxonomies", params).await
}
