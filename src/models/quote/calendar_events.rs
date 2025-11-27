//! Calendar Events Module
//!
//! Contains upcoming corporate events like earnings dates and dividends.

use serde::{Deserialize, Serialize};

/// Calendar events for a stock
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvents {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Earnings information
    #[serde(default)]
    pub earnings: Option<EarningsCalendar>,

    /// Ex-dividend date
    #[serde(default)]
    pub ex_dividend_date: Option<crate::models::quote::FormattedValue<i64>>,

    /// Dividend payment date
    #[serde(default)]
    pub dividend_date: Option<crate::models::quote::FormattedValue<i64>>,
}

/// Earnings calendar information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsCalendar {
    /// Upcoming earnings dates (can be a range)
    #[serde(default)]
    pub earnings_date: Vec<crate::models::quote::FormattedValue<i64>>,

    /// Date of last earnings call
    #[serde(default)]
    pub earnings_call_date: Option<Vec<crate::models::quote::FormattedValue<i64>>>,

    /// Whether the earnings date is an estimate
    #[serde(default)]
    pub is_earnings_date_estimate: Option<bool>,

    /// Average earnings estimate
    #[serde(default)]
    pub earnings_average: Option<crate::models::quote::FormattedValue<f64>>,

    /// High earnings estimate
    #[serde(default)]
    pub earnings_high: Option<crate::models::quote::FormattedValue<f64>>,

    /// Low earnings estimate
    #[serde(default)]
    pub earnings_low: Option<crate::models::quote::FormattedValue<f64>>,

    /// Average revenue estimate
    #[serde(default)]
    pub revenue_average: Option<crate::models::quote::FormattedValue<i64>>,

    /// High revenue estimate
    #[serde(default)]
    pub revenue_high: Option<crate::models::quote::FormattedValue<i64>>,

    /// Low revenue estimate
    #[serde(default)]
    pub revenue_low: Option<crate::models::quote::FormattedValue<i64>>,
}
