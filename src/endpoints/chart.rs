use super::urls::api;
/// Chart data endpoint
///
/// Fetches historical price and volume data for a symbol.
use crate::client::YahooClient;
use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use tracing::info;

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
        ("events", "div|split|capitalGain"),
    ];
    let response = client.request_with_params(&url, &params).await?;

    Ok(response.json().await?)
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
        ("events", "div|split|capitalGain"),
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
        // Daily and coarser have no restrictions — max, 10y, etc. all use range param
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
