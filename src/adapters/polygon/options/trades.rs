//! Options trade and quote endpoints: last trade, historical trades, historical quotes.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch the most recent trade for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
pub async fn options_last_trade(ticker: &str) -> Result<LastTradeResponse> {
    let client = build_client()?;
    let path = format!("/v2/last/trade/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "options_last_trade".to_string(),
        context: format!("Failed to parse options last trade response: {e}"),
    })
}

/// Fetch historical trades for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn options_trades(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Trade>> {
    let client = build_client()?;
    let path = format!("/v3/trades/{}", ticker);
    client.get(&path, params).await
}

/// Fetch historical quotes for an options contract.
///
/// * `ticker` - Options ticker symbol with `O:` prefix (e.g., `"O:AAPL250117C00150000"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn options_quotes(
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
    async fn test_options_last_trade_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/last/trade/O:AAPL250117C00150000")
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
                    "results": {
                        "T": "O:AAPL250117C00150000",
                        "conditions": [209],
                        "exchange": 4,
                        "id": "optiontrade1",
                        "price": 5.30,
                        "size": 10.0,
                        "sip_timestamp": 1705363200000000000_i64,
                        "participant_timestamp": 1705363200000000000_i64,
                        "sequence_number": 54321,
                        "tape": 3
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/last/trade/O:AAPL250117C00150000", &[])
            .await
            .unwrap();
        let resp: LastTradeResponse = serde_json::from_value(json).unwrap();

        assert_eq!(resp.status.as_deref(), Some("OK"));
        let trade = resp.results.unwrap();
        assert_eq!(trade.ticker.as_deref(), Some("O:AAPL250117C00150000"));
        assert!((trade.price.unwrap() - 5.30).abs() < 0.01);
        assert_eq!(trade.size.unwrap() as u64, 10);
    }

    #[tokio::test]
    async fn test_options_trades_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/trades/O:AAPL250117C00150000")
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
                        { "price": 5.30, "size": 10.0, "exchange": 4, "sip_timestamp": 1705363200000000000_i64 },
                        { "price": 5.35, "size": 5.0, "exchange": 11, "sip_timestamp": 1705363200100000000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Trade> = client
            .get("/v3/trades/O:AAPL250117C00150000", &[])
            .await
            .unwrap();
        let trades = resp.results.unwrap();
        assert_eq!(trades.len(), 2);
        assert!((trades[0].price.unwrap() - 5.30).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_options_quotes_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/quotes/O:AAPL250117C00150000")
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
                            "ask_price": 5.40,
                            "ask_size": 10.0,
                            "bid_price": 5.20,
                            "bid_size": 15.0,
                            "sip_timestamp": 1705363200000000000_i64,
                            "sequence_number": 67890,
                            "tape": 3
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Quote> = client
            .get("/v3/quotes/O:AAPL250117C00150000", &[])
            .await
            .unwrap();
        let quotes = resp.results.unwrap();
        assert_eq!(quotes.len(), 1);
        assert!((quotes[0].ask_price.unwrap() - 5.40).abs() < 0.01);
        assert!((quotes[0].bid_price.unwrap() - 5.20).abs() < 0.01);
    }
}
