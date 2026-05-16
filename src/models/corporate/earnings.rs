//! Earnings Module
//!
//! Contains quarterly and annual earnings chart data.

use serde::{Deserialize, Serialize};

/// Earnings data including charts and forecasts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Earnings {
    /// Default methodology (e.g., "gaap")
    #[serde(default)]
    pub default_methodology: Option<String>,

    /// Earnings chart data
    #[serde(default)]
    pub earnings_chart: Option<EarningsChart>,

    /// Financial chart data (revenue/earnings over time)
    #[serde(default)]
    pub financials_chart: Option<FinancialsChart>,
}

/// Earnings chart showing quarterly data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsChart {
    /// Quarterly earnings data
    #[serde(default)]
    pub quarterly: Vec<QuarterlyEarnings>,

    /// Current quarter estimate
    #[serde(default)]
    pub current_quarter_estimate: Option<crate::models::quote::FormattedValue<f64>>,

    /// Current quarter estimate date
    #[serde(default)]
    pub current_quarter_estimate_date: Option<String>,

    /// Current quarter estimate year
    #[serde(default)]
    pub current_quarter_estimate_year: Option<i32>,

    /// Current fiscal quarter
    #[serde(default)]
    pub current_fiscal_quarter: Option<String>,

    /// Current calendar quarter
    #[serde(default)]
    pub current_calendar_quarter: Option<String>,

    /// Upcoming earnings dates
    #[serde(default)]
    pub earnings_date: Vec<crate::models::quote::FormattedValue<i64>>,

    /// Whether the earnings date is an estimate
    #[serde(default)]
    pub is_earnings_date_estimate: Option<bool>,
}

/// Quarterly earnings entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarterlyEarnings {
    /// Date/quarter identifier
    #[serde(default)]
    pub date: Option<String>,

    /// Actual EPS
    #[serde(default)]
    pub actual: Option<crate::models::quote::FormattedValue<f64>>,

    /// Estimated EPS
    #[serde(default)]
    pub estimate: Option<crate::models::quote::FormattedValue<f64>>,

    /// Difference between actual and estimate (string format)
    #[serde(default)]
    pub difference: Option<String>,

    /// Surprise percentage (string format)
    #[serde(default)]
    pub surprise_pct: Option<String>,

    /// Calendar quarter
    #[serde(default)]
    pub calendar_quarter: Option<String>,

    /// Fiscal quarter
    #[serde(default)]
    pub fiscal_quarter: Option<String>,
}

/// Financial chart showing revenue and earnings over time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialsChart {
    /// Yearly financial data
    #[serde(default)]
    pub yearly: Vec<YearlyFinancials>,

    /// Quarterly financial data
    #[serde(default)]
    pub quarterly: Vec<QuarterlyFinancials>,
}

/// Yearly financial data entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearlyFinancials {
    /// Date
    #[serde(default)]
    pub date: Option<i64>,

    /// Revenue
    #[serde(default)]
    pub revenue: Option<crate::models::quote::FormattedValue<i64>>,

    /// Earnings
    #[serde(default)]
    pub earnings: Option<crate::models::quote::FormattedValue<i64>>,
}

/// Quarterly financial data entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuarterlyFinancials {
    /// Date
    #[serde(default)]
    pub date: Option<String>,

    /// Revenue
    #[serde(default)]
    pub revenue: Option<crate::models::quote::FormattedValue<i64>>,

    /// Earnings
    #[serde(default)]
    pub earnings: Option<crate::models::quote::FormattedValue<i64>>,
}
