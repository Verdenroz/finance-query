//! Futures trade and quote endpoints: historical trades, historical quotes.

use crate::error::Result;

use super::super::build_client;
use super::super::models::*;

/// Fetch historical trades for a futures ticker.
///
/// * `ticker` - Futures ticker symbol (e.g., `"ESZ4"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn futures_trades(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Trade>> {
    let client = build_client()?;
    let path = format!("/v3/trades/{}", ticker);
    client.get(&path, params).await
}

/// Fetch historical quotes for a futures ticker.
///
/// * `ticker` - Futures ticker symbol (e.g., `"ESZ4"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn futures_quotes(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Quote>> {
    let client = build_client()?;
    let path = format!("/v3/quotes/{}", ticker);
    client.get(&path, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_futures_trades_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/trades/ESZ4")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": [
                        { "price": 4770.0, "size": 10.0, "exchange": 1, "sip_timestamp": 1705363200000000000_i64 },
                        { "price": 4770.25, "size": 5.0, "exchange": 1, "sip_timestamp": 1705363200100000000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Trade> = client.get("/v3/trades/ESZ4", &[]).await.unwrap();
        let trades = resp.results.unwrap();
        assert_eq!(trades.len(), 2);
        assert!((trades[0].price.unwrap() - 4770.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_futures_quotes_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/quotes/ESZ4")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": [
                        {
                            "ask_price": 4770.50,
                            "ask_size": 20.0,
                            "bid_price": 4770.25,
                            "bid_size": 15.0,
                            "sip_timestamp": 1705363200000000000_i64,
                            "sequence_number": 12345
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Quote> = client.get("/v3/quotes/ESZ4", &[]).await.unwrap();
        let quotes = resp.results.unwrap();
        assert_eq!(quotes.len(), 1);
        assert!((quotes[0].ask_price.unwrap() - 4770.50).abs() < 0.01);
        assert!((quotes[0].bid_price.unwrap() - 4770.25).abs() < 0.01);
    }
}
