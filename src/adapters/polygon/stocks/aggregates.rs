//! Stock aggregate bar endpoints: OHLCV bars, daily summary, previous close.

use crate::error::Result;

use super::super::build_client;
use super::super::models::*;

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
    serde_json::from_value(json).map_err(|e| crate::error::FinanceError::ResponseStructureError {
        field: "aggregates".to_string(),
        context: format!("Failed to parse aggregate response: {e}"),
    })
}

/// Fetch the previous day's OHLCV bar for a stock ticker.
///
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn stock_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", ticker);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| crate::error::FinanceError::ResponseStructureError {
        field: "previous_close".to_string(),
        context: format!("Failed to parse previous close response: {e}"),
    })
}

/// Fetch grouped daily bars for the entire stock market on a given date.
///
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
pub async fn stock_grouped_daily(
    date: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponse> {
    let client = build_client()?;
    let path = format!("/v2/aggs/grouped/locale/us/market/stocks/{}", date);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| crate::error::FinanceError::ResponseStructureError {
        field: "grouped_daily".to_string(),
        context: format!("Failed to parse grouped daily response: {e}"),
    })
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
) -> Result<DailyOpenClose> {
    let client = build_client()?;
    let path = format!("/v1/open-close/{}/{}", ticker, date);

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| crate::error::FinanceError::ResponseStructureError {
        field: "daily_open_close".to_string(),
        context: format!("Failed to parse daily open/close response: {e}"),
    })
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

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/aggs/ticker/AAPL/range/1/day/2024-01-01/2024-01-31",
                &[],
            )
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
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

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/aggs/ticker/MSFT/prev", &[])
            .await
            .unwrap();

        let resp: AggregateResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("MSFT"));
        let bar = &resp.results.unwrap()[0];
        assert!((bar.close - 383.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_daily_open_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/open-close/AAPL/2024-01-15")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
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

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/open-close/AAPL/2024-01-15", &[])
            .await
            .unwrap();

        let resp: DailyOpenClose = serde_json::from_value(json).unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert!((resp.open.unwrap() - 185.09).abs() < 0.01);
        assert!((resp.after_hours.unwrap() - 186.50).abs() < 0.01);
    }
}
