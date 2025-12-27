use super::quote::ScreenerQuote;
use serde::{Deserialize, Serialize};

/// Raw response structure from Yahoo Finance screener API
///
/// This matches Yahoo's nested response format with finance.result[] wrapper.
/// Use `ScreenersResponse::from_response()` to convert to user-friendly format.
#[derive(Debug, Clone, Deserialize)]
struct RawScreenersResponse {
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
    quotes: Vec<ScreenerQuote>,
    #[serde(default)]
    last_updated: Option<i64>,
    #[serde(default)]
    description: Option<String>,
    // Skip internal Yahoo fields: count, id
}

/// Flattened, user-friendly response for screener results
///
/// Returned by the screeners API with a clean structure:
/// ```json
/// {
///   "quotes": [...],
///   "type": "most_actives",
///   "description": "Stocks ordered in descending order by intraday trade volume",
///   "lastUpdated": 1234567890
/// }
/// ```
///
/// This removes Yahoo Finance's nested wrapper structure and internal metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenersResponse {
    /// Array of quotes matching the screener criteria
    pub quotes: Vec<ScreenerQuote>,

    /// Screener type (e.g., "most_actives", "day_gainers", "day_losers")
    #[serde(rename = "type")]
    pub screener_type: String,

    /// Human-readable description of the screener
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Last updated timestamp (Unix epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,
}

impl ScreenersResponse {
    /// Create a flattened response from raw Yahoo Finance JSON
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format by extracting data from finance.result[0].
    ///
    /// # Errors
    ///
    /// Returns an error if the response contains no screener data.
    pub(crate) fn from_response(raw: &serde_json::Value) -> Result<Self, String> {
        // Deserialize the raw response
        let raw_response: RawScreenersResponse = serde_json::from_value(raw.clone())
            .map_err(|e| format!("Failed to parse screener response: {}", e))?;

        // Extract the first result
        let result = raw_response
            .finance
            .result
            .first()
            .ok_or_else(|| "No screener data in response".to_string())?;

        Ok(Self {
            quotes: result.quotes.clone(),
            screener_type: result.canonical_name.clone(),
            description: result.description.clone(),
            last_updated: result.last_updated,
        })
    }
}
