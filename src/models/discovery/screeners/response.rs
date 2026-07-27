use super::quote::ScreenerQuote;
use serde::{Deserialize, Serialize};

/// Raw response structure from Yahoo Finance screener API (predefined screeners)
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

/// Raw response structure from Yahoo Finance custom screener API (POST endpoint)
///
/// Uses records[] instead of quotes[] and has different nesting.
#[derive(Debug, Clone, Deserialize)]
struct RawCustomScreenerResponse {
    finance: RawCustomFinance,
}

#[derive(Debug, Clone, Deserialize)]
struct RawCustomFinance {
    result: Option<Vec<RawCustomResult>>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCustomResult {
    /// Total number of matches
    #[serde(default)]
    total: Option<i64>,
    /// Records returned (custom screener uses "records" instead of "quotes")
    #[serde(default)]
    records: Vec<serde_json::Value>,
    // Note: Custom screener doesn't have canonicalName, description, etc.
}

/// Flattened, user-friendly response for screener results
///
/// Returned by the screeners API with a clean structure:
/// ```json
/// {
///   "quotes": [...],
///   "type": "most_actives",
///   "description": "Stocks ordered in descending order by intraday trade volume",
///   "lastUpdated": 1234567890,
///   "total": 100
/// }
/// ```
///
/// This removes Yahoo Finance's nested wrapper structure and internal metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerResults {
    /// Array of quotes matching the screener criteria
    pub quotes: Vec<ScreenerQuote>,

    /// Screener type (e.g., "most_actives", "day_gainers", "custom")
    #[serde(rename = "type")]
    pub screener_type: String,

