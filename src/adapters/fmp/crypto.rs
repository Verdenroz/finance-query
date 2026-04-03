//! Cryptocurrency endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::{FmpQuote, HistoricalPriceResponse, IntradayPrice};

/// An available cryptocurrency or forex/commodity symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvailableSymbol {
    /// Ticker symbol (e.g., `"BTCUSD"`).
    pub symbol: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Currency.
    pub currency: Option<String>,
    /// Exchange name.
    #[serde(rename = "stockExchange")]
    pub stock_exchange: Option<String>,
    /// Short exchange name.
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
}

/// Fetch a real-time crypto quote.
///
/// * `symbol` - e.g., `"BTCUSD"`
pub async fn crypto_quote(symbol: &str) -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{symbol}");
    client.get(&path, &[]).await
}

/// List all available cryptocurrency pairs.
pub async fn crypto_available() -> Result<Vec<AvailableSymbol>> {
    let client = build_client()?;
    client
        .get("/api/v3/symbol/available-cryptocurrencies", &[])
        .await
}

/// Fetch daily historical prices for a cryptocurrency.
///
/// * `symbol` - e.g., `"BTCUSD"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn crypto_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/{symbol}");
    client.get(&path, params).await
}

/// Fetch intraday prices for a cryptocurrency.
///
/// * `symbol` - e.g., `"BTCUSD"`
/// * `interval` - e.g., `"1min"`, `"5min"`, `"15min"`, `"30min"`, `"1hour"`, `"4hour"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn crypto_intraday(
    symbol: &str,
    interval: &str,
    params: &[(&str, &str)],
) -> Result<Vec<IntradayPrice>> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-chart/{interval}/{symbol}");
    client.get(&path, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/symbol/available-cryptocurrencies")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "BTCUSD",
                        "name": "Bitcoin USD",
                        "currency": "USD",
                        "stockExchange": "CCC",
                        "exchangeShortName": "CRYPTO"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbol> = client
            .get("/api/v3/symbol/available-cryptocurrencies", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("BTCUSD"));
    }
}
