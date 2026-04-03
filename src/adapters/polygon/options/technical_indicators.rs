//! Options technical indicator endpoints: SMA, EMA, MACD, RSI.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch SMA (Simple Moving Average) for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Query params such as `timestamp`, `timespan`, `window`, `series_type`, `order`, `limit`
pub async fn options_sma(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "sma", params).await
}

/// Fetch EMA (Exponential Moving Average) for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Query params such as `timestamp`, `timespan`, `window`, `series_type`, `order`, `limit`
pub async fn options_ema(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "ema", params).await
}

/// Fetch MACD for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Query params such as `timestamp`, `timespan`, `short_window`, `long_window`,
///   `signal_window`, `series_type`, `order`, `limit`
pub async fn options_macd(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "macd", params).await
}

/// Fetch RSI (Relative Strength Index) for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Query params such as `timestamp`, `timespan`, `window`, `series_type`, `order`, `limit`
pub async fn options_rsi(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "rsi", params).await
}

async fn fetch_indicator(
    ticker: &str,
    indicator: &str,
    params: &[(&str, &str)],
) -> Result<IndicatorResponse> {
    let client = build_client()?;
    let path = format!("/v1/indicators/{}/{}", indicator, ticker);
    let json = client.get_raw(&path, params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: indicator.to_string(),
        context: format!("Failed to parse {indicator} response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_options_sma_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/sma/O:AAPL250117C00150000")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": {
                        "underlying": {
                            "url": "https://api.polygon.io/v2/aggs/ticker/O:AAPL250117C00150000/range/1/day/2024-01-01/2024-01-31",
                            "aggregates": [
                                { "o": 5.10, "h": 5.50, "l": 4.90, "c": 5.30, "v": 1200.0, "t": 1704067200000_i64, "n": 450 },
                                { "o": 5.35, "h": 5.60, "l": 5.10, "c": 5.45, "v": 800.0, "t": 1704153600000_i64, "n": 320 }
                            ]
                        },
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 5.20 },
                            { "timestamp": 1704153600000_i64, "value": 5.35 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/sma/O:AAPL250117C00150000", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        let values = results.values.unwrap();
        assert_eq!(values.len(), 2);
        assert!((values[0].value.unwrap() - 5.20).abs() < 0.01);
        assert!((values[1].value.unwrap() - 5.35).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_options_macd_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/macd/O:AAPL250117C00150000")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": {
                        "underlying": {
                            "url": "https://api.polygon.io/v2/aggs/ticker/O:AAPL250117C00150000/range/1/day/2024-01-01/2024-01-31"
                        },
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 0.15, "signal": 0.12, "histogram": 0.03 },
                            { "timestamp": 1704153600000_i64, "value": 0.18, "signal": 0.14, "histogram": 0.04 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/macd/O:AAPL250117C00150000", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 2);
        assert!((values[0].signal.unwrap() - 0.12).abs() < 0.01);
        assert!((values[0].histogram.unwrap() - 0.03).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_options_rsi_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/rsi/O:AAPL250117C00150000")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": {
                        "underlying": {
                            "url": "https://api.polygon.io/v2/aggs/ticker/O:AAPL250117C00150000/range/1/day/2024-01-01/2024-01-31"
                        },
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 65.4 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/rsi/O:AAPL250117C00150000", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 1);
        assert!((values[0].value.unwrap() - 65.4).abs() < 0.1);
    }
}
