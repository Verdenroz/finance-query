//! SEC filing endpoints: 10-K sections, 8-K text, EDGAR index, risk factors.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::filings::{ProviderFiling, ProviderFilings};

use super::build_client;
use super::models::PaginatedResponseDTO;

/// SEC filing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingEntryDTO {
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
pub struct FilingSectionDTO {
    /// Section key/name.
    pub section: Option<String>,
    /// Section text content.
    pub content: Option<String>,
}

/// Risk factor entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskFactorDTO {
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
pub struct RiskCategoryDTO {
    /// Category name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Filing sections response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSectionsResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Sections.
    pub results: Option<Vec<FilingSectionDTO>>,
}

/// Fetch SEC EDGAR index (filing metadata).
pub async fn sec_edgar_index(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<FilingEntryDTO>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/filings", params).await
}

/// Fetch filings (canonical) for a stock ticker.
pub async fn fetch_filings_response(symbol: &str) -> Result<ProviderFilings> {
    let paginated = sec_edgar_index(&[("ticker", symbol)]).await?;
    let filings = paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|f| ProviderFiling {
            accession_number: f.accession_number,
            filing_date: f.filing_date,
            filing_type: f.filing_type,
            filing_url: f.filing_url,
            company_name: f.company_name,
            cik: f.cik,
        })
        .collect();
    Ok(ProviderFilings {
        symbol: symbol.to_string(),
        filings,
    })
}

/// Fetch 10-K filing section content.
pub async fn filing_10k_sections(
    accession_number: &str,
    params: &[(&str, &str)],
) -> Result<FilingSectionsResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/reference/sec/filings/{}/sections",
        encode_path_segment(accession_number)
    );
    client
        .get_as(&path, params, "10k_sections", "10-K sections")
        .await
}

/// Fetch 8-K filing text.
pub async fn filing_8k_text(
    accession_number: &str,
    params: &[(&str, &str)],
) -> Result<FilingSectionsResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/reference/sec/filings/{}/8k",
        encode_path_segment(accession_number)
    );
    client.get_as(&path, params, "8k_text", "8-K text").await
}

/// Fetch risk factors from SEC filings.
pub async fn risk_factors(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<RiskFactorDTO>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/risk-factors", params).await
}

/// Fetch risk factor categories.
pub async fn risk_categories() -> Result<PaginatedResponseDTO<RiskCategoryDTO>> {
    let client = build_client()?;
    client.get("/v1/reference/sec/risk-categories", &[]).await
}
