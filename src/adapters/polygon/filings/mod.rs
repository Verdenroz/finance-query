//! SEC filing endpoints: 10-K sections, 8-K text, EDGAR index, risk factors.

use serde::{Deserialize, Serialize};

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
    pub form_type: Option<String>,
    /// Filing URL.
    pub filing_url: Option<String>,
    /// Company name.
    pub issuer_name: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// Primary ticker.
    pub ticker: Option<String>,
}

/// SEC filing section content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingSectionDTO {
    /// Section key/name.
    pub section: Option<String>,
    /// Section text content.
    #[serde(alias = "text", alias = "items_text")]
    pub content: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
    /// Filing URL.
    pub filing_url: Option<String>,
    /// Ticker symbol.
    pub ticker: Option<String>,
}

/// Risk factor entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskFactorDTO {
    /// Top-level risk category.
    pub primary_category: Option<String>,
    /// Mid-level risk category.
    pub secondary_category: Option<String>,
    /// Most specific risk category.
    pub tertiary_category: Option<String>,
    /// Supporting disclosure text.
    pub supporting_text: Option<String>,
    /// Filing date.
    pub filing_date: Option<String>,
}

/// Risk category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)] // unrouted: risk-category taxonomy; fold into risk factors if a consumer appears
pub struct RiskCategoryDTO {
    /// Top-level category.
    pub primary_category: Option<String>,
    /// Mid-level category.
    pub secondary_category: Option<String>,
    /// Most specific category.
    pub tertiary_category: Option<String>,
    /// Taxonomy name.
    pub taxonomy: Option<serde_json::Value>,
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
    client.get("/stocks/filings/vX/index", params).await
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
            filing_type: f.form_type,
            filing_url: f.filing_url,
            company_name: f.issuer_name,
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
    let mut query = vec![("accession_number", accession_number)];
    query.extend_from_slice(params);
    client
        .get_as(
            "/stocks/filings/10-K/vX/sections",
            &query,
            "10k_sections",
            "10-K sections",
        )
        .await
}

/// Fetch 8-K filing text.
pub async fn filing_8k_text(
    accession_number: &str,
    params: &[(&str, &str)],
) -> Result<FilingSectionsResponseDTO> {
    let client = build_client()?;
    let mut query = vec![("accession_number", accession_number)];
    query.extend_from_slice(params);
    client
        .get_as("/stocks/filings/8-K/vX/text", &query, "8k_text", "8-K text")
        .await
}

/// Fetch risk factors from SEC filings.
pub async fn risk_factors(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<RiskFactorDTO>> {
    let client = build_client()?;
    client.get("/stocks/filings/vX/risk-factors", params).await
}

/// Fetch risk factor categories.
#[allow(dead_code)] // unrouted: risk-category taxonomy; fold into risk factors if a consumer appears
pub async fn risk_categories() -> Result<PaginatedResponseDTO<RiskCategoryDTO>> {
    let client = build_client()?;
    client.get("/stocks/taxonomies/vX/risk-factors", &[]).await
}

/// Fetch canonical sectioned text for one filing.
pub async fn fetch_filing_sections_response(
    accession_number: &str,
    form: crate::models::filings::FilingSectionForm,
) -> Result<Vec<crate::models::filings::FilingSection>> {
    use crate::models::filings::FilingSectionForm;
    let resp = match form {
        FilingSectionForm::TenK => filing_10k_sections(accession_number, &[]).await?,
        FilingSectionForm::EightK => filing_8k_text(accession_number, &[]).await?,
    };
    Ok(resp
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|d| crate::models::filings::FilingSection {
            section: d.section,
            content: d.content,
        })
        .collect())
}

/// Fetch canonical risk factors for a stock ticker.
pub async fn fetch_risk_factors_response(
    symbol: &str,
) -> Result<Vec<crate::models::filings::RiskFactor>> {
    let paginated = risk_factors(&[("ticker", symbol), ("limit", "100")]).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|d| crate::models::filings::RiskFactor {
            title: d.tertiary_category.clone(),
            text: d.supporting_text,
            category: d
                .tertiary_category
                .or(d.secondary_category)
                .or(d.primary_category),
            filing_date: d.filing_date,
        })
        .collect())
}
