//! Stock aggregate bar endpoints: OHLCV bars, daily summary, previous close.
#![allow(dead_code)]

use crate::Provider;
use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::chart::{Candle, Chart};
use crate::{Interval, TimeRange};
use chrono::Datelike;

use super::build_client;
use super::models::*;

/// Fetch aggregate bars (OHLCV) for a stock ticker over a date range.
///
/// # Arguments
///
/// * `ticker` - Stock ticker symbol (e.g., `"AAPL"`)
/// * `multiplier` - Size of the timespan multiplier (e.g., `1`, `5`, `15`)
/// * `timespan` - Timespan unit (e.g., `Timespan::Day`)
/// * `from` - Start date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `to` - End date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `params` - Optional parameters (adjusted, sort, limit)
pub async fn stock_aggregates(
    ticker: &str,
    multiplier: u32,
    timespan: Timespan,
    from: &str,
    to: &str,
    params: Option<AggregateParams>,
) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/aggs/ticker/{}/range/{}/{}/{}/{}",
        ticker,
        multiplier,
        timespan.as_str(),
        from,
        to
    );

    let mut query_params: Vec<(&str, String)> = Vec::new();
    if let Some(ref p) = params {
        if let Some(adjusted) = p.adjusted {
            query_params.push(("adjusted", adjusted.to_string()));
        }
        if let Some(sort) = p.sort {
            query_params.push(("sort", sort.as_str().to_string()));
        }
        if let Some(limit) = p.limit {
            query_params.push(("limit", limit.to_string()));
        }
    }

    let query_refs: Vec<(&str, &str)> =
        query_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

    client
        .get_as(&path, &query_refs, "aggregates", "aggregate response")
        .await
}

/// Helper: convert interval to (multiplier, timespan).
fn interval_to_polygon(interval: Interval) -> (u32, Timespan) {
    match interval {
        Interval::OneMinute => (1, Timespan::Minute),
        Interval::FiveMinutes => (5, Timespan::Minute),
        Interval::FifteenMinutes => (15, Timespan::Minute),
        Interval::ThirtyMinutes => (30, Timespan::Minute),
        Interval::OneHour => (1, Timespan::Hour),
        Interval::OneDay => (1, Timespan::Day),
        Interval::OneWeek => (1, Timespan::Week),
        Interval::OneMonth => (1, Timespan::Month),
        Interval::ThreeMonths => (3, Timespan::Month),
    }
}

/// Helper: convert TimeRange to (from, to) date strings.
fn range_to_dates(range: TimeRange) -> (String, String) {
    let now = chrono::Utc::now();
    let from = match range {
        TimeRange::OneDay => now - chrono::Duration::days(1),
        TimeRange::FiveDays => now - chrono::Duration::days(5),
        TimeRange::OneMonth => now - chrono::Duration::days(30),
        TimeRange::ThreeMonths => now - chrono::Duration::days(90),
        TimeRange::SixMonths => now - chrono::Duration::days(180),
        TimeRange::OneYear | TimeRange::TwoYears => now - chrono::Duration::days(365),
        TimeRange::YearToDate => chrono::Utc::now()
            .with_day(1)
            .and_then(|d| d.with_month(1))
            .unwrap_or(now - chrono::Duration::days(365)),
        TimeRange::FiveYears | TimeRange::TenYears => now - chrono::Duration::days(1825),
        TimeRange::Max => chrono::DateTime::from_timestamp(0, 0).unwrap_or(now),
    };
    (
        from.format("%Y-%m-%d").to_string(),
        now.format("%Y-%m-%d").to_string(),
    )
}

/// Helper: convert a Unix timestamp to "YYYY-MM-DD".
fn timestamp_to_date(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string())
}

/// Convert aggregate DTO results into canonical candles.
fn aggs_to_candles(aggs: AggregateResponseDTO) -> Vec<Candle> {
    aggs.results
        .into_iter()
        .flatten()
        .map(|r| Candle {
            timestamp: r.timestamp,
            open: r.open,
            high: r.high,
            low: r.low,
            close: r.close,
            volume: r.volume as i64,
            adj_close: None,
            provider_id: Some(Provider::Polygon),
        })
        .collect()
}

/// Fetch chart data (canonical) for a stock ticker by interval and time range.
pub async fn fetch_chart_response(
    symbol: &str,
    interval: Interval,
    range: TimeRange,
) -> Result<Chart> {
    let (from, to) = range_to_dates(range);
    let (mult, timespan) = interval_to_polygon(interval);
    let aggs = stock_aggregates(symbol, mult, timespan, &from, &to, None).await?;
    let candles = aggs_to_candles(aggs);
    Ok(Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles,
        interval: Some(interval),
        range: Some(range),
        provider_id: Some(Provider::Polygon),
    })
}

