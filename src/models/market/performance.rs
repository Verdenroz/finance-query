//! Market-wide performance models.
//!
//! Returned by the [`Capability::MARKET`](finance_query_types::Capability::MARKET) route via
//! `Providers::market`.

use serde::{Deserialize, Serialize};

/// Which set of market movers to fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MoverDirection {
    /// Largest percentage gainers.
    Gainers,
    /// Largest percentage losers.
    Losers,
    /// Highest traded volume.
    MostActive,
}

/// A sector's aggregate performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct SectorPerformance {
    /// Sector name (e.g. `"Technology"`).
    pub sector: String,
    /// Exchange the average was computed over (e.g. `"NASDAQ"`). Providers
    /// report one row per sector per exchange.
    pub exchange: Option<String>,
    /// Percentage change, as a number (e.g. `1.23` for `"1.23%"`).
    pub change_percent: Option<f64>,
}

/// One day of aggregate performance across every sector and exchange.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct SectorPerformanceHistory {
    /// Date (`YYYY-MM-DD`).
    pub date: Option<String>,
    /// Per-sector percentage change on that date, one entry per
    /// (sector, exchange) pair.
    #[cfg_attr(feature = "dataframe", dataframe(nested))]
    pub sectors: Vec<SectorPerformance>,
}

/// A sector's aggregate price/earnings ratio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct SectorPe {
    /// Sector name.
    pub sector: String,
    /// Exchange the ratio was computed over.
    pub exchange: Option<String>,
    /// Price/earnings ratio.
    pub pe: Option<f64>,
    /// As-of date (`YYYY-MM-DD`).
    pub date: Option<String>,
}

/// An industry's aggregate price/earnings ratio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct IndustryPe {
    /// Industry name.
    pub industry: String,
    /// Exchange the ratio was computed over.
    pub exchange: Option<String>,
    /// Price/earnings ratio.
    pub pe: Option<f64>,
    /// As-of date (`YYYY-MM-DD`).
    pub date: Option<String>,
}

/// A symbol appearing in a market-movers list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[cfg_attr(feature = "dataframe", dataframe(crate = "crate"))]
#[non_exhaustive]
pub struct MoverQuote {
    /// Ticker symbol.
    pub symbol: String,
    /// Company name.
    pub name: Option<String>,
    /// Latest price.
    pub price: Option<f64>,
    /// Absolute price change.
    pub change: Option<f64>,
    /// Percentage price change.
    pub change_percent: Option<f64>,
    /// Exchange the symbol trades on.
    pub exchange: Option<String>,
}
