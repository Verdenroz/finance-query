//! Cross-market snapshot model.
//!
//! Served through the [`Capability::QUOTE`](crate::Capability::QUOTE) route by
//! providers whose snapshot endpoint spans asset classes; Polygon is currently
//! the only one. A single request can mix equities, options contracts, FX
//! pairs, crypto pairs, and indices — see
//! [`Providers::snapshot`](crate::Providers::snapshot).

use serde::{Deserialize, Serialize};

/// Which market a snapshot row belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AssetClass {
    /// Equities.
    Stocks,
    /// Options contracts.
    Options,
    /// Currency pairs.
    Fx,
    /// Cryptocurrency pairs.
    Crypto,
    /// Stock market indices.
    Indices,
}

impl AssetClass {
    /// Lowercase provider-facing name (`"stocks"`, `"fx"`, …).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stocks => "stocks",
            Self::Options => "options",
            Self::Fx => "fx",
            Self::Crypto => "crypto",
            Self::Indices => "indices",
        }
    }
}

/// One symbol's current market state, flattened across asset classes.
///
/// Providers return a row per requested symbol even when the lookup failed, so
/// a batch is never silently short: check [`error`](Self::error) before reading
/// the price fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketSnapshot {
    /// Ticker symbol as the provider spells it (e.g. `"AAPL"`, `"X:BTCUSD"`).
    pub symbol: Option<String>,
    /// Human-readable name of the instrument.
    pub name: Option<String>,
    /// Market this row belongs to.
    pub asset_class: Option<AssetClass>,
    /// Trading status of that market (e.g. `"open"`, `"closed"`).
    pub market_status: Option<String>,
    /// Most recent trade price.
    pub last_price: Option<f64>,
    /// Best bid.
    pub bid: Option<f64>,
    /// Best ask.
    pub ask: Option<f64>,
    /// Session open.
    pub open: Option<f64>,
    /// Session high.
    pub high: Option<f64>,
    /// Session low.
    pub low: Option<f64>,
    /// Session close (last price of the current session).
    pub close: Option<f64>,
    /// Previous session's close.
    pub previous_close: Option<f64>,
    /// Session volume.
    pub volume: Option<f64>,
    /// Absolute change over the session.
    pub change: Option<f64>,
    /// Percentage change over the session.
    pub change_percent: Option<f64>,
    /// Per-symbol error code, if the provider could not resolve this symbol.
    pub error: Option<String>,
    /// Human-readable message accompanying [`error`](Self::error).
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_class_round_trips_through_provider_spelling() {
        for (variant, name) in [
            (AssetClass::Stocks, "stocks"),
            (AssetClass::Options, "options"),
            (AssetClass::Fx, "fx"),
            (AssetClass::Crypto, "crypto"),
            (AssetClass::Indices, "indices"),
        ] {
            assert_eq!(variant.as_str(), name);
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{name}\""));
            assert_eq!(
                serde_json::from_str::<AssetClass>(&json).unwrap(),
                variant,
                "{name}"
            );
        }
    }
}
