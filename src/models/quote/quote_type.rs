/// Quote Type module data
///
/// Contains metadata about the symbol including exchange, type, and timezone information.
use serde::{Deserialize, Serialize};

/// Response wrapper for quote type endpoint
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuoteTypeResponse {
    /// Quote type container
    pub quote_type: QuoteTypeContainer,
}

/// Container for quote type results
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QuoteTypeContainer {
    /// Quote type results
    pub result: Vec<QuoteTypeResult>,
    /// Error if any
    pub error: Option<serde_json::Value>,
}

/// Quote type result for a symbol
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuoteTypeResult {
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

/// Quote type metadata for a symbol (used in quoteSummary module)
///
/// Contains exchange information, company names, timezone data, and other metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteTypeData {
    /// Exchange code (e.g., "NMS", "NYQ")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,

    /// First trade date as Unix epoch UTC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_trade_date_epoch_utc: Option<i64>,

    /// GMT offset in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gmt_off_set_milliseconds: Option<i64>,

    /// Full company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_name: Option<String>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Message board ID for discussions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_board_id: Option<String>,

    /// Quote type (e.g., "EQUITY", "ETF", "MUTUALFUND")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_type: Option<String>,

    /// Short company name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,

    /// Stock symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// Full timezone name (e.g., "America/New_York")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone_full_name: Option<String>,

    /// Short timezone name (e.g., "EST", "PST")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone_short_name: Option<String>,

    /// Underlying symbol (for derivatives, usually same as symbol for stocks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying_symbol: Option<String>,

    /// Unique identifier UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_quote_type() {
        let json = r#"{
            "exchange": "NMS",
            "firstTradeDateEpochUtc": 345479400,
            "gmtOffSetMilliseconds": -18000000,
            "longName": "Apple Inc.",
            "maxAge": 1,
            "messageBoardId": "finmb_24937",
            "quoteType": "EQUITY",
            "shortName": "Apple Inc.",
            "symbol": "AAPL",
            "timeZoneFullName": "America/New_York",
            "timeZoneShortName": "EST",
            "underlyingSymbol": "AAPL",
            "uuid": "8b10e4ae-9eeb-3684-921a-9ab27e4d87aa"
        }"#;

        let quote_type: QuoteTypeData = serde_json::from_str(json).unwrap();
        assert_eq!(quote_type.symbol.as_deref(), Some("AAPL"));
        assert_eq!(quote_type.exchange.as_deref(), Some("NMS"));
        assert_eq!(quote_type.quote_type.as_deref(), Some("EQUITY"));
        assert_eq!(quote_type.long_name.as_deref(), Some("Apple Inc."));
    }
}