/// Fetch chart data (canonical) for a stock ticker by explicit date range.
pub async fn fetch_chart_range_response(
    symbol: &str,
    interval: Interval,
    start: i64,
    end: i64,
) -> Result<Chart> {
    let from = timestamp_to_date(start);
    let to = timestamp_to_date(end);
    let (mult, timespan) = interval_to_polygon(interval);
    let aggs = stock_aggregates(symbol, mult, timespan, &from, &to, None).await?;
    let candles = aggs_to_candles(aggs);
    Ok(Chart {
        symbol: symbol.to_string(),
        meta: Default::default(),
        candles,
        interval: None,
        range: None,
        provider_id: Some(Provider::Polygon),
    })
}

/// Fetch the previous day's OHLCV bar for a stock ticker.
///
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn stock_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", encode_path_segment(ticker));

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    client
        .get_as(&path, &params, "previous_close", "previous close response")
        .await
}

/// Fetch grouped daily bars for the entire stock market on a given date.
///
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn stock_grouped_daily(
    date: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/aggs/grouped/locale/us/market/stocks/{}",
        encode_path_segment(date)
    );

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    client
        .get_as(&path, &params, "grouped_daily", "grouped daily response")
        .await
}

/// Fetch daily open/close for a stock ticker on a specific date.
///
/// * `ticker` - Stock ticker symbol (e.g., `"AAPL"`)
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn stock_daily_open_close(
    ticker: &str,
    date: &str,
    adjusted: Option<bool>,
) -> Result<DailyOpenCloseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/open-close/{}/{}",
        encode_path_segment(ticker),
        encode_path_segment(date)
    );

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    client
        .get_as(
            &path,
            &params,
            "daily_open_close",
            "daily open/close response",
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stock_aggregates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "AAPL",
                    "status": "OK",
                    "adjusted": true,
                    "queryCount": 1,
                    "resultsCount": 2,
                    "request_id": "abc123",
                    "results": [
                        { "o": 185.09, "h": 187.01, "l": 184.35, "c": 186.19, "v": 65076600.0, "vw": 185.87, "t": 1704067200000_i64, "n": 823456 },
                        { "o": 186.06, "h": 186.74, "l": 185.19, "c": 185.59, "v": 40434100.0, "vw": 185.92, "t": 1704153600000_i64, "n": 612345 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("AAPL"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].open - 185.09).abs() < 0.01);
        assert!((results[0].close - 186.19).abs() < 0.01);
        assert_eq!(results[0].timestamp, 1704067200000);
    }

    #[tokio::test]
    async fn test_stock_previous_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/ticker/MSFT/prev")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "MSFT",
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "results": [
                        { "o": 380.0, "h": 385.0, "l": 378.0, "c": 383.5, "v": 25000000.0, "t": 1704067200000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/ticker/MSFT/prev", &[])
            .await
            .unwrap();

        let resp: AggregateResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("MSFT"));
        let bar = &resp.results.unwrap()[0];
        assert!((bar.close - 383.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_daily_open_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/open-close/AAPL/2024-01-15")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "from": "2024-01-15",
                    "symbol": "AAPL",
                    "open": 185.09,
                    "high": 187.01,
                    "low": 184.35,
                    "close": 186.19,
                    "volume": 65076600.0,
                    "afterHours": 186.50,
                    "preMarket": 184.80
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/open-close/AAPL/2024-01-15", &[])
            .await
            .unwrap();

        let resp: DailyOpenCloseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert!((resp.open.unwrap() - 185.09).abs() < 0.01);
        assert!((resp.after_hours.unwrap() - 186.50).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_polygon_rate_limit_returns_rate_limited_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(429)
            .with_body("{}")
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/v2/aggs/ticker/AAPL/prev", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::RateLimited { .. })
        ));
    }

    #[tokio::test]
    async fn test_polygon_401_returns_authentication_failed() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(401)
            .with_body("{}")
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/v2/aggs/ticker/AAPL/prev", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::AuthenticationFailed { .. })
        ));
    }

    #[tokio::test]
    async fn test_polygon_body_error_status_returns_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"status":"ERROR","error":"bad request"}"#)
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/v2/aggs/ticker/AAPL/prev", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::ExternalApiError { .. })
        ));
    }

    #[tokio::test]
    async fn test_polygon_body_not_found_returns_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"{"status":"NOT_FOUND","message":"ticker not found"}"#)
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/v2/aggs/ticker/XYZ/prev", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::SymbolNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_polygon_500_returns_server_error() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .with_body("{}")
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result = client.get_raw("/v2/aggs/ticker/AAPL/prev", &[]).await;

        assert!(matches!(
            result,
            Err(crate::error::FinanceError::ServerError { .. })
        ));
    }
}
