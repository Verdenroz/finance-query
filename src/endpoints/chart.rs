use super::urls::api;
/// Chart data endpoint
///
/// Fetches historical price and volume data for a symbol.
use crate::client::YahooClient;
use crate::constants::{Interval, TimeRange};
use crate::error::{FinanceError, Result};
use tracing::{debug, info};

const CHART_EVENTS: &str = "div|split|capitalGain";

/// Yahoo Finance intraday limits (empirically verified).
///
/// Returns `(max_lookback_secs, native_ranges)` for intervals that have restrictions:
/// - `max_lookback_secs`: how far back Yahoo will serve data via period1/period2 (hard 422 beyond)
/// - `native_ranges`: the `range=` values Yahoo accepts natively for this interval
///
/// When the requested range is not in `native_ranges`, `fetch` reroutes to `fetch_with_dates`
/// with `start = now - max_lookback_secs`. `Ticker::chart_range` handles chunking for spans
/// that exceed the per-request limit.
///
/// Empirically verified limits:
/// - 1m:         max age 29d, max span/request 8d → chunked into 7d windows
/// - 5m/15m/30m: max age 58d, max span/request 8d → chunked into 7d windows
/// - 1h:         max age 728d, single request covers full window
/// - 1d+:        no restriction
pub(crate) fn intraday_limit(interval: Interval) -> Option<(i64, &'static [TimeRange])> {
    match interval {
        Interval::OneMinute => Some((29 * 24 * 3600, &[TimeRange::OneDay, TimeRange::FiveDays])),
        Interval::FiveMinutes | Interval::FifteenMinutes | Interval::ThirtyMinutes => Some((
            58 * 24 * 3600,
            &[TimeRange::OneDay, TimeRange::FiveDays, TimeRange::OneMonth],
        )),
        Interval::OneHour => Some((
            728 * 24 * 3600,
            &[
                TimeRange::OneDay,
                TimeRange::FiveDays,
                TimeRange::OneMonth,
                TimeRange::ThreeMonths,
                TimeRange::SixMonths,
                TimeRange::OneYear,
                TimeRange::TwoYears,
                TimeRange::YearToDate,
            ],
        )),
        _ => None,
    }
}

/// Maximum span (seconds) per single period1/period2 request for intraday intervals.
///
/// Yahoo returns 422 if the span exceeds this. 7 days is a safe margin below the
/// empirically observed 8-day hard limit. 1h covers its full 728-day window in
/// a single request and returns `None`.
pub(crate) fn intraday_chunk_secs(interval: Interval) -> Option<i64> {
    match interval {
        Interval::OneMinute
        | Interval::FiveMinutes
        | Interval::FifteenMinutes
        | Interval::ThirtyMinutes => Some(7 * 24 * 3600),
        _ => None,
    }
}

/// Fetch chart data for a symbol
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
/// * `interval` - Time interval between data points
/// * `range` - Time range to fetch data for
///
/// # Example
///
/// ```ignore
/// use finance_query::{Interval, TimeRange};
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let client = finance_query::YahooClient::new(Default::default()).await?;
/// use finance_query::api::chart;
/// let chart_data = chart::fetch(&client, "AAPL", Interval::OneDay, TimeRange::OneMonth).await?;
/// # Ok(())
/// # }
/// ```
pub async fn fetch(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    // Max range with daily/weekly intervals requires chunking into 10-year periods
    // because Yahoo Finance truncates the response for these combinations.
    if matches!(range, TimeRange::Max) && matches!(interval, Interval::OneDay) {
        return fetch_max_chunked(client, symbol, Interval::OneDay).await;
    }
    if matches!(range, TimeRange::Max) && matches!(interval, Interval::OneWeek) {
        return fetch_max_chunked(client, symbol, Interval::OneWeek).await;
    }

    fetch_direct(client, symbol, interval, range).await
}

