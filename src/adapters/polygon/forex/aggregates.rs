//! Forex aggregate bar endpoints: OHLCV bars, previous close, grouped daily.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch aggregate bars (OHLCV) for a forex ticker over a date range.
///
/// # Arguments
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `multiplier` - Size of the timespan multiplier (e.g., `1`, `5`, `15`)
/// * `timespan` - Timespan unit (e.g., `Timespan::Day`)
/// * `from` - Start date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `to` - End date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `params` - Optional parameters (adjusted, sort, limit)
pub async fn forex_aggregates(
    ticker: &str,
    multiplier: u32,
    timespan: Timespan,
    from: &str,
    to: &str,
    params: Option<AggregateParams>,
) -> Result<AggregateResponse> {
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

    let json = client.get_raw(&path, &query_refs).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "forex_aggregates".to_string(),
        context: format!("Failed to parse forex aggregate response: {e}"),
    })
}

/// Fetch the previous day's OHLCV bar for a forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn forex_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", ticker);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "forex_previous_close".to_string(),
        context: format!("Failed to parse forex previous close response: {e}"),
    })
}

/// Fetch grouped daily bars for the entire forex market on a given date.
///
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn forex_grouped_daily(date: &str, adjusted: Option<bool>) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/grouped/locale/global/market/fx/{}", date);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "forex_grouped_daily".to_string(),
        context: format!("Failed to parse forex grouped daily response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_forex_aggregates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/aggs/ticker/C:EURUSD/range/1/day/2024-01-01/2024-01-31",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "C:EURUSD",
                    "status": "OK",
                    "adjusted": true,
                    "queryCount": 1,
                    "resultsCount": 2,
                    "request_id": "abc123",
                    "results": [
                        { "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0, "vw": 1.1060, "t": 1704067200000_i64, "n": 1000 },
                        { "o": 1.1080, "h": 1.1120, "l": 1.1040, "c": 1.1095, "v": 45000.0, "vw": 1.1075, "t": 1704153600000_i64, "n": 900 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/C:EURUSD/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("C:EURUSD"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].open - 1.1050).abs() < 0.0001);
        assert!((results[0].close - 1.1080).abs() < 0.0001);
        assert_eq!(results[0].timestamp, 1704067200000);
    }

    #[tokio::test]
    async fn test_forex_previous_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/ticker/C:GBPUSD/prev")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "C:GBPUSD",
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "results": [
                        { "o": 1.2700, "h": 1.2750, "l": 1.2680, "c": 1.2730, "v": 30000.0, "t": 1704067200000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/ticker/C:GBPUSD/prev", &[])
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("C:GBPUSD"));
        let bar = &resp.results.unwrap()[0];
        assert!((bar.close - 1.2730).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_forex_grouped_daily_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/grouped/locale/global/market/fx/2024-01-15")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "queryCount": 1,
                    "results": [
                        { "T": "C:EURUSD", "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0, "t": 1705276800000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/grouped/locale/global/market/fx/2024-01-15", &[])
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        assert!(!results.is_empty());
        assert!((results[0].open - 1.1050).abs() < 0.0001);
    }
}
