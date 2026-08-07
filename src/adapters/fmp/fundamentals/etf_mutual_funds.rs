//! ETF and mutual fund endpoints for Financial Modeling Prep.

use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::crypto::AvailableSymbolDTO;
use crate::adapters::fmp::models::{FmpQuoteDTO, HistoricalPriceResponseDTO};

/// Fetch a real-time ETF quote.
///
/// * `symbol` - e.g., `"SPY"`
pub async fn etf_quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    client.get("/stable/quote", &[("symbol", symbol)]).await
}

/// List all available ETFs.
pub async fn etf_available() -> Result<Vec<AvailableSymbolDTO>> {
    let client = build_client()?;
    client.get("/stable/etf-list", &[]).await
}

/// Fetch daily historical prices for an ETF.
///
/// * `symbol` - e.g., `"SPY"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn etf_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponseDTO> {
    let client = build_client()?;
    let mut query = params.to_vec();
    query.push(("symbol", symbol));
    let historical = client
        .get("/stable/historical-price-eod/full", &query)
        .await?;
    Ok(HistoricalPriceResponseDTO {
        symbol: Some(symbol.to_string()),
        historical,
    })
}

/// Fetch a real-time mutual fund quote.
///
/// * `symbol` - e.g., `"VFIAX"`
pub async fn mutual_fund_quote(symbol: &str) -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    client.get("/stable/quote", &[("symbol", symbol)]).await
}

/// Fetch daily historical prices for a mutual fund.
///
/// * `symbol` - e.g., `"VFIAX"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn mutual_fund_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponseDTO> {
    let client = build_client()?;
    let mut query = params.to_vec();
    query.push(("symbol", symbol));
    let historical = client
        .get("/stable/historical-price-eod/full", &query)
        .await?;
    Ok(HistoricalPriceResponseDTO {
        symbol: Some(symbol.to_string()),
        historical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_etf_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/etf-list")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "SPY",
                        "name": "SPDR S&P 500 ETF Trust",
                        "currency": "USD",
                        "stockExchange": "NYSE Arca",
                        "exchangeShortName": "AMEX"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbolDTO> = client.get("/stable/etf-list", &[]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("SPY"));
    }
}
