//! Options aggregate bar endpoints: previous close, daily open/close.

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::super::build_client;
use super::super::models::*;

/// Fetch the previous day's OHLCV bar for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `adjusted` - Whether results are adjusted for splits (default: true)
#[allow(dead_code)] // unrouted: no capability route for a standalone previous-close lookup yet
pub async fn options_previous_close(
    ticker: &str,
    adjusted: Option<bool>,
) -> Result<AggregateResponseDTO> {
    let client = build_client()?;
    let path = format!("/v2/aggs/ticker/{}/prev", encode_path_segment(ticker));

    let adj_str = adjusted.unwrap_or(true).to_string();
    let params = [("adjusted", adj_str.as_str())];

    client
        .get_as(
            &path,
            &params,
            "options_previous_close",
            "options previous close response",
        )
        .await
}

/// Fetch daily open/close for an options contract on a specific date.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `date` - Date as `"YYYY-MM-DD"`
/// * `adjusted` - Whether results are adjusted for splits (default: true)
#[allow(dead_code)] // unrouted: no capability route for a standalone daily-open-close lookup yet
pub async fn options_daily_open_close(
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
            "options_daily_open_close",
            "options daily open/close response",
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let resp: AggregateResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.ticker.as_deref(), Some("O:AAPL250117C00150000"));
        let bar = &resp.results.unwrap()[0];
        assert!((bar.close - 5.30).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_options_daily_open_close_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/open-close/O:AAPL250117C00150000/2024-01-15")
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

        let resp: DailyOpenCloseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("O:AAPL250117C00150000"));
        assert!((resp.open.unwrap() - 5.10).abs() < 0.01);
        assert!((resp.after_hours.unwrap() - 5.35).abs() < 0.01);
    }
}
