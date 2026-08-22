//! Earnings-surprise history.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP is currently the only provider.

use serde::{Deserialize, Serialize};

/// One reported earnings result versus the analyst estimate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsSurprise {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Earnings report date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Actual reported EPS.
    pub actual_eps: Option<f64>,
    /// Analyst-estimated EPS.
    pub estimated_eps: Option<f64>,
    /// `actual_eps - estimated_eps`.
    pub surprise: Option<f64>,
    /// Surprise as a percentage of the estimate.
    pub surprise_percent: Option<f64>,
}
