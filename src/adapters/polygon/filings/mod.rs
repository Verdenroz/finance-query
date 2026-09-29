//! SEC filing endpoints: filing index, 10-K sections, 8-K text, risk factors.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};
use crate::models::filings::{
    FilingSection, FilingSectionForm, ProviderFiling, ProviderFilings, RiskFactor,
};

use super::build_client;
use super::client::PathRule;
use super::models::PaginatedResponseDTO;

const INDEX_PATH: &str = "/stocks/filings/vX/index";
const TEN_K_SECTIONS_PATH: &str = "/stocks/filings/10-K/vX/sections";
const EIGHT_K_TEXT_PATH: &str = "/stocks/filings/8-K/vX/text";
const RISK_FACTORS_PATH: &str = "/stocks/filings/vX/risk-factors";
/// Bounds the scan of one filer's year of filings for an accession number.
const MAX_SECTION_PAGES: usize = 10;

/// One filing from the EDGAR filing index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FilingEntryDTO {
    /// Accession number.
    pub accession_number: Option<String>,
    /// Filer CIK.
    pub cik: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// EDGAR URL of the filing.
    pub filing_url: Option<String>,
    /// Form type (e.g., `"10-K"`, `"8-K"`).
    pub form_type: Option<String>,
    /// Issuer name.
    pub issuer_name: Option<String>,
    /// Issuer ticker.
    pub ticker: Option<String>,
}

/// One parsed 10-K section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TenKSectionDTO {
    /// Issuer CIK.
    pub cik: Option<String>,
    /// Issuer ticker.
    pub ticker: Option<String>,
    /// Section identifier (e.g., `"risk_factors"`, `"business"`).
    pub section: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// Fiscal period end (`YYYY-MM-DD`).
    pub period_end: Option<String>,
    /// EDGAR URL of the filing, which names its accession number.
    pub filing_url: Option<String>,
    /// Plain-text section content.
    pub text: Option<String>,
}

/// The parsed Items text of one 8-K or 8-K/A.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EightKTextDTO {
    /// Accession number.
    pub accession_number: Option<String>,
    /// Issuer CIK.
    pub cik: Option<String>,
    /// Issuer ticker.
    pub ticker: Option<String>,
    /// Form type (`"8-K"` or `"8-K/A"`).
    pub form_type: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// Items section text.
    pub items_text: Option<String>,
}

/// One risk factor, classified by Massive's three-tier taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskFactorDTO {
    /// Issuer CIK.
    pub cik: Option<String>,
    /// Issuer ticker.
    pub ticker: Option<String>,
    /// Broadest taxonomy category.
    pub primary_category: Option<String>,
    /// Middle taxonomy category.
    pub secondary_category: Option<String>,
    /// Most specific taxonomy category.
    pub tertiary_category: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// Quote from the filing supporting the classification.
    pub supporting_text: Option<String>,
}

