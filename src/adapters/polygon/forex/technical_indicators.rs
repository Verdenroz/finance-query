//! Forex technical indicator endpoints: SMA, EMA, MACD, RSI.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch SMA (Simple Moving Average) for a forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn forex_sma(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "sma", params).await
}

/// Fetch EMA (Exponential Moving Average) for a forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn forex_ema(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "ema", params).await
}

/// Fetch MACD for a forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `params` - Optional query params: `short_window`, `long_window`, `signal_window`, `timespan`, `series_type`, `order`, `limit`
pub async fn forex_macd(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "macd", params).await
}

/// Fetch RSI (Relative Strength Index) for a forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn forex_rsi(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
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
    async fn test_forex_sma_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/sma/C:EURUSD")
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
                            "url": "https://api.polygon.io/v2/aggs/ticker/C:EURUSD/range/1/day/2024-01-01/2024-01-31",
                            "aggregates": [
                                { "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0, "t": 1704067200000_i64 }
                            ]
                        },
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 1.1065 },
                            { "timestamp": 1704153600000_i64, "value": 1.1072 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/sma/C:EURUSD", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        let values = results.values.unwrap();
        assert_eq!(values.len(), 2);
        assert!((values[0].value.unwrap() - 1.1065).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_forex_ema_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/ema/C:GBPUSD")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "def456",
                    "results": {
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 1.2715 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/ema/C:GBPUSD", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 1);
        assert!((values[0].value.unwrap() - 1.2715).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_forex_macd_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/macd/C:USDJPY")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "ghi789",
                    "results": {
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 0.25, "signal": 0.18, "histogram": 0.07 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/macd/C:USDJPY", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 1);
        assert!((values[0].value.unwrap() - 0.25).abs() < 0.01);
        assert!((values[0].signal.unwrap() - 0.18).abs() < 0.01);
        assert!((values[0].histogram.unwrap() - 0.07).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_forex_rsi_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/rsi/C:EURUSD")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "jkl012",
                    "results": {
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 62.5 },
                            { "timestamp": 1704153600000_i64, "value": 58.3 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/rsi/C:EURUSD", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 2);
        assert!((values[0].value.unwrap() - 62.5).abs() < 0.1);
        assert!((values[1].value.unwrap() - 58.3).abs() < 0.1);
    }
}
