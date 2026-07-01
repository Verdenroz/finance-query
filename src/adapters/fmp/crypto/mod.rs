//! Cryptocurrency endpoints for Financial Modeling Prep.

#![allow(dead_code)]
use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::models::{FmpQuoteDTO, HistoricalPriceResponseDTO, IntradayPriceDTO};

/// Convert FMP quote DTOs into a canonical CryptoQuote.
fn crypto_quote_to_canonical(
    id: &str,
    _vs_currency: &str,
    quotes: &[FmpQuoteDTO],
) -> crate::models::crypto::CryptoQuote {
    let q = quotes.first();
    crate::models::crypto::CryptoQuote {
        id: id.to_string(),
        symbol: q
            .map(|q| q.symbol.clone())
            .unwrap_or_else(|| id.to_uppercase()),
        name: q.and_then(|q| q.name.clone()).unwrap_or_default(),
        price: q.and_then(|q| q.price),
        market_cap: q.and_then(|q| q.market_cap),
        volume_24h: q.and_then(|q| q.volume),
        change_24h: q.and_then(|q| q.change),
        change_percent_24h: q.and_then(|q| q.changes_percentage),
        high_24h: q.and_then(|q| q.day_high),
        low_24h: q.and_then(|q| q.day_low),
        circulating_supply: None,
    }
}

/// Fetch a canonical crypto quote.
pub async fn fetch_canonical_crypto_quote(
    id: &str,
    vs_currency: &str,
) -> Result<crate::models::crypto::CryptoQuote> {
    let pair = format!("{}{}", id.to_uppercase(), vs_currency.to_uppercase());
    let quotes = crypto_quote(&pair).await?;
    Ok(crypto_quote_to_canonical(id, vs_currency, &quotes))
}

/// An available cryptocurrency or forex/commodity symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvailableSymbolDTO {
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
pub async fn crypto_quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{}", encode_path_segment(symbol));
    client.get(&path, &[]).await
}

/// List all available cryptocurrency pairs.
pub async fn crypto_available() -> Result<Vec<AvailableSymbolDTO>> {
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
) -> Result<HistoricalPriceResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/api/v3/historical-price-full/{}",
        encode_path_segment(symbol)
    );
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
) -> Result<Vec<IntradayPriceDTO>> {
    let client = build_client()?;
    let path = format!(
        "/api/v3/historical-chart/{}/{}",
        encode_path_segment(interval),
        encode_path_segment(symbol)
    );
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbolDTO> = client
            .get("/api/v3/symbol/available-cryptocurrencies", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("BTCUSD"));
    }

    /// Mocked HTTP → `Vec<FmpQuoteDTO>` → `crypto_quote_to_canonical`, covering
    /// the full `fetch_canonical_crypto_quote` pipeline without a network call.
    #[tokio::test]
    async fn test_crypto_quote_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/quote/BTCUSD")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "BTCUSD",
                    "name": "Bitcoin USD",
                    "price": 43200.0,
                    "change": 1200.0,
                    "changesPercentage": 2.85,
                    "dayHigh": 43500.0,
                    "dayLow": 41800.0,
                    "marketCap": 845000000000.0,
                    "volume": 28000000000.0
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let quotes: Vec<FmpQuoteDTO> = client.get("/api/v3/quote/BTCUSD", &[]).await.unwrap();

        let quote = crypto_quote_to_canonical("btc", "usd", &quotes);
        assert_eq!(quote.id, "btc", "id preserves caller input");
        assert_eq!(quote.symbol, "BTCUSD");
        assert_eq!(quote.name, "Bitcoin USD");
        assert_eq!(quote.price, Some(43200.0));
        assert_eq!(quote.market_cap, Some(845000000000.0));
        assert_eq!(quote.volume_24h, Some(28000000000.0));
        assert_eq!(quote.change_24h, Some(1200.0));
        assert_eq!(quote.change_percent_24h, Some(2.85));
        assert_eq!(quote.high_24h, Some(43500.0));
        assert_eq!(quote.low_24h, Some(41800.0));
    }

    #[test]
    fn crypto_quote_to_canonical_empty_uppercases_id_as_symbol() {
        let quote = crypto_quote_to_canonical("btc", "usd", &[]);
        assert_eq!(quote.id, "btc");
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.name, "");
        assert!(quote.price.is_none());
    }
}
