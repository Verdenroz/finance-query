//! Earnings Trend Module
//!
//! Contains earnings trend and analyst estimate data.

use serde::{Deserialize, Serialize};

/// Earnings trend and estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTrend {
    /// Default methodology (e.g., "gaap")
    #[serde(default)]
    pub default_methodology: Option<String>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// List of trend periods
    #[serde(default)]
    pub trend: Vec<EarningsTrendPeriod>,
}

/// Earnings trend for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTrendPeriod {
    /// End date for this period
    #[serde(default)]
    pub end_date: Option<String>,

    /// Earnings estimates
    #[serde(default)]
    pub earnings_estimate: Option<EarningsEstimate>,

    /// Revenue estimates
    #[serde(default)]
    pub revenue_estimate: Option<RevenueEstimate>,

    /// EPS trend over time
    #[serde(default)]
    pub eps_trend: Option<EpsTrend>,

    /// EPS revisions
    #[serde(default)]
    pub eps_revisions: Option<EpsRevisions>,
}

/// Earnings estimate data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsEstimate {
    /// Average estimate
    #[serde(default)]
    pub avg: Option<crate::models::quote::FormattedValue<f64>>,

    /// Low estimate
    #[serde(default)]
    pub low: Option<crate::models::quote::FormattedValue<f64>>,

    /// High estimate
    #[serde(default)]
    pub high: Option<crate::models::quote::FormattedValue<f64>>,

    /// Year-ago EPS
    #[serde(default)]
    pub year_ago_eps: Option<crate::models::quote::FormattedValue<f64>>,

    /// Number of analysts providing estimates
    #[serde(default)]
    pub number_of_analysts: Option<crate::models::quote::FormattedValue<i64>>,

    /// Expected growth rate
    #[serde(default)]
    pub growth: Option<crate::models::quote::FormattedValue<f64>>,

    /// Currency for earnings
    #[serde(default)]
    pub earnings_currency: Option<String>,
}

/// Revenue estimate data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevenueEstimate {
    /// Average estimate
    #[serde(default)]
    pub avg: Option<crate::models::quote::FormattedValue<i64>>,

    /// Low estimate
    #[serde(default)]
    pub low: Option<crate::models::quote::FormattedValue<i64>>,

    /// High estimate
    #[serde(default)]
    pub high: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of analysts providing estimates
    #[serde(default)]
    pub number_of_analysts: Option<crate::models::quote::FormattedValue<i64>>,

    /// Year-ago revenue
    #[serde(default)]
    pub year_ago_revenue: Option<crate::models::quote::FormattedValue<i64>>,

    /// Expected growth rate
    #[serde(default)]
    pub growth: Option<crate::models::quote::FormattedValue<f64>>,
}

/// EPS trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpsTrend {
    /// Current EPS
    #[serde(default)]
    pub current: Option<crate::models::quote::FormattedValue<f64>>,

    /// EPS 7 days ago
    #[serde(default)]
    #[serde(rename = "7daysAgo")]
    pub seven_days_ago: Option<crate::models::quote::FormattedValue<f64>>,

    /// EPS 30 days ago
    #[serde(default)]
    #[serde(rename = "30daysAgo")]
    pub thirty_days_ago: Option<crate::models::quote::FormattedValue<f64>>,

    /// EPS 60 days ago
    #[serde(default)]
    #[serde(rename = "60daysAgo")]
    pub sixty_days_ago: Option<crate::models::quote::FormattedValue<f64>>,

    /// EPS 90 days ago
    #[serde(default)]
    #[serde(rename = "90daysAgo")]
    pub ninety_days_ago: Option<crate::models::quote::FormattedValue<f64>>,
}

/// EPS revision data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpsRevisions {
    /// Number of upward revisions in last 7 days
    #[serde(default)]
    pub up_last7days: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of upward revisions in last 30 days
    #[serde(default)]
    pub up_last30days: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of downward revisions in last 7 days
    #[serde(default)]
    #[serde(rename = "downLast7Days")]
    pub down_last7_days: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of downward revisions in last 30 days
    #[serde(default)]
    pub down_last30days: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of downward revisions in last 90 days
    #[serde(default)]
    pub down_last90days: Option<crate::models::quote::FormattedValue<i64>>,

    /// Currency for EPS revisions
    #[serde(default)]
    pub eps_revisions_currency: Option<String>,
}
