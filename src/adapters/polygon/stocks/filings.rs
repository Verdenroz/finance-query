//! SEC filing endpoints: 10-K sections, 8-K text, EDGAR index, risk factors.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::PaginatedResponse;

/// SEC filing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingEntry {
    /// Accession number.
    pub accession_number: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
    /// Filing type (e.g., `"10-K"`, `"8-K"`).
    pub filing_type: Option<String>,
    /// Filing URL.
    pub filing_url: Option<String>,
    /// Company name.
    pub company_name: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// Tickers.
    pub tickers: Option<Vec<String>>,
}

/// SEC filing section content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSection {
    /// Section key/name.
    pub section: Option<String>,
    /// Section text content.
    pub content: Option<String>,
}

/// Risk factor entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskFactor {
    /// Risk factor title.
    pub title: Option<String>,
    /// Risk factor text.
    pub text: Option<String>,
    /// Risk category.
    pub category: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
}

/// Risk category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskCategory {
    /// Category name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Filing sections response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSectionsResponse {
    /// Request ID.
    pub request_id: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Sections.
    pub results: Option<Vec<FilingSection>>,
}

/// Fetch SEC EDGAR index (filing metadata).
pub async fn sec_edgar_index(params: &[(&str, &str)]) -> Result<PaginatedResponse<FilingEntry>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/filings", params).await
}

/// Fetch 10-K filing section content.
pub async fn filing_10k_sections(
    accession_number: &str,
    params: &[(&str, &str)],
) -> Result<FilingSectionsResponse> {
    let client = build_client()?;
    let path = format!("/v1/reference/sec/filings/{}/sections", accession_number);
    let json = client.get_raw(&path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "10k_sections".to_string(),
        context: format!("Failed to parse 10-K sections: {e}"),
    })
}

/// Fetch 8-K filing text.
pub async fn filing_8k_text(
    accession_number: &str,
    params: &[(&str, &str)],
) -> Result<FilingSectionsResponse> {
    let client = build_client()?;
    let path = format!("/v1/reference/sec/filings/{}/8k", accession_number);
    let json = client.get_raw(&path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "8k_text".to_string(),
        context: format!("Failed to parse 8-K text: {e}"),
    })
}

/// Fetch risk factors from SEC filings.
pub async fn risk_factors(params: &[(&str, &str)]) -> Result<PaginatedResponse<RiskFactor>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/risk-factors", params).await
}

/// Fetch risk factor categories.
pub async fn risk_categories() -> Result<PaginatedResponse<RiskCategory>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/risk-categories", &[]).await
}
