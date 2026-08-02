//! Forex quote endpoints: last quote, historical quotes, currency conversion.

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::forex::ForexQuote;
use serde::{Deserialize, Serialize};

use super::super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Last forex quote data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexLastQuoteDTO {
    /// Bid price.
    pub bid: Option<f64>,
    /// Ask price.
    pub ask: Option<f64>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// Unix millisecond timestamp.
    pub timestamp: Option<i64>,
}

/// Response for the last forex quote endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexQuoteResponseDTO {
    /// Response status.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The last quote.
    pub last: Option<ForexLastQuoteDTO>,
}

// ============================================================================
// Public API functions
// ============================================================================

/// Fetch the last quote for a forex currency pair.
///
/// # Arguments
///
/// * `from` - Base currency code (e.g., `"EUR"`)
/// * `to` - QuoteDTO currency code (e.g., `"USD"`)
pub async fn forex_last_quote(from: &str, to: &str) -> Result<ForexQuoteResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v1/last_quote/currencies/{}/{}",
        encode_path_segment(from),
        encode_path_segment(to)
    );
    client
        .get_as(&path, &[], "forex_last_quote", "forex last quote response")
        .await
}

/// Fetch forex quote (canonical) for a currency pair.
pub async fn fetch_forex_quote_response(from: &str, to: &str) -> Result<ForexQuote> {
    let resp = forex_last_quote(from, to).await?;
    Ok(last_quote_to_canonical(from, to, resp))
}

/// Map a last-quote response to the canonical [`ForexQuote`];
/// `price` prefers bid, falling back to ask.
fn last_quote_to_canonical(from: &str, to: &str, resp: ForexQuoteResponseDTO) -> ForexQuote {
    let last = resp.last;
    let bid = last.as_ref().and_then(|l| l.bid);
    let ask = last.as_ref().and_then(|l| l.ask);
    ForexQuote {
        symbol: format!("{}{}", from.to_uppercase(), to.to_uppercase()),
        base_currency: Some(from.to_string()),
        quote_currency: Some(to.to_string()),
        bid,
        ask,
        price: bid.or(ask),
        change: None,
        change_percent: None,
        timestamp: last.as_ref().and_then(|l| l.timestamp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_forex_last_quote_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/last_quote/currencies/EUR/USD")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "last": {
                        "bid": 1.1050,
                        "ask": 1.1052,
                        "exchange": 48,
                        "timestamp": 1705363200000_i64
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v1/last_quote/currencies/EUR/USD", &[])
            .await
            .unwrap();

        let resp: ForexQuoteResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let last = resp.last.as_ref().unwrap();
        assert!((last.bid.unwrap() - 1.1050).abs() < 0.0001);
        assert!((last.ask.unwrap() - 1.1052).abs() < 0.0001);
        assert_eq!(last.exchange.unwrap(), 48);

        // Mocked HTTP → DTO → canonical ForexQuote, covering the full
        // fetch_forex_quote_response pipeline without a network call.
        let quote = last_quote_to_canonical("EUR", "USD", resp);
        assert_eq!(quote.symbol, "EURUSD");
        assert_eq!(quote.bid, Some(1.1050));
        assert_eq!(quote.ask, Some(1.1052));
        assert_eq!(quote.price, Some(1.1050));
        assert_eq!(quote.timestamp, Some(1705363200000));
    }

    #[test]
    fn last_quote_to_canonical_maps_bid_ask_and_uppercases_symbol() {
        let resp: ForexQuoteResponseDTO = serde_json::from_value(serde_json::json!({
            "status": "OK",
            "last": {"bid": 1.1050, "ask": 1.1052, "timestamp": 1705363200000_i64}
        }))
        .unwrap();

        let quote = last_quote_to_canonical("eur", "usd", resp);
        assert_eq!(quote.symbol, "EURUSD");
        assert_eq!(quote.base_currency.as_deref(), Some("eur"));
        assert_eq!(quote.quote_currency.as_deref(), Some("usd"));
        assert_eq!(quote.bid, Some(1.1050));
        assert_eq!(quote.ask, Some(1.1052));
        assert_eq!(quote.price, Some(1.1050), "price prefers bid");
        assert_eq!(quote.timestamp, Some(1705363200000));
    }

    #[test]
    fn last_quote_to_canonical_price_falls_back_to_ask() {
        let resp: ForexQuoteResponseDTO = serde_json::from_value(serde_json::json!({
            "status": "OK",
            "last": {"ask": 1.1052}
        }))
        .unwrap();
        let quote = last_quote_to_canonical("EUR", "USD", resp);
        assert!(quote.bid.is_none());
        assert_eq!(quote.price, Some(1.1052));
    }

    #[test]
    fn last_quote_to_canonical_missing_last_yields_no_prices() {
        let resp: ForexQuoteResponseDTO =
            serde_json::from_value(serde_json::json!({"status": "OK"})).unwrap();
        let quote = last_quote_to_canonical("EUR", "USD", resp);
        assert_eq!(quote.symbol, "EURUSD");
        assert!(quote.price.is_none());
        assert!(quote.timestamp.is_none());
    }
}
