//! FMP governance endpoints: executive compensation and employee headcount.
//!
//! Both are extracted from the company's own SEC filings (DEF 14A proxies and
//! 10-K cover pages), so figures are annual and lag the filing date.

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;
use crate::models::corporate::governance::{EmployeeCount, ExecutiveCompensation};

// ============================================================================
// Response types
// ============================================================================

/// Executive compensation entry (`/stable/governance-executive-compensation`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExecutiveCompensationDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// SEC Central Index Key.
    pub cik: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Executive name and position.
    #[serde(rename = "nameAndPosition")]
    pub name_and_position: Option<String>,
    /// Fiscal year.
    pub year: Option<i32>,
    /// Base salary.
    pub salary: Option<f64>,
    /// Cash bonus.
    pub bonus: Option<f64>,
    /// Stock awards.
    #[serde(rename = "stock_award")]
    pub stock_award: Option<f64>,
    /// Option awards.
    #[serde(rename = "option_award")]
    pub option_award: Option<f64>,
    /// Non-equity incentive plan compensation.
    #[serde(rename = "incentive_plan_compensation")]
    pub incentive_plan_compensation: Option<f64>,
    /// All other compensation.
    #[serde(rename = "all_other_compensation")]
    pub all_other_compensation: Option<f64>,
    /// Total compensation.
    pub total: Option<f64>,
    /// Filing date.
    #[serde(rename = "filingDate")]
    pub filing_date: Option<String>,
    /// Source filing URL.
    pub url: Option<String>,
}

/// Employee headcount entry (`/stable/historical-employee-count`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EmployeeCountDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// SEC Central Index Key.
    pub cik: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Employee count.
    #[serde(rename = "employeeCount")]
    pub employee_count: Option<i64>,
    /// Period the count is as of.
    #[serde(rename = "periodOfReport")]
    pub period_of_report: Option<String>,
    /// Form type the count was reported on.
    #[serde(rename = "formType")]
    pub form_type: Option<String>,
    /// Filing date.
    #[serde(rename = "filingDate")]
    pub filing_date: Option<String>,
    /// Source filing URL.
    pub source: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch executive compensation history for a symbol.
pub async fn executive_compensation(symbol: &str) -> Result<Vec<ExecutiveCompensationDTO>> {
    build_client()?
        .get(
            "/stable/governance-executive-compensation",
            &[("symbol", symbol)],
        )
        .await
}

/// Fetch historical employee headcount for a symbol.
pub async fn employee_count(symbol: &str) -> Result<Vec<EmployeeCountDTO>> {
    build_client()?
        .get("/stable/historical-employee-count", &[("symbol", symbol)])
        .await
}

// ============================================================================
// Canonical conversions
// ============================================================================

pub(crate) fn to_executive_compensation(
    dto: ExecutiveCompensationDTO,
    symbol: &str,
) -> ExecutiveCompensation {
    ExecutiveCompensation {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        cik: dto.cik,
        company_name: dto.company_name,
        name_and_position: dto.name_and_position,
        year: dto.year,
        salary: dto.salary,
        bonus: dto.bonus,
        stock_award: dto.stock_award,
        option_award: dto.option_award,
        incentive_plan_compensation: dto.incentive_plan_compensation,
        other_compensation: dto.all_other_compensation,
        total: dto.total,
        filing_date: dto.filing_date,
        url: dto.url,
    }
}

pub(crate) fn to_employee_count(dto: EmployeeCountDTO, symbol: &str) -> EmployeeCount {
    EmployeeCount {
        symbol: dto.symbol.or_else(|| Some(symbol.to_string())),
        cik: dto.cik,
        company_name: dto.company_name,
        employee_count: dto.employee_count,
        period_of_report: dto.period_of_report,
        form_type: dto.form_type,
        filing_date: dto.filing_date,
        source: dto.source,
    }
}

/// Fetch canonical executive compensation, most recent fiscal year first.
pub async fn fetch_executive_compensation_response(
    symbol: &str,
) -> Result<Vec<ExecutiveCompensation>> {
    let dtos = executive_compensation(symbol).await?;
    let mut out: Vec<_> = dtos
        .into_iter()
        .map(|d| to_executive_compensation(d, symbol))
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(r.year));
    Ok(out)
}

/// Fetch canonical employee headcount history, most recent period first.
pub async fn fetch_employee_count_response(symbol: &str) -> Result<Vec<EmployeeCount>> {
    let dtos = employee_count(symbol).await?;
    let mut out: Vec<_> = dtos
        .into_iter()
        .map(|d| to_employee_count(d, symbol))
        .collect();
    out.sort_by(|a, b| b.period_of_report.cmp(&a.period_of_report));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comp_dto(year: i32) -> ExecutiveCompensationDTO {
        serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "cik": "0000320193",
            "companyName": "Apple Inc.",
            "nameAndPosition": "Timothy D. Cook Chief Executive Officer",
            "year": year,
            "salary": 3_000_000.0,
            "bonus": 0.0,
            "stock_award": 46_970_283.0,
            "option_award": 0.0,
            "incentive_plan_compensation": 10_713_450.0,
            "all_other_compensation": 2_521_124.0,
            "total": 63_209_845.0,
            "filingDate": "2024-01-11",
            "url": "https://www.sec.gov/Archives/..."
        }))
        .unwrap()
    }

    #[test]
    fn executive_compensation_maps_snake_case_award_keys() {
        let out = to_executive_compensation(comp_dto(2023), "AAPL");
        assert_eq!(
            out.name_and_position.as_deref(),
            Some("Timothy D. Cook Chief Executive Officer")
        );
        assert_eq!(out.year, Some(2023));
        assert_eq!(out.salary, Some(3_000_000.0));
        assert_eq!(out.stock_award, Some(46_970_283.0));
        assert_eq!(out.incentive_plan_compensation, Some(10_713_450.0));
        // `all_other_compensation` is exposed under the shorter public name.
        assert_eq!(out.other_compensation, Some(2_521_124.0));
        assert_eq!(out.total, Some(63_209_845.0));
    }

    #[test]
    fn employee_count_maps_filing_metadata() {
        let dto: EmployeeCountDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "cik": "0000320193",
            "companyName": "Apple Inc.",
            "employeeCount": 161_000,
            "periodOfReport": "2023-09-30",
            "formType": "10-K",
            "filingDate": "2023-11-03",
            "source": "https://www.sec.gov/Archives/..."
        }))
        .unwrap();

        let out = to_employee_count(dto, "AAPL");
        assert_eq!(out.employee_count, Some(161_000));
        assert_eq!(out.period_of_report.as_deref(), Some("2023-09-30"));
        assert_eq!(out.form_type.as_deref(), Some("10-K"));
    }

    /// FMP returns compensation oldest-first; the canonical order is newest-first.
    #[test]
    fn executive_compensation_is_sorted_newest_year_first() {
        let mut rows: Vec<_> = [2019, 2023, 2021]
            .into_iter()
            .map(|y| to_executive_compensation(comp_dto(y), "AAPL"))
            .collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.year));
        assert_eq!(
            rows.iter().map(|r| r.year).collect::<Vec<_>>(),
            vec![Some(2023), Some(2021), Some(2019)]
        );
    }

    #[test]
    fn missing_symbol_falls_back_to_the_requested_one() {
        let dto: EmployeeCountDTO = serde_json::from_value(serde_json::json!({})).unwrap();
        assert_eq!(
            to_employee_count(dto, "MSFT").symbol.as_deref(),
            Some("MSFT")
        );
    }
}
