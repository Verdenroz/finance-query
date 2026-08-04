//! Aggregated analyst-consensus models.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; FMP is currently the only provider. These are provider-side rollups
//! over the whole analyst panel — distinct from the raw per-analyst grade
//! actions in [`UpgradeDowngradeHistory`](crate::UpgradeDowngradeHistory).

use serde::{Deserialize, Serialize};

/// Consensus analyst price target for a symbol.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceTargetConsensus {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Highest price target across the analyst panel.
    pub target_high: Option<f64>,
    /// Lowest price target across the analyst panel.
    pub target_low: Option<f64>,
    /// Mean price target.
    pub target_consensus: Option<f64>,
    /// Median price target.
    pub target_median: Option<f64>,
}

/// Price-target activity over trailing windows: how many targets were published
/// and their average, per window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PriceTargetSummary {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Number of price targets published in the last month.
    pub last_month_count: Option<i64>,
    /// Average price target published in the last month.
    pub last_month_avg: Option<f64>,
    /// Number of price targets published in the last quarter.
    pub last_quarter_count: Option<i64>,
    /// Average price target published in the last quarter.
    pub last_quarter_avg: Option<f64>,
    /// Number of price targets published in the last year.
    pub last_year_count: Option<i64>,
    /// Average price target published in the last year.
    pub last_year_avg: Option<f64>,
    /// Number of price targets published all time.
    pub all_time_count: Option<i64>,
    /// Average price target published all time.
    pub all_time_avg: Option<f64>,
}

/// Consensus rating rollup — the analyst panel's grade distribution plus the
/// provider's headline recommendation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RatingConsensus {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Analysts rating the symbol a strong buy.
    pub strong_buy: Option<i64>,
    /// Analysts rating the symbol a buy.
    pub buy: Option<i64>,
    /// Analysts rating the symbol a hold.
    pub hold: Option<i64>,
    /// Analysts rating the symbol a sell.
    pub sell: Option<i64>,
    /// Analysts rating the symbol a strong sell.
    pub strong_sell: Option<i64>,
    /// Headline consensus label (e.g. `"Buy"`).
    pub consensus: Option<String>,
}