/// Search the EDGAR filing index.
pub async fn filing_index(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<FilingEntryDTO>> {
    build_client()?.get(INDEX_PATH, params).await
}

/// Fetch taxonomy-classified risk factors.
pub async fn risk_factors(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<RiskFactorDTO>> {
    build_client()?.get(RISK_FACTORS_PATH, params).await
}

fn filing_to_canonical(entry: FilingEntryDTO) -> ProviderFiling {
    ProviderFiling {
        accession_number: entry.accession_number,
        filing_date: entry.filing_date,
        filing_type: entry.form_type,
        filing_url: entry.filing_url,
        company_name: entry.issuer_name,
        cik: entry.cik,
    }
}

fn risk_factor_to_canonical(row: RiskFactorDTO) -> RiskFactor {
    RiskFactor {
        title: row.tertiary_category,
        text: row.supporting_text,
        category: row.primary_category,
        filing_date: row.filing_date,
    }
}

/// Fetch a symbol's most recent filings (canonical).
pub async fn fetch_filings_response(symbol: &str) -> Result<ProviderFilings> {
    let page = filing_index(&[
        ("ticker", symbol),
        ("sort", "filing_date.desc"),
        ("limit", "100"),
    ])
    .await?;
    Ok(ProviderFilings {
        symbol: symbol.to_string(),
        filings: page
            .results
            .unwrap_or_default()
            .into_iter()
            .map(filing_to_canonical)
            .collect(),
    })
}

/// Fetch canonical risk factors for a stock ticker.
pub async fn fetch_risk_factors_response(symbol: &str) -> Result<Vec<RiskFactor>> {
    let page = risk_factors(&[("ticker", symbol), ("limit", "100")]).await?;
    Ok(page
        .results
        .unwrap_or_default()
        .into_iter()
        .map(risk_factor_to_canonical)
        .collect())
}

/// The filer CIK and filing-year window encoded in an accession number
/// (`##########-YY-######`).
fn accession_window(accession_number: &str) -> Option<(&str, String, String)> {
    let mut parts = accession_number.split('-');
    let (cik, year, sequence) = (parts.next()?, parts.next()?, parts.next()?);
    let digits = |s: &str, len: usize| s.len() == len && s.bytes().all(|b| b.is_ascii_digit());
    if parts.next().is_some() || !digits(cik, 10) || !digits(year, 2) || !digits(sequence, 6) {
        return None;
    }
    let yy: i32 = year.parse().ok()?;
    let year = if yy >= 93 { 1900 + yy } else { 2000 + yy };
    // A filing accepted after hours on the last business day is dated the next year.
    Some((cik, format!("{year}-01-01"), format!("{}-01-15", year + 1)))
}

fn url_names_accession(url: &str, accession_number: &str) -> bool {
    url.contains(accession_number) || url.contains(&accession_number.replace('-', ""))
}

/// Collect the rows matching one accession number across an issuer's pages.
async fn scan_pages<T: DeserializeOwned>(
    path: &str,
    params: &[(&str, &str)],
    matches: &impl Fn(&T) -> bool,
) -> Result<Vec<T>> {
    let client = build_client()?;
    let mut found = Vec::new();
    let mut cursor = None;
    for _ in 0..MAX_SECTION_PAGES {
        let (page, current): (PaginatedResponseDTO<T>, _) = client
            .page(path, params, PathRule::Exact, cursor.as_ref())
            .await?;
        found.extend(page.results.unwrap_or_default().into_iter().filter(matches));
        cursor = client.continuation(page.next_url, path, params, PathRule::Exact, &current)?;
        if cursor.is_none() {
            break;
        }
    }
    Ok(found)
}

/// Find one accession number's rows in the filing year it encodes.
///
/// Massive has no accession filter and silently ignores unknown parameters, so
/// rows are narrowed by issuer and year and matched here. The issuer is the
/// caller's ticker when known, then the filer CIK in the accession number,
/// which names a filing agent rather than the issuer for agent-filed documents.
async fn rows_for_accession<T: DeserializeOwned>(
    path: &str,
    ticker: Option<&str>,
    accession_number: &str,
    matches: impl Fn(&T) -> bool,
) -> Result<Vec<T>> {
    let (filer_cik, from, to) =
        accession_window(accession_number).ok_or_else(|| FinanceError::InvalidParameter {
            param: "accession_number".into(),
            reason: "expected ##########-YY-######".into(),
        })?;
    let issuers = ticker
        .map(|ticker| ("ticker", ticker))
        .into_iter()
        .chain([("cik", filer_cik)]);
    for issuer in issuers {
        let params = [
            issuer,
            ("filing_date.gte", from.as_str()),
            ("filing_date.lte", to.as_str()),
            ("limit", "100"),
        ];
        let found = scan_pages(path, &params, &matches).await?;
        if !found.is_empty() {
            return Ok(found);
        }
    }
    Ok(Vec::new())
}

/// Fetch canonical sectioned text for one filing, searching under `ticker`
/// first when the issuer is known.
pub async fn fetch_filing_sections_response(
    ticker: Option<&str>,
    accession_number: &str,
    form: FilingSectionForm,
) -> Result<Vec<FilingSection>> {
    let sections: Vec<FilingSection> = match form {
        FilingSectionForm::TenK => rows_for_accession(
            TEN_K_SECTIONS_PATH,
            ticker,
            accession_number,
            |row: &TenKSectionDTO| {
                row.filing_url
                    .as_deref()
                    .is_some_and(|url| url_names_accession(url, accession_number))
            },
        )
        .await?
        .into_iter()
        .map(|row| FilingSection {
            section: row.section,
            content: row.text,
        })
        .collect(),
        FilingSectionForm::EightK => rows_for_accession(
            EIGHT_K_TEXT_PATH,
            ticker,
            accession_number,
            |row: &EightKTextDTO| row.accession_number.as_deref() == Some(accession_number),
        )
        .await?
        .into_iter()
        .map(|row| FilingSection {
            section: Some("items".into()),
            content: row.items_text,
        })
        .collect(),
    };
    if sections.is_empty() {
        return Err(FinanceError::SymbolNotFound {
            symbol: None,
            context: format!("Massive has no parsed text for filing {accession_number}"),
        });
    }
    Ok(sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accession_numbers_give_the_filer_and_a_filing_year_window() {
        assert_eq!(
            accession_window("0000320193-25-000079"),
            Some(("0000320193", "2025-01-01".into(), "2026-01-15".into()))
        );
        assert_eq!(
            accession_window("0001047469-98-000123").map(|w| w.1),
            Some("1998-01-01".into())
        );
        for bad in [
            "",
            "320193-25-000079",
            "0000320193-2025-000079",
            "0000320193-25-79x",
            "a-b-c-d",
        ] {
            assert_eq!(accession_window(bad), None, "{bad}");
        }
    }

    #[test]
    fn filing_urls_match_dashed_and_undashed_accessions() {
        let accession = "0000320193-25-000079";
        assert!(url_names_accession(
            "https://www.sec.gov/Archives/edgar/data/320193/0000320193-25-000079.txt",
            accession
        ));
        assert!(url_names_accession(
            "https://www.sec.gov/Archives/edgar/data/320193/000032019325000079/aapl-20250927.htm",
            accession
        ));
        assert!(!url_names_accession(
            "https://www.sec.gov/Archives/edgar/data/320193/0000320193-24-000123.txt",
            accession
        ));
    }

    #[test]
    fn index_and_risk_rows_map_to_canonical_models() {
        let entry: FilingEntryDTO = serde_json::from_value(serde_json::json!({
            "cik": "0000320193",
            "issuer_name": "Apple Inc.",
            "form_type": "10-K",
            "filing_date": "2025-10-31",
            "filing_url": "https://www.sec.gov/Archives/edgar/data/320193/0000320193-25-000079.txt",
            "accession_number": "0000320193-25-000079",
            "ticker": "AAPL"
        }))
        .unwrap();
        let filing = filing_to_canonical(entry);
        assert_eq!(filing.filing_type.as_deref(), Some("10-K"));
        assert_eq!(filing.company_name.as_deref(), Some("Apple Inc."));

        let row: RiskFactorDTO = serde_json::from_value(serde_json::json!({
            "cik": "0000320193",
            "ticker": "AAPL",
            "primary_category": "financial_and_market",
            "secondary_category": "capital_structure_and_performance",
            "tertiary_category": "dividend_policy_and_capital_allocation",
            "filing_date": "2024-11-01",
            "supporting_text": "The Company believes the price of its stock..."
        }))
        .unwrap();
        let risk = risk_factor_to_canonical(row);
        assert_eq!(risk.category.as_deref(), Some("financial_and_market"));
        assert_eq!(
            risk.title.as_deref(),
            Some("dividend_policy_and_capital_allocation")
        );
        assert!(risk.text.unwrap().starts_with("The Company"));
    }
}
