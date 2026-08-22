//! Company profile model.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route. Alpha Vantage is currently the only provider. Scoped to identity
//! and classification fields — valuation ratios and earnings figures live in
//! [`KeyMetricsTtm`](crate::KeyMetricsTtm), [`RatingConsensus`](crate::RatingConsensus),
//! and [`EarningsSurprise`](crate::EarningsSurprise) instead of being duplicated here.

use serde::{Deserialize, Serialize};

/// A company's identity and classification profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyProfile {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Business description.
    pub description: Option<String>,
    /// Asset type as reported by the provider (e.g. `"Common Stock"`).
    pub asset_type: Option<String>,
    /// Listing exchange.
    pub exchange: Option<String>,
    /// Trading currency.
    pub currency: Option<String>,
    /// Country of incorporation or primary listing.
    pub country: Option<String>,
    /// GICS sector.
    pub sector: Option<String>,
    /// GICS industry.
    pub industry: Option<String>,
    /// Market capitalization.
    pub market_capitalization: Option<f64>,
}
