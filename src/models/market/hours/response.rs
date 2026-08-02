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
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
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
pub struct MarketHours {
    /// List of market times
    pub markets: Vec<MarketTime>,
}

/// Display label for a Yahoo market id, for the regions the crate supports.
///
/// Yahoo's markettime endpoint returns correct per-region session times and
/// ids, but hardcodes the `"U.S. markets"` label into `name` and `message`
/// for every region. Unknown ids return `None` and keep Yahoo's label as-is.
fn market_label(id: &str) -> Option<&'static str> {
    Some(match id {
        "us" => "U.S. markets",
        "gb" | "uk" => "U.K. markets",
        "ar" => "Argentine markets",
        "au" => "Australian markets",
        "br" => "Brazilian markets",
        "ca" => "Canadian markets",
        "cn" => "Chinese markets",
        "de" => "German markets",
        "dk" => "Danish markets",
        "es" => "Spanish markets",
        "fi" => "Finnish markets",
        "fr" => "French markets",
        "gr" => "Greek markets",
        "hk" => "Hong Kong markets",
        "id" => "Indonesian markets",
        "il" => "Israeli markets",
        "in" => "Indian markets",
        "it" => "Italian markets",
        "jp" => "Japanese markets",
        "kr" => "South Korean markets",
        "mx" => "Mexican markets",
        "my" => "Malaysian markets",
        "nl" => "Dutch markets",
        "no" => "Norwegian markets",
        "nz" => "New Zealand markets",
        "pt" => "Portuguese markets",
        "qa" => "Qatari markets",
        "ru" => "Russian markets",
        "se" => "Swedish markets",
        "sg" => "Singaporean markets",
        "th" => "Thai markets",
        "tr" => "Turkish markets",
        "tw" => "Taiwanese markets",
        "vn" => "Vietnamese markets",
        _ => return None,
    })
}

impl MarketHours {
    /// Create a flattened response from raw Yahoo Finance JSON
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format. Yahoo labels every region's market
    /// `"U.S. markets"` in `name`/`message` (the times themselves are
    /// correct); the label is corrected from the market id here.
    pub(crate) fn from_response(raw: serde_json::Value) -> Result<Self, String> {
        let raw_response: RawHoursResponse = serde_json::from_value(raw)
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

                let (name, message) = match market_label(&market_time.id) {
                    Some(label) if label != market_time.name => (
                        label.to_string(),
                        market_time
                            .message
                            .as_ref()
                            .map(|m| m.replace(&market_time.name, label)),
                    ),
                    _ => (market_time.name.clone(), market_time.message.clone()),
                };

                markets.push(MarketTime {
                    id: market_time.id.clone(),
                    name,
                    status: market_time.status.clone(),
                    message,
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

#[cfg(feature = "dataframe")]
impl MarketHours {
    /// Converts the market times to a polars DataFrame.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        MarketTime::vec_to_dataframe(&self.markets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(id: &str, name: &str, message: &str) -> serde_json::Value {
        serde_json::json!({
            "finance": {
                "marketTimes": [{
                    "marketTime": [{
                        "id": id,
                        "name": name,
                        "status": "closed",
                        "message": message,
                        "open": "2026-08-03T00:00:00Z",
                        "close": "2026-08-03T06:30:00Z",
                        "time": "2026-08-02T21:14:10Z",
                        "timezone": [{
                            "dst": "false",
                            "gmtoffset": "32400",
                            "short": "JST",
                            "$text": "Asia/Tokyo"
                        }]
                    }]
                }]
            }
        })
    }

    #[test]
    fn corrects_yahoo_us_label_for_non_us_market() {
        // Yahoo returns correct JP session times but labels them "U.S. markets".
        let hours = MarketHours::from_response(raw(
            "jp",
            "U.S. markets",
            "U.S. markets open in 2 hours 46 minutes",
        ))
        .unwrap();
        let m = &hours.markets[0];
        assert_eq!(m.name, "Japanese markets");
        assert_eq!(
            m.message.as_deref(),
            Some("Japanese markets open in 2 hours 46 minutes")
        );
        // Times pass through untouched.
        assert_eq!(m.open.as_deref(), Some("2026-08-03T00:00:00Z"));
        assert_eq!(m.timezone.as_deref(), Some("Asia/Tokyo"));
    }

    #[test]
    fn us_label_and_unknown_ids_pass_through() {
        let us =
            MarketHours::from_response(raw("us", "U.S. markets", "U.S. markets closed")).unwrap();
        assert_eq!(us.markets[0].name, "U.S. markets");
        assert_eq!(
            us.markets[0].message.as_deref(),
            Some("U.S. markets closed")
        );

        // Unknown id: keep whatever Yahoo said rather than guessing.
        let other =
            MarketHours::from_response(raw("zz", "U.S. markets", "U.S. markets closed")).unwrap();
        assert_eq!(other.markets[0].name, "U.S. markets");
    }
}