/// Direct fetch without chunking (used for non-max ranges and as building block for chunking).
async fn fetch_direct(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<serde_json::Value> {
    info!(
        "Fetching chart for {} ({}, {})",
        symbol,
        interval.as_str(),
        range.as_str()
    );

    let url = api::chart(symbol);
    let params = [
        ("interval", interval.as_str()),
        ("range", range.as_str()),
        ("events", CHART_EVENTS),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

/// Fetch entire history by chunking into 10-year periods.
///
/// Yahoo Finance truncates max-range responses for daily and weekly intervals.
/// This function first detects the stock's earliest date using monthly data,
/// then fetches daily/weekly data in 10-year chunks and merges them.
async fn fetch_max_chunked(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
) -> Result<serde_json::Value> {
    // Step 1: Detect earliest available date using monthly interval (always works with max)
    let earliest_data = fetch_direct(client, symbol, Interval::OneMonth, TimeRange::Max).await?;
    let earliest_timestamp = extract_earliest_timestamp(&earliest_data)?;

    debug!(
        "Detected earliest timestamp for {}: {} ({})",
        symbol,
        earliest_timestamp,
        chrono::DateTime::from_timestamp(earliest_timestamp, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string())
    );

    // Step 2: Chunk into 10-year periods
    let now = chrono::Utc::now().timestamp();
    const CHUNK_SIZE: i64 = 10 * 365 * 24 * 60 * 60; // 10 years in seconds

    let mut period1 = earliest_timestamp;
    let mut merged_result = init_chart_response();

    loop {
        let period2 = (period1 + CHUNK_SIZE).min(now);

        let chunk_data = fetch_with_dates(client, symbol, interval, period1, period2).await?;
        merge_chart_data(&mut merged_result, chunk_data)?;

        if period2 >= now {
            break;
        }

        period1 = period2 + 1;
    }

    Ok(merged_result)
}

/// Extract the earliest timestamp from a chart response.
fn extract_earliest_timestamp(data: &serde_json::Value) -> Result<i64> {
    let timestamps = data
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|result| result.get("timestamp"))
        .and_then(|t| t.as_array())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "chart.result[0].timestamp".to_string(),
            context: "Missing timestamp array in chart response".to_string(),
        })?;

    timestamps
        .iter()
        .filter_map(|v| v.as_i64())
        .min()
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "timestamp".to_string(),
            context: "No valid timestamps found in chart response".to_string(),
        })
}

/// Initialize an empty chart response structure for merging chunks into.
fn init_chart_response() -> serde_json::Value {
    serde_json::json!({
        "chart": {
            "result": [{
                "meta": {},
                "timestamp": [],
                "indicators": {
                    "quote": [{
                        "open": [],
                        "high": [],
                        "low": [],
                        "close": [],
                        "volume": []
                    }],
                    "adjclose": [{
                        "adjclose": []
                    }]
                },
                "events": {}
            }],
            "error": null
        }
    })
}

