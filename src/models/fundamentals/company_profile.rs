//! Company profile model.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route through Yahoo, Alpha Vantage, or FMP. Scoped to identity
//! and classification fields — valuation ratios and earnings figures live in
//! [`KeyMetricsTtm`](crate::KeyMetricsTtm), [`RatingConsensus`](crate::RatingConsensus),
//! and [`EarningsSurprise`](crate::EarningsSurprise) instead of being duplicated here.

use serde::{Deserialize, Serialize};

/// A company's identity and classification profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyProfile {
    /// Provider-reported ISIN, preserving its exact text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isin: Option<String>,
    /// Provider-reported CUSIP, when supplied by this provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cusip: Option<String>,
    /// Provider-reported current trading status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Whether the provider classifies this instrument as an ETF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_etf: Option<bool>,
    /// Whether the provider classifies this instrument as an ADR.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_adr: Option<bool>,
    /// Whether the provider classifies this instrument as a fund.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_fund: Option<bool>,
    /// SEC company identifier, preserving leading zeros.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cik: Option<String>,
    /// Provider-reported IPO date, in YYYY-MM-DD format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ipo_date: Option<String>,
    /// Provider that supplied this profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<crate::Provider>,
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
