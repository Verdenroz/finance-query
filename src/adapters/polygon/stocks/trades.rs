//! Stock trade and quote endpoints: last trade, historical trades, last quote, historical quotes.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch the most recent trade for a stock ticker.
pub async fn stock_last_trade(ticker: &str) -> Result<LastTradeResponse> {
    let client = build_client()?;
    let path = format!("/v2/last/trade/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "last_trade".to_string(),
        context: format!("Failed to parse last trade response: {e}"),
    })
}

/// Fetch historical trades for a stock ticker.
///
/// * `ticker` - Stock ticker symbol
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn stock_trades(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Trade>> {
    let client = build_client()?;
    let path = format!("/v3/trades/{}", ticker);
    client.get(&path, params).await
}

/// Fetch the most recent NBBO quote for a stock ticker.
pub async fn stock_last_quote(ticker: &str) -> Result<LastQuoteResponse> {
    let client = build_client()?;
    let path = format!("/v2/last/nbbo/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "last_quote".to_string(),
        context: format!("Failed to parse last quote response: {e}"),
    })
}

/// Fetch historical NBBO quotes for a stock ticker.
///
/// * `ticker` - Stock ticker symbol
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn stock_quotes(
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
    async fn test_last_trade_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/last/trade/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": {
                        "T": "AAPL",
                        "conditions": [12, 37],
                        "exchange": 4,
                        "id": "trade1",
                        "price": 186.19,
                        "size": 100.0,
                        "sip_timestamp": 1705363200000000000_i64,
                        "participant_timestamp": 1705363200000000000_i64,
                        "sequence_number": 12345,
                        "tape": 3
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client.get_raw("/v2/last/trade/AAPL", &[]).await.unwrap();
        let resp: LastTradeResponse = serde_json::from_value(json).unwrap();

        assert_eq!(resp.status.as_deref(), Some("OK"));
        let trade = resp.results.unwrap();
        assert_eq!(trade.ticker.as_deref(), Some("AAPL"));
        assert!((trade.price.unwrap() - 186.19).abs() < 0.01);
        assert_eq!(trade.size.unwrap() as u64, 100);
    }

    #[tokio::test]
    async fn test_last_quote_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/last/nbbo/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "results": {
                        "ask_exchange": 11,
                        "ask_price": 186.25,
                        "ask_size": 3.0,
                        "bid_exchange": 19,
                        "bid_price": 186.18,
                        "bid_size": 2.0,
                        "conditions": [1],
                        "sip_timestamp": 1705363200000000000_i64,
                        "participant_timestamp": 1705363200000000000_i64,
                        "sequence_number": 67890,
                        "tape": 3
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client.get_raw("/v2/last/nbbo/AAPL", &[]).await.unwrap();
        let resp: LastQuoteResponse = serde_json::from_value(json).unwrap();

        let quote = resp.results.unwrap();
        assert!((quote.ask_price.unwrap() - 186.25).abs() < 0.01);
        assert!((quote.bid_price.unwrap() - 186.18).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_historical_trades_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/trades/AAPL")
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
                        { "price": 186.19, "size": 100.0, "exchange": 4, "sip_timestamp": 1705363200000000000_i64 },
                        { "price": 186.20, "size": 50.0, "exchange": 11, "sip_timestamp": 1705363200100000000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Trade> = client.get("/v3/trades/AAPL", &[]).await.unwrap();
        let trades = resp.results.unwrap();
        assert_eq!(trades.len(), 2);
        assert!((trades[0].price.unwrap() - 186.19).abs() < 0.01);
    }
}
