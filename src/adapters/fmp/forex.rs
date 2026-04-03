//! Forex endpoints for Financial Modeling Prep.

use crate::error::Result;

use super::build_client;
use super::crypto::AvailableSymbol;
use super::models::{FmpQuote, HistoricalPriceResponse, IntradayPrice};

/// Fetch a real-time forex quote.
///
/// * `symbol` - e.g., `"EURUSD"`
pub async fn forex_quote(symbol: &str) -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{symbol}");
    client.get(&path, &[]).await
}

/// List all available forex currency pairs.
pub async fn forex_available() -> Result<Vec<AvailableSymbol>> {
    let client = build_client()?;
    client
        .get("/api/v3/symbol/available-forex-currency-pairs", &[])
        .await
}

/// Fetch daily historical prices for a forex pair.
///
/// * `symbol` - e.g., `"EURUSD"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn forex_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/{symbol}");
    client.get(&path, params).await
}

/// Fetch intraday prices for a forex pair.
///
/// * `symbol` - e.g., `"EURUSD"`
/// * `interval` - e.g., `"1min"`, `"5min"`, `"15min"`, `"30min"`, `"1hour"`, `"4hour"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn forex_intraday(
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
    async fn test_forex_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/symbol/available-forex-currency-pairs")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "EURUSD",
                        "name": "EUR/USD",
                        "currency": "USD",
                        "stockExchange": "CCY",
                        "exchangeShortName": "FOREX"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbol> = client
            .get("/api/v3/symbol/available-forex-currency-pairs", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("EURUSD"));
    }
}
