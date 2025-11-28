use super::quote::MoverQuote;
use serde::{Deserialize, Serialize};

/// Root response structure for market movers API
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoversResponse {
    /// Finance data wrapper
    pub finance: MoversFinance,
}

/// Finance data container for movers response
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoversFinance {
    /// Error information, if any
    pub error: Option<serde_json::Value>,
    /// Result array containing mover data
    pub result: Vec<MoversResult>,
}

/// Individual mover result (e.g., most actives, gainers, losers)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoversResult {
    /// Canonical name of the screener (e.g., "MOST_ACTIVES", "DAY_GAINERS", "DAY_LOSERS")
    pub canonical_name: String,
    /// Number of quotes in the result
    pub count: i32,
    /// Array of quote data
    pub quotes: Vec<MoverQuote>,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional screener ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Last updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
}
