//! Forex quote endpoints: last quote, historical quotes, currency conversion.

use crate::error::{FinanceError, Result};
use serde::{Deserialize, Serialize};

use super::super::build_client;
use super::super::models::*;

// ============================================================================
// Response types
// ============================================================================

/// Last forex quote data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForexLastQuote {
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
pub struct ForexQuoteResponse {
    /// Response status.
    pub status: Option<String>,
    /// Request ID.
    pub request_id: Option<String>,
    /// The last quote.
    pub last: Option<ForexLastQuote>,
}

/// Currency conversion last price data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConversionLast {
    /// Bid price.
    pub bid: Option<f64>,
    /// Ask price.
    pub ask: Option<f64>,
    /// Exchange ID.
    pub exchange: Option<i32>,
    /// Unix millisecond timestamp.
    pub timestamp: Option<i64>,
}

/// Response for the currency conversion endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CurrencyConversion {
    /// Response status.
    pub status: Option<String>,
    /// The converted amount.
    pub converted: Option<f64>,
    /// From currency code.
    pub from: Option<String>,
    /// To currency code.
    pub to: Option<String>,
    /// Initial amount before conversion.
    #[serde(rename = "initialAmount")]
    pub initial_amount: Option<f64>,
    /// Last quote used for conversion.
    pub last: Option<ConversionLast>,
}

// ============================================================================
// Public API functions
// ============================================================================

/// Fetch the last quote for a forex currency pair.
///
/// # Arguments
///
/// * `from` - Base currency code (e.g., `"EUR"`)
/// * `to` - Quote currency code (e.g., `"USD"`)
pub async fn forex_last_quote(from: &str, to: &str) -> Result<ForexQuoteResponse> {
    let client = build_client()?;
    let path = format!("/v1/last_quote/currencies/{}/{}", from, to);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "forex_last_quote".to_string(),
        context: format!("Failed to parse forex last quote response: {e}"),
    })
}

/// Fetch historical quotes for a forex ticker.
///
/// # Arguments
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
/// * `params` - Optional query params: `timestamp`, `order`, `limit`, `sort`
pub async fn forex_quotes(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponse<Quote>> {
    let client = build_client()?;
    let path = format!("/v3/quotes/{}", ticker);
    client.get(&path, params).await
}

/// Convert a currency amount from one currency to another.
///
/// # Arguments
///
/// * `from` - Base currency code (e.g., `"EUR"`)
/// * `to` - Quote currency code (e.g., `"USD"`)
/// * `amount` - Amount to convert
pub async fn currency_conversion(from: &str, to: &str, amount: f64) -> Result<CurrencyConversion> {
    let client = build_client()?;
    let path = format!("/v1/conversion/{}/{}", from, to);
    let amount_str = amount.to_string();
    let params = [("amount", amount_str.as_str())];
    let json = client.get_raw(&path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "currency_conversion".to_string(),
        context: format!("Failed to parse currency conversion response: {e}"),
    })
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

        let resp: ForexQuoteResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let last = resp.last.unwrap();
        assert!((last.bid.unwrap() - 1.1050).abs() < 0.0001);
        assert!((last.ask.unwrap() - 1.1052).abs() < 0.0001);
        assert_eq!(last.exchange.unwrap(), 48);
    }

    #[tokio::test]
    async fn test_forex_quotes_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/quotes/C:EURUSD")
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
                        { "ask_price": 1.1052, "bid_price": 1.1050, "ask_size": 1000.0, "bid_size": 1500.0, "sip_timestamp": 1705363200000000000_i64 },
                        { "ask_price": 1.1053, "bid_price": 1.1051, "ask_size": 800.0, "bid_size": 1200.0, "sip_timestamp": 1705363200100000000_i64 }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponse<Quote> = client.get("/v3/quotes/C:EURUSD", &[]).await.unwrap();
        let quotes = resp.results.unwrap();
        assert_eq!(quotes.len(), 2);
        assert!((quotes[0].ask_price.unwrap() - 1.1052).abs() < 0.0001);
        assert!((quotes[0].bid_price.unwrap() - 1.1050).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_currency_conversion_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v1/conversion/EUR/USD")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("amount".into(), "100".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "converted": 110.50,
                    "from": "EUR",
                    "to": "USD",
                    "initialAmount": 100.0,
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
            .get_raw("/v1/conversion/EUR/USD", &[("amount", "100")])
            .await
            .unwrap();

        let resp: CurrencyConversion = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        assert!((resp.converted.unwrap() - 110.50).abs() < 0.01);
        assert_eq!(resp.from.as_deref(), Some("EUR"));
        assert_eq!(resp.to.as_deref(), Some("USD"));
        assert!((resp.initial_amount.unwrap() - 100.0).abs() < 0.01);
        let last = resp.last.unwrap();
        assert!((last.bid.unwrap() - 1.1050).abs() < 0.0001);
    }
}