    /// Human-readable description of the screener
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Last updated timestamp (Unix epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<i64>,

    /// Total number of matching results (for pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

impl ScreenerResults {
    /// Create a flattened response from raw Yahoo Finance JSON (predefined screeners)
    ///
    /// Converts the nested Yahoo Finance response structure into a clean,
    /// user-friendly format by extracting data from finance.result[0].
    ///
    /// # Errors
    ///
    /// Returns an error if the response contains no screener data.
    pub(crate) fn from_response(raw: serde_json::Value) -> Result<Self, String> {
        // Deserialize the raw response
        let raw_response: RawScreenersResponse = serde_json::from_value(raw)
            .map_err(|e| format!("Failed to parse screener response: {}", e))?;

        // Extract the first result
        let result = raw_response
            .finance
            .result
            .into_iter()
            .next()
            .ok_or_else(|| "No screener data in response".to_string())?;

        Ok(Self {
            quotes: result.quotes,
            screener_type: result.canonical_name,
            description: result.description,
            last_updated: result.last_updated,
            total: None, // Predefined screeners don't provide total count
        })
    }

    /// Create a flattened response from raw Yahoo Finance JSON (custom screeners)
    ///
    /// Custom screeners return records in a different format (raw JSON values)
    /// that need to be mapped to ScreenerQuote fields.
    ///
    /// # Errors
    ///
    /// Returns an error if the response contains no screener data or has an error.
    pub(crate) fn from_custom_response(raw: serde_json::Value) -> Result<Self, String> {
        // Deserialize the raw response
        let raw_response: RawCustomScreenerResponse = serde_json::from_value(raw)
            .map_err(|e| format!("Failed to parse custom screener response: {}", e))?;

        // Check for error
        if let Some(err) = raw_response.finance.error {
            return Err(format!("Yahoo Finance error: {}", err));
        }

        // Extract the first result
        let results = raw_response
            .finance
            .result
            .ok_or_else(|| "No result in response".to_string())?;

        let result = results
            .into_iter()
            .next()
            .ok_or_else(|| "No screener data in response".to_string())?;

        // Convert records to ScreenerQuote using a custom mapping
        // Custom screener returns flat records that need field mapping
        let quotes: Vec<ScreenerQuote> = result
            .records
            .into_iter()
            .filter_map(|record| map_custom_record_to_quote(record).ok())
            .collect();

        Ok(Self {
            quotes,
            screener_type: "custom".to_string(),
            description: Some("Custom screener query results".to_string()),
            last_updated: None,
            total: result.total,
        })
    }
}

/// Map a custom screener record to a ScreenerQuote
///
/// Custom screener records use different field names than predefined screeners.
#[cfg(feature = "dataframe")]
impl ScreenerResults {
    /// Converts the quotes to a polars DataFrame.
    ///
    /// Each quote becomes a row with all available columns.
    pub fn to_dataframe(&self) -> ::polars::prelude::PolarsResult<::polars::prelude::DataFrame> {
        ScreenerQuote::vec_to_dataframe(&self.quotes)
    }
}

fn map_custom_record_to_quote(record: serde_json::Value) -> Result<ScreenerQuote, String> {
    use crate::models::quote::FormattedValue;

    let mut record = match record {
        serde_json::Value::Object(map) => map,
        _ => return Err("Record is not an object".to_string()),
    };

    // Helper to extract FormattedValue from Yahoo's format, taking ownership of the field
    fn extract_formatted<T: serde::de::DeserializeOwned + Default>(
        record: &mut serde_json::Map<String, serde_json::Value>,
        field: &str,
    ) -> Option<FormattedValue<T>> {
        record.remove(field).and_then(|v| {
            // Custom screener returns {raw, fmt} or just a value
            if v.is_object() {
                serde_json::from_value(v).ok()
            } else {
                // Wrap plain value in FormattedValue
                serde_json::from_value::<T>(v)
                    .ok()
                    .map(|raw| FormattedValue {
                        raw: Some(raw),
                        fmt: None,
                        long_fmt: None,
                    })
            }
        })
    }

    fn extract_string(
        record: &mut serde_json::Map<String, serde_json::Value>,
        field: &str,
    ) -> Option<String> {
        match record.remove(field) {
            Some(serde_json::Value::String(s)) => Some(s),
            _ => None,
        }
    }

    // Required fields
    let symbol = extract_string(&mut record, "ticker")
        .or_else(|| extract_string(&mut record, "symbol"))
        .ok_or_else(|| "Missing symbol/ticker field".to_string())?;

    let short_name = extract_string(&mut record, "companyshortname")
        .or_else(|| extract_string(&mut record, "shortName"))
        .unwrap_or_else(|| symbol.clone());

    // Price fields - use intradayprice for custom screener
    let regular_market_price = extract_formatted::<f64>(&mut record, "intradayprice")
        .or_else(|| extract_formatted::<f64>(&mut record, "regularMarketPrice"))
        .unwrap_or_default();

    let regular_market_change = extract_formatted::<f64>(&mut record, "intradaypricechange")
        .or_else(|| extract_formatted::<f64>(&mut record, "regularMarketChange"))
        .unwrap_or_default();

    let regular_market_change_percent = extract_formatted::<f64>(&mut record, "percentchange")
        .or_else(|| extract_formatted::<f64>(&mut record, "regularMarketChangePercent"))
        .unwrap_or_default();

    Ok(ScreenerQuote {
        symbol,
        short_name,
        long_name: extract_string(&mut record, "longName"),
        display_name: extract_string(&mut record, "displayName"),
        quote_type: extract_string(&mut record, "quoteType")
            .unwrap_or_else(|| "EQUITY".to_string()),
        exchange: extract_string(&mut record, "exchange").unwrap_or_default(),
        regular_market_price,
        regular_market_change,
        regular_market_change_percent,
        regular_market_open: extract_formatted(&mut record, "day_open_price")
            .or_else(|| extract_formatted(&mut record, "regularMarketOpen")),
        regular_market_day_high: extract_formatted(&mut record, "dayhigh")
            .or_else(|| extract_formatted(&mut record, "regularMarketDayHigh")),
        regular_market_day_low: extract_formatted(&mut record, "daylow")
            .or_else(|| extract_formatted(&mut record, "regularMarketDayLow")),
        regular_market_previous_close: extract_formatted(&mut record, "regularMarketPreviousClose"),
        regular_market_time: extract_formatted(&mut record, "regularMarketTime"),
        regular_market_volume: extract_formatted(&mut record, "dayvolume")
            .or_else(|| extract_formatted(&mut record, "regularMarketVolume")),
        average_daily_volume3_month: extract_formatted(&mut record, "avgdailyvol3m")
            .or_else(|| extract_formatted(&mut record, "averageDailyVolume3Month")),
        average_daily_volume10_day: extract_formatted(&mut record, "averageDailyVolume10Day"),
        market_cap: extract_formatted(&mut record, "intradaymarketcap")
            .or_else(|| extract_formatted(&mut record, "marketCap")),
        shares_outstanding: extract_formatted(&mut record, "sharesOutstanding"),
        fifty_two_week_high: extract_formatted(&mut record, "fiftytwowkhigh")
            .or_else(|| extract_formatted(&mut record, "fiftyTwoWeekHigh")),
        fifty_two_week_low: extract_formatted(&mut record, "fiftytwowklow")
            .or_else(|| extract_formatted(&mut record, "fiftyTwoWeekLow")),
        fifty_two_week_change: extract_formatted(&mut record, "fiftyTwoWeekChange"),
        fifty_two_week_change_percent: extract_formatted(&mut record, "fiftyTwoWeekChangePercent"),
        fifty_day_average: extract_formatted(&mut record, "fiftyDayAverage"),
        fifty_day_average_change: extract_formatted(&mut record, "fiftyDayAverageChange"),
        fifty_day_average_change_percent: extract_formatted(
            &mut record,
            "fiftyDayAverageChangePercent",
        ),
        two_hundred_day_average: extract_formatted(&mut record, "twoHundredDayAverage"),
        two_hundred_day_average_change: extract_formatted(
            &mut record,
            "twoHundredDayAverageChange",
        ),
        two_hundred_day_average_change_percent: extract_formatted(
            &mut record,
            "twoHundredDayAverageChangePercent",
        ),
        average_analyst_rating: extract_string(&mut record, "averageAnalystRating"),
        trailing_pe: extract_formatted::<f64>(&mut record, "peratio.lasttwelvemonths")
            .or_else(|| extract_formatted(&mut record, "trailingPE")),
        forward_pe: extract_formatted(&mut record, "forwardPE"),
        price_to_book: extract_formatted(&mut record, "priceToBook"),
        book_value: extract_formatted(&mut record, "bookValue"),
        eps_trailing_twelve_months: extract_formatted::<f64>(&mut record, "eps.lasttwelvemonths")
            .or_else(|| extract_formatted(&mut record, "epsTrailingTwelveMonths")),
        eps_forward: extract_formatted(&mut record, "epsForward"),
        eps_current_year: extract_formatted(&mut record, "epsCurrentYear"),
        price_eps_current_year: extract_formatted(&mut record, "priceEpsCurrentYear"),
        dividend_yield: extract_formatted::<f64>(&mut record, "annual_dividend_yield")
            .or_else(|| extract_formatted(&mut record, "dividendYield")),
        dividend_rate: extract_formatted::<f64>(&mut record, "annual_dividend_rate")
            .or_else(|| extract_formatted(&mut record, "dividendRate")),
        dividend_date: extract_formatted(&mut record, "dividendDate"),
        trailing_annual_dividend_rate: extract_formatted(&mut record, "trailingAnnualDividendRate"),
        trailing_annual_dividend_yield: extract_formatted(
            &mut record,
            "trailingAnnualDividendYield",
        ),
        bid: extract_formatted(&mut record, "bid"),
        bid_size: extract_formatted(&mut record, "bidSize"),
        ask: extract_formatted(&mut record, "ask"),
        ask_size: extract_formatted(&mut record, "askSize"),
        post_market_price: extract_formatted(&mut record, "postMarketPrice"),
        post_market_change: extract_formatted(&mut record, "postMarketChange"),
        post_market_change_percent: extract_formatted(&mut record, "postMarketChangePercent"),
        post_market_time: extract_formatted(&mut record, "postMarketTime"),
        pre_market_price: extract_formatted(&mut record, "preMarketPrice"),
        pre_market_change: extract_formatted(&mut record, "preMarketChange"),
        pre_market_change_percent: extract_formatted(&mut record, "preMarketChangePercent"),
        pre_market_time: extract_formatted(&mut record, "preMarketTime"),
        earnings_timestamp: extract_formatted(&mut record, "earningsTimestamp"),
        earnings_timestamp_start: extract_formatted(&mut record, "earningsTimestampStart"),
        earnings_timestamp_end: extract_formatted(&mut record, "earningsTimestampEnd"),
        currency: extract_string(&mut record, "quotesCurrency")
            .or_else(|| extract_string(&mut record, "currency")),
    })
}