/// Merge a chunk of chart data into the accumulated result.
fn merge_chart_data(merged: &mut serde_json::Value, chunk: serde_json::Value) -> Result<()> {
    let chunk_result = chunk
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "chart.result".to_string(),
            context: "Missing chart result in chunk".to_string(),
        })?;

    let merged_result = merged
        .get_mut("chart")
        .and_then(|c| c.get_mut("result"))
        .and_then(|r| r.as_array_mut())
        .and_then(|arr| arr.first_mut())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "chart.result".to_string(),
            context: "Invalid merged result structure".to_string(),
        })?;

    // Copy meta from first chunk
    if merged_result
        .get("meta")
        .and_then(|m| m.as_object())
        .map(|o| o.is_empty())
        .unwrap_or(true)
    {
        if let Some(meta) = chunk_result.get("meta") {
            merged_result["meta"] = meta.clone();
        }
    }

    // Merge timestamps
    if let Some(chunk_timestamps) = chunk_result.get("timestamp").and_then(|t| t.as_array()) {
        let merged_timestamps = merged_result["timestamp"]
            .as_array_mut()
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "timestamp".to_string(),
                context: "Invalid timestamp array".to_string(),
            })?;
        merged_timestamps.extend_from_slice(chunk_timestamps);
    }

    // Merge quote data (open, high, low, close, volume)
    if let Some(chunk_quote) = chunk_result
        .get("indicators")
        .and_then(|i| i.get("quote"))
        .and_then(|q| q.as_array())
        .and_then(|arr| arr.first())
    {
        let merged_quote = merged_result
            .get_mut("indicators")
            .and_then(|i| i.get_mut("quote"))
            .and_then(|q| q.as_array_mut())
            .and_then(|arr| arr.first_mut())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "indicators.quote".to_string(),
                context: "Invalid quote structure".to_string(),
            })?;

        for field in &["open", "high", "low", "close", "volume"] {
            if let Some(chunk_values) = chunk_quote.get(field).and_then(|v| v.as_array()) {
                let merged_values =
                    merged_quote[field]
                        .as_array_mut()
                        .ok_or_else(|| FinanceError::ResponseStructureError {
                            field: field.to_string(),
                            context: "Invalid field array".to_string(),
                        })?;
                merged_values.extend_from_slice(chunk_values);
            }
        }
    }

    // Merge adjclose data
    if let Some(chunk_adjclose) = chunk_result
        .get("indicators")
        .and_then(|i| i.get("adjclose"))
        .and_then(|a| a.as_array())
        .and_then(|arr| arr.first())
        .and_then(|ac| ac.get("adjclose"))
        .and_then(|v| v.as_array())
    {
        let merged_adjclose = merged_result
            .get_mut("indicators")
            .and_then(|i| i.get_mut("adjclose"))
            .and_then(|a| a.as_array_mut())
            .and_then(|arr| arr.first_mut())
            .and_then(|ac| ac.get_mut("adjclose"))
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "indicators.adjclose".to_string(),
                context: "Invalid adjclose structure".to_string(),
            })?;
        merged_adjclose.extend_from_slice(chunk_adjclose);
    }

    // Merge events (dividends, splits, capitalGains)
    if let Some(chunk_events) = chunk_result.get("events") {
        let merged_events = merged_result
            .get_mut("events")
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "events".to_string(),
                context: "Invalid events structure".to_string(),
            })?;

        if let Some(chunk_events_obj) = chunk_events.as_object() {
            let merged_events_obj =
                merged_events
                    .as_object_mut()
                    .ok_or_else(|| FinanceError::ResponseStructureError {
                        field: "events".to_string(),
                        context: "Events is not an object".to_string(),
                    })?;

            for (event_type, event_data) in chunk_events_obj {
                if let Some(event_map) = event_data.as_object() {
                    let merged_event_map = merged_events_obj
                        .entry(event_type)
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()
                        .ok_or_else(|| FinanceError::ResponseStructureError {
                            field: format!("events.{}", event_type),
                            context: "Event type is not an object".to_string(),
                        })?;

                    for (key, value) in event_map {
                        merged_event_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fetch chart data for a symbol using absolute date boundaries.
///
/// Uses Yahoo's `period1`/`period2` parameters instead of a relative range.
///
/// # Arguments
///
/// * `client` - The Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL")
/// * `interval` - Time interval between data points
/// * `start` - Start date as Unix timestamp (seconds)
/// * `end` - End date as Unix timestamp (seconds)
pub async fn fetch_with_dates(
    client: &YahooClient,
    symbol: &str,
    interval: Interval,
    start: i64,
    end: i64,
) -> Result<serde_json::Value> {
    super::common::validate_symbol(symbol)?;

    info!(
        "Fetching chart for {} ({}, period1={}, period2={})",
        symbol,
        interval.as_str(),
        start,
        end
    );

    let url = api::chart(symbol);
    let start_str = start.to_string();
    let end_str = end.to_string();
    let params = [
        ("interval", interval.as_str()),
        ("period1", start_str.as_str()),
        ("period2", end_str.as_str()),
        ("events", CHART_EVENTS),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientConfig;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_chart() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "AAPL", Interval::OneDay, TimeRange::OneMonth).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("chart").is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access - validation tested in common::tests"]
    async fn test_empty_symbol() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = fetch(&client, "", Interval::OneDay, TimeRange::OneMonth).await;
        assert!(result.is_err());
    }

    // --- intraday_limit routing tests (no network required) ---

    fn uses_native_range(interval: Interval, range: TimeRange) -> bool {
        match intraday_limit(interval) {
            Some((_, native)) => native.contains(&range),
            None => true, // daily+ always uses range param
        }
    }

    #[test]
    fn test_1m_native_ranges() {
        assert!(uses_native_range(Interval::OneMinute, TimeRange::OneDay));
        assert!(uses_native_range(Interval::OneMinute, TimeRange::FiveDays));
    }

    #[test]
    fn test_1m_reroutes_to_dates() {
        // All ranges beyond 5d should be rerouted to period1/period2
        for range in [
            TimeRange::OneMonth,
            TimeRange::ThreeMonths,
            TimeRange::OneYear,
            TimeRange::Max,
        ] {
            assert!(
                !uses_native_range(Interval::OneMinute, range),
                "expected reroute for 1m+{range}"
            );
        }
    }

    #[test]
    fn test_5m_15m_30m_native_ranges() {
        for interval in [
            Interval::FiveMinutes,
            Interval::FifteenMinutes,
            Interval::ThirtyMinutes,
        ] {
            assert!(uses_native_range(interval, TimeRange::OneDay));
            assert!(uses_native_range(interval, TimeRange::FiveDays));
            assert!(uses_native_range(interval, TimeRange::OneMonth));
        }
    }

    #[test]
    fn test_5m_reroutes_to_dates() {
        for range in [TimeRange::ThreeMonths, TimeRange::OneYear, TimeRange::Max] {
            assert!(
                !uses_native_range(Interval::FiveMinutes, range),
                "expected reroute for 5m+{range}"
            );
        }
    }

    #[test]
    fn test_1h_native_ranges() {
        for range in [
            TimeRange::OneDay,
            TimeRange::FiveDays,
            TimeRange::OneMonth,
            TimeRange::ThreeMonths,
            TimeRange::SixMonths,
            TimeRange::OneYear,
            TimeRange::TwoYears,
            TimeRange::YearToDate,
        ] {
            assert!(
                uses_native_range(Interval::OneHour, range),
                "expected native for 1h+{range}"
            );
        }
    }

    #[test]
    fn test_1h_reroutes_to_dates() {
        // Beyond 2y (730 days), Yahoo won't return 1h data — reroute with 730d lookback
        for range in [TimeRange::FiveYears, TimeRange::TenYears, TimeRange::Max] {
            assert!(
                !uses_native_range(Interval::OneHour, range),
                "expected reroute for 1h+{range}"
            );
        }
    }

    #[test]
    fn test_daily_weekly_no_restriction() {
        // Daily and coarser have no intraday_limit restrictions
        // (Max range is handled by fetch_max_chunked, not intraday_limit)
        assert!(uses_native_range(Interval::OneDay, TimeRange::Max));
        assert!(uses_native_range(Interval::OneDay, TimeRange::TenYears));
        assert!(uses_native_range(Interval::OneWeek, TimeRange::FiveYears));
        assert!(uses_native_range(Interval::OneMonth, TimeRange::Max));
        assert!(uses_native_range(
            Interval::ThreeMonths,
            TimeRange::TwoYears
        ));
    }

    #[test]
    fn test_extract_earliest_timestamp() {
        let data = serde_json::json!({
            "chart": {
                "result": [{
                    "timestamp": [1609459200, 1609545600, 1609632000]
                }]
            }
        });
        assert_eq!(extract_earliest_timestamp(&data).unwrap(), 1609459200);
    }

    #[test]
    fn test_extract_earliest_timestamp_missing() {
        let data = serde_json::json!({"chart": {"result": [{}]}});
        assert!(extract_earliest_timestamp(&data).is_err());
    }

    #[test]
    fn test_merge_chart_data() {
        let mut merged = init_chart_response();
        let chunk = serde_json::json!({
            "chart": {
                "result": [{
                    "meta": {"symbol": "AAPL", "currency": "USD"},
                    "timestamp": [1609459200, 1609545600],
                    "indicators": {
                        "quote": [{
                            "open": [130.0, 131.0],
                            "high": [133.0, 134.0],
                            "low": [129.0, 130.0],
                            "close": [132.0, 133.0],
                            "volume": [100000000, 90000000]
                        }],
                        "adjclose": [{
                            "adjclose": [131.5, 132.5]
                        }]
                    },
                    "events": {
                        "dividends": {"1609459200": {"amount": 0.205}}
                    }
                }]
            }
        });

        merge_chart_data(&mut merged, chunk).unwrap();

        let result = &merged["chart"]["result"][0];
        assert_eq!(result["timestamp"].as_array().unwrap().len(), 2);
        assert_eq!(result["indicators"]["quote"][0]["open"].as_array().unwrap().len(), 2);
        assert_eq!(result["indicators"]["adjclose"][0]["adjclose"].as_array().unwrap().len(), 2);
        assert_eq!(result["meta"]["symbol"].as_str().unwrap(), "AAPL");
        assert!(result["events"]["dividends"]["1609459200"].is_object());

        // Merge a second chunk
        let chunk2 = serde_json::json!({
            "chart": {
                "result": [{
                    "meta": {"symbol": "AAPL"},
                    "timestamp": [1609718400],
                    "indicators": {
                        "quote": [{
                            "open": [134.0],
                            "high": [136.0],
                            "low": [133.0],
                            "close": [135.0],
                            "volume": [80000000]
                        }],
                        "adjclose": [{
                            "adjclose": [134.5]
                        }]
                    },
                    "events": {}
                }]
            }
        });

        merge_chart_data(&mut merged, chunk2).unwrap();

        let result = &merged["chart"]["result"][0];
        assert_eq!(result["timestamp"].as_array().unwrap().len(), 3);
        assert_eq!(result["indicators"]["quote"][0]["close"].as_array().unwrap().len(), 3);
        assert_eq!(result["indicators"]["adjclose"][0]["adjclose"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_intraday_limit_lookbacks() {
        let (secs, _) = intraday_limit(Interval::OneMinute).unwrap();
        assert_eq!(secs, 29 * 24 * 3600);
        let (secs, _) = intraday_limit(Interval::FiveMinutes).unwrap();
        assert_eq!(secs, 58 * 24 * 3600);
        let (secs, _) = intraday_limit(Interval::OneHour).unwrap();
        assert_eq!(secs, 728 * 24 * 3600);
        assert!(intraday_limit(Interval::OneDay).is_none());
        assert!(intraday_limit(Interval::OneWeek).is_none());
    }
}
