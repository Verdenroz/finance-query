//! ETF profile and holdings models.
//!
//! Served through the [`Capability::FUNDAMENTALS`](crate::Capability::FUNDAMENTALS)
//! route; Alpha Vantage is currently the only provider, and the only wired
//! source of ETF holdings at all.

use serde::{Deserialize, Serialize};

/// Profile and composition of an exchange-traded fund.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfProfile {
    /// Fund ticker symbol.
    pub symbol: Option<String>,
    /// Fund name.
    pub name: Option<String>,
    /// Asset type as reported by the provider.
    pub asset_type: Option<String>,
    /// Total net assets.
    pub net_assets: Option<f64>,
    /// Net expense ratio, as a fraction (0.0003 = 3 bps).
    pub net_expense_ratio: Option<f64>,
    /// Annual portfolio turnover, as a fraction.
    pub portfolio_turnover: Option<f64>,
    /// Trailing dividend yield, as a fraction.
    pub dividend_yield: Option<f64>,
    /// Inception date (`YYYY-MM-DD`).
    pub inception_date: Option<String>,
    /// Portfolio holdings, heaviest first.
    pub holdings: Vec<EtfHolding>,
}

/// One position inside an ETF's portfolio.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfHolding {
    /// Ticker symbol of the held security.
    pub symbol: Option<String>,
    /// Security description.
    pub description: Option<String>,
    /// Portfolio weight, as a fraction of net assets.
    pub weight: Option<f64>,
}
