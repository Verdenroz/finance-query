//! Commodities endpoints for Financial Modeling Prep.

#![allow(dead_code)]
use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::crypto::AvailableSymbolDTO;
use crate::adapters::fmp::models::{FmpQuoteDTO, HistoricalPriceResponseDTO};

/// Convert FMP quote DTOs into a canonical CommodityQuote.
fn commodity_quote_to_canonical(
    symbol: &str,
    quotes: &[FmpQuoteDTO],
) -> crate::models::commodities::CommodityQuote {
    let q = quotes.first();
    crate::models::commodities::CommodityQuote {
        symbol: symbol.to_string(),
        name: q.and_then(|q| q.name.clone()),
        unit: None,
        price: q.and_then(|q| q.price),
        change: q.and_then(|q| q.change),
        change_percent: q.and_then(|q| q.changes_percentage),
        timestamp: None,
    }
}

/// Fetch a canonical commodity quote.
pub async fn fetch_canonical_commodity_quote(
    symbol: &str,
) -> Result<crate::models::commodities::CommodityQuote> {
    let quotes = commodity_quote(symbol).await?;
    Ok(commodity_quote_to_canonical(symbol, &quotes))
}

/// Fetch a real-time commodity quote.
///
/// * `symbol` - e.g., `"GCUSD"` (gold)
pub async fn commodity_quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{}", encode_path_segment(symbol));
    client.get(&path, &[]).await
}

/// List all available commodities.
pub async fn commodity_available() -> Result<Vec<AvailableSymbolDTO>> {
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
) -> Result<HistoricalPriceResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/api/v3/historical-price-full/{}",
        encode_path_segment(symbol)
    );
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
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbolDTO> = client
            .get("/api/v3/symbol/available-commodities", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("GCUSD"));
    }
}
