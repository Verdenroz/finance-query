//! Crypto trade endpoints: last trade, historical trades.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Last trade data for a crypto pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoLastTrade {
    /// Price of the last trade.
    pub price: Option<f64>,
    /// Size of the last trade.
    pub size: Option<f64>,
    /// Exchange where the trade occurred.
    pub exchange: Option<i32>,
    /// Trade conditions.
    pub conditions: Option<Vec<i32>>,
    /// Timestamp of the trade.
    pub timestamp: Option<i64>,
}

/// Response wrapper for crypto last trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CryptoLastTradeResponse {
    /// The last trade data.
    pub last: Option<CryptoLastTrade>,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Symbol of the crypto pair.
    pub symbol: Option<String>,
}

/// Fetch the most recent trade for a crypto pair.
///
/// The ticker should be in the format `"X:BTCUSD"`. The `from` and `to` components
/// are extracted automatically (e.g., `BTC` and `USD`).
///
/// * `from` - The base currency (e.g., `"BTC"`)
/// * `to` - The quote currency (e.g., `"USD"`)
pub async fn crypto_last_trade(from: &str, to: &str) -> Result<CryptoLastTradeResponse> {
    let client = build_client()?;
    let path = format!("/v1/last/crypto/{}/{}", from, to);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_last_trade".to_string(),
        context: format!("Failed to parse crypto last trade response: {e}"),
    })
}

/// Fetch historical trades for a crypto ticker.
///
/// * `ticker` - Crypto ticker symbol with `X:` prefix (e.g., `"X:BTCUSD"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn crypto_trades(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Trade>> {
    let client = build_client()?;
    let path = format!("/v3/trades/{}", ticker);
    client.get(&path, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_last_trade_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/last/crypto/BTC/USD")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "symbol": "X:BTCUSD",
                    "last": {
                        "price": 43100.50,
                        "size": 0.5,
                        "exchange": 2,
                        "conditions": [1],
                        "timestamp": 1705363200000_i64
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/last/crypto/BTC/USD", &[])
            .await
            .unwrap();

        let resp: CryptoLastTradeResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        assert_eq!(resp.symbol.as_deref(), Some("X:BTCUSD"));
        let last = resp.last.unwrap();
        assert!((last.price.unwrap() - 43100.50).abs() < 0.01);
        assert!((last.size.unwrap() - 0.5).abs() < 0.01);
    }
}
