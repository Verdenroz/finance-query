//! Corporate governance models: executive compensation and employee headcount.
//!
//! Served through the [`Capability::CORPORATE`](crate::Capability::CORPORATE)
//! route. Both are derived from the company's own SEC filings (DEF 14A proxy
//! statements and 10-K/10-Q cover pages respectively), so figures are annual
//! and lag the filing date. EDGAR serves employee counts filed under the
//! voluntary `dei:EntityNumberOfEmployees` tag, which most filers don't use;
//! FMP remains the more complete source.

use serde::{Deserialize, Serialize};

/// One executive's reported compensation for one fiscal year.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExecutiveCompensation {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// SEC Central Index Key of the filer.
    pub cik: Option<String>,
    /// Company name as filed.
    pub company_name: Option<String>,
    /// Executive name and position, as a single filed string.
    pub name_and_position: Option<String>,
    /// Fiscal year the compensation covers.
    pub year: Option<i32>,
    /// Base salary.
    pub salary: Option<f64>,
    /// Cash bonus.
    pub bonus: Option<f64>,
    /// Value of stock awards.
    pub stock_award: Option<f64>,
    /// Value of option awards.
    pub option_award: Option<f64>,
    /// Non-equity incentive plan compensation.
    pub incentive_plan_compensation: Option<f64>,
    /// All other compensation.
    pub other_compensation: Option<f64>,
    /// Total compensation.
    pub total: Option<f64>,
    /// Filing date of the source document (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// URL of the source filing.
    pub url: Option<String>,
}

/// Employee headcount as reported on one filing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EmployeeCount {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// SEC Central Index Key of the filer.
    pub cik: Option<String>,
    /// Company name as filed.
    pub company_name: Option<String>,
    /// Number of employees reported.
    pub employee_count: Option<i64>,
    /// Period the count is as of (`YYYY-MM-DD`).
    pub period_of_report: Option<String>,
    /// Form the count was reported on (e.g. `"10-K"`).
    pub form_type: Option<String>,
    /// Filing date (`YYYY-MM-DD`).
    pub filing_date: Option<String>,
    /// URL of the source filing.
    pub source: Option<String>,
}
