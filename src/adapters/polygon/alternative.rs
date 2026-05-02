//! Alternative data: consumer spending (Fable Data merchant aggregates and hierarchy).

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::PaginatedResponse;

/// Merchant aggregate data (European consumer spending).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MerchantAggregate {
    /// Merchant name.
    pub merchant: Option<String>,
    /// Parent company.
    pub parent: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Transaction count.
    pub transaction_count: Option<f64>,
    /// Average transaction value.
    pub avg_transaction_value: Option<f64>,
    /// Total spend.
    pub total_spend: Option<f64>,
}

/// Merchant hierarchy entry (merchant-to-parent mapping).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MerchantHierarchy {
    /// Merchant name.
    pub merchant: Option<String>,
    /// Parent company name.
    pub parent: Option<String>,
    /// Ticker symbol (if public).
    pub ticker: Option<String>,
    /// Category.
    pub category: Option<String>,
}

/// Fetch merchant aggregate spending data.
pub async fn merchant_aggregates(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<MerchantAggregate>> {
    let client = build_client()?;
    client
        .get("/v1/alternative/merchant-aggregates", params)
        .await
}

/// Fetch merchant hierarchy (merchant-to-parent mappings).
pub async fn merchant_hierarchy(
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<MerchantHierarchy>> {
    let client = build_client()?;
    client
        .get("/v1/alternative/merchant-hierarchy", params)
        .await
}
