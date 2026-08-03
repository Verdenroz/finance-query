//! Short interest, short volume, and share float models.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; Polygon is currently the only provider.

use serde::{Deserialize, Serialize};

/// A short-interest data point (bi-monthly settlement report).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortInterest {
    /// Settlement date (`YYYY-MM-DD`).
    pub settlement_date: Option<String>,
    /// Total shares held short at settlement.
    pub short_interest: Option<f64>,
    /// Average daily trading volume over the reporting period.
    pub avg_daily_volume: Option<f64>,
    /// Days to cover (short interest / average daily volume).
    pub days_to_cover: Option<f64>,
}

/// A daily short-volume data point.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortVolume {
    /// Trade date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Shares sold short.
    pub short_volume: Option<f64>,
    /// Shares sold short exempt from the uptick rule.
    pub short_exempt_volume: Option<f64>,
    /// Total volume.
    pub total_volume: Option<f64>,
}

/// Share float and shares outstanding.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShareFloat {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Freely tradable shares.
    pub float_shares: Option<f64>,
    /// Total shares outstanding.
    pub outstanding_shares: Option<f64>,
    /// As-of date (`YYYY-MM-DD`).
    pub date: Option<String>,
}
