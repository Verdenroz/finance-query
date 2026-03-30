//! Commodities endpoints for Financial Modeling Prep.

use crate::error::Result;

use super::build_client;
use super::crypto::AvailableSymbol;
use super::models::{FmpQuote, HistoricalPriceResponse};

/// Fetch a real-time commodity quote.
///
/// * `symbol` - e.g., `"GCUSD"` (gold)
pub async fn commodity_quote(symbol: &str) -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{symbol}");
    client.get(&path, &[]).await
}

/// List all available commodities.
pub async fn commodity_available() -> Result<Vec<AvailableSymbol>> {
    let client = build_client()?;
    client
        .get("/api/v3/symbol/available-commodities", &[])
        .await
}

/// Fetch daily historical prices for a commodity.
///
/// * `symbol` - e.g., `"GCUSD"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn commodity_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/{symbol}");
    client.get(&path, params).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_commodity_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/symbol/available-commodities")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "GCUSD",
                        "name": "Gold",
                        "currency": "USD",
                        "stockExchange": "COMMODITY",
                        "exchangeShortName": "COMMODITY"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbol> = client
            .get("/api/v3/symbol/available-commodities", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("GCUSD"));
    }
}
