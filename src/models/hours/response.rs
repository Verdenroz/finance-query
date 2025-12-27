use serde::{Deserialize, Serialize};

// ============================================================================
// Raw Yahoo Finance response structures (internal)
// ============================================================================

/// Raw response from Yahoo Finance markettime endpoint
#[derive(Debug, Clone, Deserialize)]
struct RawHoursResponse {
    finance: RawFinance,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFinance {
    #[serde(default)]
    market_times: Vec<RawMarketTimes>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMarketTimes {
    #[serde(default)]
    market_time: Vec<RawMarketTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMarketTime {
    id: String,
    name: String,
    status: String,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    open: Option<String>,
    #[serde(default)]
    close: Option<String>,
    #[serde(default)]
    time: Option<String>,
    #[serde(default)]
    timezone: Vec<RawTimezone>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawTimezone {
    #[serde(default)]
    dst: Option<String>,
    #[serde(default)]
    gmtoffset: Option<String>,
    #[serde(default)]
    short: Option<String>,
    #[serde(rename = "$text", default)]
    text: Option<String>,
}

// ============================================================================
// Public API structures
// ============================================================================

/// Market time information for a specific market
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MarketTime {
    /// Market identifier (e.g., "us", "uk", "jp")
    pub id: String,

    /// Human-readable market name (e.g., "U.S. markets")
    pub name: String,

    /// Market status (e.g., "open", "closed")
    pub status: String,

    /// Status message (e.g., "U.S. markets closed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Market open time (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,

    /// Market close time (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<String>,

    /// Current time (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    /// Timezone name (e.g., "America/New_York")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Short timezone name (e.g., "EST")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone_short: Option<String>,

    /// GMT offset in seconds (e.g., -18000 for EST)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gmt_offset: Option<i32>,

    /// Whether daylight saving time is in effect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst: Option<bool>,
}

/// Flattened response for market hours
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct HoursResponse {
    /// List of market times
    pub markets: Vec<MarketTime>,
}

impl HoursResponse {
    /// Create a flattened response from raw Yahoo Finance JSON
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format.
    pub(crate) fn from_response(raw: &serde_json::Value) -> Result<Self, String> {
        let raw_response: RawHoursResponse = serde_json::from_value(raw.clone())
            .map_err(|e| format!("Failed to parse hours response: {}", e))?;

        let mut markets = Vec::new();

        for market_times in &raw_response.finance.market_times {
            for market_time in &market_times.market_time {
                // Extract timezone info from the first timezone entry
                let tz = market_time.timezone.first();

                let gmt_offset = tz
                    .and_then(|t| t.gmtoffset.as_ref())
                    .and_then(|s| s.parse::<i32>().ok());

                let dst = tz
                    .and_then(|t| t.dst.as_ref())
                    .map(|s| s.eq_ignore_ascii_case("true"));

                markets.push(MarketTime {
                    id: market_time.id.clone(),
                    name: market_time.name.clone(),
                    status: market_time.status.clone(),
                    message: market_time.message.clone(),
                    open: market_time.open.clone(),
                    close: market_time.close.clone(),
                    time: market_time.time.clone(),
                    timezone: tz.and_then(|t| t.text.clone()),
                    timezone_short: tz.and_then(|t| t.short.clone()),
                    gmt_offset,
                    dst,
                });
            }
        }

        Ok(Self { markets })
    }
}
