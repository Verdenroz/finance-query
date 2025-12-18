use super::quote::MoverQuote;
use serde::{Deserialize, Serialize};

/// Raw response structure from Yahoo Finance movers API
///
/// This matches Yahoo's nested response format with finance.result[] wrapper.
/// Use `MoversResponse::from_response()` to convert to user-friendly format.
#[derive(Debug, Clone, Deserialize)]
struct RawMoversResponse {
    finance: RawFinance,
}

#[derive(Debug, Clone, Deserialize)]
struct RawFinance {
    result: Vec<RawResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawResult {
    canonical_name: String,
    quotes: Vec<MoverQuote>,
    #[serde(default)]
    last_updated: Option<i64>,
    #[serde(default)]
    description: Option<String>,
    // Skip internal Yahoo fields: count, id
}

/// Flattened, user-friendly response for market movers
///
/// Returned by the movers API with a clean structure:
/// ```json
/// {
///   "quotes": [...],
///   "type": "MOST_ACTIVES",
///   "description": "Most actively traded stocks...",
///   "lastUpdated": 1234567890
/// }
/// ```
///
/// This removes Yahoo Finance's nested wrapper structure and internal metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoversResponse {
    /// Array of quotes (most actives, gainers, or losers)
    pub quotes: Vec<MoverQuote>,

    /// Type of movers (e.g., "MOST_ACTIVES", "DAY_GAINERS", "DAY_LOSERS")
    #[serde(rename = "type")]
    pub mover_type: String,

    /// Human-readable description of the mover type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Last updated timestamp (Unix epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
}

impl MoversResponse {
    /// Create a flattened response from raw Yahoo Finance JSON
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format by extracting data from finance.result[0].
    ///
    /// # Errors
    ///
    /// Returns an error if the response contains no mover data.
    pub(crate) fn from_response(raw: &serde_json::Value) -> Result<Self, String> {
        // Deserialize the raw response
        let raw_response: RawMoversResponse = serde_json::from_value(raw.clone())
            .map_err(|e| format!("Failed to parse movers response: {}", e))?;

        // Extract the first result
        let result = raw_response
            .finance
            .result
            .first()
            .ok_or_else(|| "No mover data in response".to_string())?;

        Ok(Self {
            quotes: result.quotes.clone(),
            mover_type: result.canonical_name.clone(),
            description: result.description.clone(),
            last_updated: result.last_updated,
        })
    }
}
