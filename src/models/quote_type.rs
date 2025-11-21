//! Quote type model for symbol metadata

use serde::{Deserialize, Serialize};

/// Response wrapper for quote type endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTypeResponse {
    /// Quote type container
    pub quote_type: QuoteTypeContainer,
}

/// Container for quote type results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteTypeContainer {
    /// Quote type results
    pub result: Vec<QuoteTypeResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Quote type result for a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTypeResult {
    /// Stock symbol
    pub symbol: String,
    /// Quote type (EQUITY, ETF, etc.)
    pub quote_type: Option<String>,
    /// Exchange
    pub exchange: Option<String>,
    /// Short name
    pub short_name: Option<String>,
    /// Long name
    pub long_name: Option<String>,
    /// Message board ID
    pub message_board_id: Option<String>,
    /// Exchange timezone name
    pub exchange_timezone_name: Option<String>,
    /// Exchange timezone short name
    pub exchange_timezone_short_name: Option<String>,
    /// GMT offset in milliseconds
    pub gmt_off_set_milliseconds: Option<i64>,
    /// Market
    pub market: Option<String>,
    /// Is EsgPopulated
    pub is_esg_populated: Option<bool>,
    /// Quartr ID (company ID for earnings transcripts)
    #[serde(rename = "quartrId")]
    pub quartr_id: Option<String>,
}

impl QuoteTypeResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get the first result
    pub fn first(&self) -> Option<&QuoteTypeResult> {
        self.quote_type.result.first()
    }

    /// Get the quartr ID (company ID) for earnings transcripts
    pub fn quartr_id(&self) -> Option<&str> {
        self.first().and_then(|r| r.quartr_id.as_deref())
    }
}
