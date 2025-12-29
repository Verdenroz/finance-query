//! Exchange models for Yahoo Finance supported exchanges.

use serde::{Deserialize, Serialize};

/// Information about a supported exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exchange {
    /// Country or region where the exchange operates.
    pub country: String,
    /// Name of the market or index.
    pub market: String,
    /// Symbol suffix used for this exchange (e.g., ".L" for London).
    /// "N/A" if no suffix is needed.
    pub suffix: String,
    /// Data delay (e.g., "Real-time", "15 min", "20 min").
    pub delay: String,
    /// Data provider (e.g., "ICE Data Services").
    pub data_provider: String,
}
