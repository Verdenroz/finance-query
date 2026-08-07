//! Forex endpoints for Financial Modeling Prep.

use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::models::FmpQuoteDTO;

/// Convert FMP quote DTOs into a canonical ForexQuote.
fn forex_quote_to_canonical(
    pair: &str,
    from: &str,
    to: &str,
    quotes: &[FmpQuoteDTO],
) -> crate::models::forex::ForexQuote {
    let q = quotes.first();
    let price = q.and_then(|q| q.price);
    crate::models::forex::ForexQuote {
        symbol: pair.to_string(),
        base_currency: Some(from.to_uppercase()),
        quote_currency: Some(to.to_uppercase()),
        bid: price,
        ask: None,
        price,
        change: None,
        change_percent: None,
        timestamp: q.and_then(|q| q.timestamp),
    }
}

/// Fetch a canonical forex quote.
pub async fn fetch_canonical_forex_quote(
    from: &str,
    to: &str,
) -> Result<crate::models::forex::ForexQuote> {
    let pair = format!("{}{}", from.to_uppercase(), to.to_uppercase());
    let quotes = forex_quote(&pair).await?;
    Ok(forex_quote_to_canonical(&pair, from, to, &quotes))
}

/// Fetch a real-time forex quote.
///
/// * `symbol` - e.g., `"EURUSD"`
pub async fn forex_quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    client.get("/stable/quote", &[("symbol", symbol)]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mocked HTTP → `Vec<FmpQuoteDTO>` → `forex_quote_to_canonical`, covering
    /// the full `fetch_canonical_forex_quote` pipeline without a network call.
    #[tokio::test]
    async fn test_forex_quote_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/quote")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "EURUSD",
                    "price": 1.1050,
                    "timestamp": 1705363200
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let quotes: Vec<FmpQuoteDTO> = client.get("/stable/quote", &[]).await.unwrap();

        let quote = forex_quote_to_canonical("EURUSD", "eur", "usd", &quotes);
        assert_eq!(quote.symbol, "EURUSD");
        assert_eq!(quote.base_currency.as_deref(), Some("EUR"));
        assert_eq!(quote.quote_currency.as_deref(), Some("USD"));
        assert_eq!(quote.price, Some(1.1050));
        assert_eq!(
            quote.bid,
            Some(1.1050),
            "bid mirrors price (FMP has no bid/ask)"
        );
        assert!(quote.ask.is_none());
        assert_eq!(quote.timestamp, Some(1705363200));
    }

    #[test]
    fn forex_quote_to_canonical_empty_yields_no_price() {
        let quote = forex_quote_to_canonical("EURUSD", "EUR", "USD", &[]);
        assert_eq!(quote.symbol, "EURUSD");
        assert!(quote.price.is_none());
        assert!(quote.timestamp.is_none());
    }
}
