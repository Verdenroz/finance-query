//! Options aggregate bar endpoints: OHLCV bars, previous close, daily open/close.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch aggregate bars (OHLCV) for an options contract over a date range.
///
/// # Arguments
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `multiplier` - Size of the timespan multiplier (e.g., `1`, `5`, `15`)
/// * `timespan` - Timespan unit (e.g., `Timespan::Day`)
/// * `from` - Start date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `to` - End date as `"YYYY-MM-DD"` or millisecond timestamp string
/// * `params` - Optional parameters (adjusted, sort, limit)
pub async fn options_aggregates(
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

    let query_refs: Vec<(&str, &str)> = query_params
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();

    let json = client.get_raw(&path, &query_refs).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_aggregates".to_string(),
        context: format!("Failed to parse options aggregate response: {e}"),
    })
}

/// Fetch the previous day's OHLCV bar for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn options_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", ticker);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_previous_close".to_string(),
        context: format!("Failed to parse options previous close response: {e}"),
    })
}

/// Fetch daily open/close for an options contract on a specific date.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn options_daily_open_close(
    ticker: &str,
    date: &str,
    adjusted: Option<bool>,
) -> Result<DailyOpenClose> {
    let client = build_client()?;
    let path = format!("/v1/open-close/{}/{}", ticker, date);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_daily_open_close".to_string(),
        context: format!("Failed to parse options daily open/close response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_options_aggregates_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/aggs/ticker/O:AAPL250117C00150000/range/1/day/2024-01-01/2024-01-31",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "O:AAPL250117C00150000",
                    "status": "OK",
                    "adjusted": true,
                    "queryCount": 1,
                    "resultsCount": 2,
                    "request_id": "abc123",
                    "results": [
                        { "o": 5.10, "h": 5.50, "l": 4.90, "c": 5.30, "v": 1200.0, "vw": 5.20, "t": 1704067200000_i64, "n": 450 },
                        { "o": 5.35, "h": 5.60, "l": 5.10, "c": 5.45, "v": 800.0, "vw": 5.35, "t": 1704153600000_i64, "n": 320 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/O:AAPL250117C00150000/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("O:AAPL250117C00150000"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 2);
        assert!((results[0].open - 5.10).abs() < 0.01);
        assert!((results[0].close - 5.30).abs() < 0.01);
        assert_eq!(results[0].timestamp, 1704067200000);
    }

    #[tokio::test]
    async fn test_options_previous_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/aggs/ticker/O:AAPL250117C00150000/prev")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "ticker": "O:AAPL250117C00150000",
                    "status": "OK",
                    "adjusted": true,
                    "resultsCount": 1,
                    "results": [
                        { "o": 5.10, "h": 5.50, "l": 4.90, "c": 5.30, "v": 1200.0, "t": 1704067200000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/ticker/O:AAPL250117C00150000/prev", &[])
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("O:AAPL250117C00150000"));
        let bar = &resp.results.unwrap()[0];
        assert!((bar.close - 5.30).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_options_daily_open_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/open-close/O:AAPL250117C00150000/2024-01-15")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "from": "2024-01-15",
                    "symbol": "O:AAPL250117C00150000",
                    "open": 5.10,
                    "high": 5.50,
                    "low": 4.90,
                    "close": 5.30,
                    "volume": 1200.0,
                    "afterHours": 5.35,
                    "preMarket": 5.05
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/open-close/O:AAPL250117C00150000/2024-01-15", &[])
            .await
            .unwrap();

        let resp: DailyOpenClose = serde_json::from_value(json).unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("O:AAPL250117C00150000"));
        assert!((resp.open.unwrap() - 5.10).abs() < 0.01);
        assert!((resp.after_hours.unwrap() - 5.35).abs() < 0.01);
    }
}
