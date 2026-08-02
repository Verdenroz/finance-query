//! Market data models.
//!
//! Market summary, sectors, industries, hours, currencies, exchanges, and index trends.

pub mod currencies;
/// Exchange information.
pub mod exchanges;
/// Market trading hours.
pub mod hours;
/// Industry-level market data.
pub mod industries;
/// Market summary (indices, commodities, forex overview).
pub mod market_summary;
/// Currency pair data.
/// Market-wide performance statistics routed through `Capability::MARKET`.
#[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
pub mod performance;
/// Sector-level market data.
pub mod sectors;

// quoteSummary modules (canonical home, re-exported from quote/ for backward compat)
pub(crate) mod index_trend;
pub(crate) use index_trend::{IndexTrend, IndustryTrend, SectorTrend};
