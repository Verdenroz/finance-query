//! Index technical indicator endpoints: SMA, EMA, MACD, RSI.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch SMA (Simple Moving Average) for an index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn index_sma(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "sma", params).await
}

/// Fetch EMA (Exponential Moving Average) for an index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn index_ema(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "ema", params).await
}

/// Fetch MACD for an index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
/// * `params` - Optional query params: `short_window`, `long_window`, `signal_window`, `timespan`, `series_type`, `order`, `limit`
pub async fn index_macd(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
    fetch_indicator(ticker, "macd", params).await
}

/// Fetch RSI (Relative Strength Index) for an index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
/// * `params` - Optional query params: `window`, `timespan`, `series_type`, `order`, `limit`
pub async fn index_rsi(ticker: &str, params: &[(&str, &str)]) -> Result<IndicatorResponse> {
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
        context: format!("Failed to parse index {indicator} response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index_rsi_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/indicators/rsi/I:SPX")
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
                            "url": "https://api.polygon.io/v2/aggs/ticker/I:SPX/range/1/day/...",
                            "aggregates": [
                                { "o": 4750.0, "h": 4780.0, "l": 4740.0, "c": 4770.0, "v": 3500000000.0, "t": 1704067200000_i64 }
                            ]
                        },
                        "values": [
                            { "timestamp": 1704067200000_i64, "value": 65.3 }
                        ]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/indicators/rsi/I:SPX", &[])
            .await
            .unwrap();

        let resp: IndicatorResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let values = resp.results.unwrap().values.unwrap();
        assert_eq!(values.len(), 1);
        assert!((values[0].value.unwrap() - 65.3).abs() < 0.01);
    }
}
