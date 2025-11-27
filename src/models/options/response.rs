use super::contract::OptionContract;
use serde::{Deserialize, Serialize};

/// Response wrapper for options endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionsResponse {
    /// Option chain container
    pub option_chain: OptionChainContainer,
}

/// Container for option chain results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionChainContainer {
    /// Results array
    pub result: Vec<OptionChainResult>,

    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Single option chain result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionChainResult {
    /// Underlying symbol
    pub underlying_symbol: Option<String>,

    /// Available expiration dates (Unix timestamps)
    pub expiration_dates: Option<Vec<i64>>,

    /// Available strike prices
    pub strikes: Option<Vec<f64>>,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Quote data
    pub quote: Option<serde_json::Value>,

    /// Options data (array of option chains)
    pub options: Vec<OptionChainData>,
}

/// Option chain data for a specific expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionChainData {
    /// Expiration date (Unix timestamp)
    pub expiration_date: i64,

    /// Whether has mini options
    pub has_mini_options: Option<bool>,

    /// Call options
    pub calls: Option<Vec<OptionContract>>,

    /// Put options
    pub puts: Option<Vec<OptionContract>>,
}

impl OptionsResponse {
    /// Parse from JSON value
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Get the first result (most common case)
    pub fn first_result(&self) -> Option<&OptionChainResult> {
        self.option_chain.result.first()
    }

    /// Get available expiration dates
    pub fn expiration_dates(&self) -> Vec<i64> {
        self.first_result()
            .and_then(|r| r.expiration_dates.clone())
            .unwrap_or_default()
    }

    /// Get strike prices
    pub fn strikes(&self) -> Vec<f64> {
        self.first_result()
            .and_then(|r| r.strikes.clone())
            .unwrap_or_default()
    }
}
