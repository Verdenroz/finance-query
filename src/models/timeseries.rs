//! Timeseries model for financial fundamentals data

use serde::{Deserialize, Serialize};

/// Response wrapper for fundamentals timeseries endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesResponse {
    /// Timeseries container
    pub timeseries: TimeseriesContainer,
}

/// Container for timeseries results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesContainer {
    /// Timeseries results
    pub result: Vec<TimeseriesResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// A single timeseries result (e.g., annualTotalRevenue)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeseriesResult {
    /// Metadata about this timeseries
    pub meta: TimeseriesMeta,
    /// The actual timeseries data points
    #[serde(flatten)]
    pub data: std::collections::HashMap<String, Vec<TimeseriesDataPoint>>,
}

/// Metadata for a timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesMeta {
    /// Stock symbol
    pub symbol: Vec<String>,
    /// Data type (e.g., "annualTotalRevenue")
    #[serde(rename = "type")]
    pub data_type: Vec<String>,
}

/// A single data point in a timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeseriesDataPoint {
    /// As of date (YYYY-MM-DD)
    pub as_of_date: Option<String>,
    /// Period type (e.g., "12M", "3M")
    pub period_type: Option<String>,
    /// Currency code
    pub currency_code: Option<String>,
    /// The reported value
    pub reported_value: Option<ReportedValue>,
}

/// A reported value with raw and formatted versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportedValue {
    /// Raw numeric value
    pub raw: Option<f64>,
    /// Formatted string value
    pub fmt: Option<String>,
}

impl TimeseriesResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

/// Common fundamental data types for timeseries queries
pub mod fundamental_types {
    /// Annual total revenue
    pub const ANNUAL_TOTAL_REVENUE: &str = "annualTotalRevenue";
    /// Quarterly total revenue
    pub const QUARTERLY_TOTAL_REVENUE: &str = "quarterlyTotalRevenue";
    /// Annual net income
    pub const ANNUAL_NET_INCOME: &str = "annualNetIncome";
    /// Quarterly net income
    pub const QUARTERLY_NET_INCOME: &str = "quarterlyNetIncome";
    /// Annual gross profit
    pub const ANNUAL_GROSS_PROFIT: &str = "annualGrossProfit";
    /// Quarterly gross profit
    pub const QUARTERLY_GROSS_PROFIT: &str = "quarterlyGrossProfit";
    /// Annual operating income
    pub const ANNUAL_OPERATING_INCOME: &str = "annualOperatingIncome";
    /// Quarterly operating income
    pub const QUARTERLY_OPERATING_INCOME: &str = "quarterlyOperatingIncome";
    /// Annual EBITDA
    pub const ANNUAL_EBITDA: &str = "annualEbitda";
    /// Quarterly EBITDA
    pub const QUARTERLY_EBITDA: &str = "quarterlyEbitda";
    /// Annual total assets
    pub const ANNUAL_TOTAL_ASSETS: &str = "annualTotalAssets";
    /// Quarterly total assets
    pub const QUARTERLY_TOTAL_ASSETS: &str = "quarterlyTotalAssets";
    /// Annual total liabilities
    pub const ANNUAL_TOTAL_LIABILITIES: &str = "annualTotalLiabilitiesNetMinorityInterest";
    /// Quarterly total liabilities
    pub const QUARTERLY_TOTAL_LIABILITIES: &str = "quarterlyTotalLiabilitiesNetMinorityInterest";
    /// Annual total equity
    pub const ANNUAL_TOTAL_EQUITY: &str = "annualStockholdersEquity";
    /// Quarterly total equity
    pub const QUARTERLY_TOTAL_EQUITY: &str = "quarterlyStockholdersEquity";
    /// Annual operating cash flow
    pub const ANNUAL_OPERATING_CASH_FLOW: &str = "annualOperatingCashFlow";
    /// Quarterly operating cash flow
    pub const QUARTERLY_OPERATING_CASH_FLOW: &str = "quarterlyOperatingCashFlow";
    /// Annual free cash flow
    pub const ANNUAL_FREE_CASH_FLOW: &str = "annualFreeCashFlow";
    /// Quarterly free cash flow
    pub const QUARTERLY_FREE_CASH_FLOW: &str = "quarterlyFreeCashFlow";
    /// Annual basic EPS
    pub const ANNUAL_BASIC_EPS: &str = "annualBasicEPS";
    /// Quarterly basic EPS
    pub const QUARTERLY_BASIC_EPS: &str = "quarterlyBasicEPS";
    /// Annual diluted EPS
    pub const ANNUAL_DILUTED_EPS: &str = "annualDilutedEPS";
    /// Quarterly diluted EPS
    pub const QUARTERLY_DILUTED_EPS: &str = "quarterlyDilutedEPS";
}
