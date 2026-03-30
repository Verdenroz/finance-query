//! ETF and mutual fund endpoints for Financial Modeling Prep.

use crate::error::Result;

use super::build_client;
use super::crypto::AvailableSymbol;
use super::models::{FmpQuote, HistoricalPriceResponse};

/// Fetch a real-time ETF quote.
///
/// * `symbol` - e.g., `"SPY"`
pub async fn etf_quote(symbol: &str) -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{symbol}");
    client.get(&path, &[]).await
}

/// List all available ETFs.
pub async fn etf_available() -> Result<Vec<AvailableSymbol>> {
    let client = build_client()?;
    client.get("/api/v3/symbol/available-etfs", &[]).await
}

/// Fetch daily historical prices for an ETF.
///
/// * `symbol` - e.g., `"SPY"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn etf_historical(
    symbol: &str,
    params: &[(&str, &str)],
) -> Result<HistoricalPriceResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/{symbol}");
    client.get(&path, params).await
}

/// Fetch a real-time mutual fund quote.
///
/// * `symbol` - e.g., `"VFIAX"`
pub async fn mutual_fund_quote(symbol: &str) -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    let path = format!("/api/v3/quote/{symbol}");
    client.get(&path, &[]).await
}

/// List all available mutual funds.
pub async fn mutual_fund_available() -> Result<Vec<AvailableSymbol>> {
    let client = build_client()?;
    client
        .get("/api/v3/symbol/available-mutual-funds", &[])
        .await
}

/// Fetch daily historical prices for a mutual fund.
///
/// * `symbol` - e.g., `"VFIAX"`
/// * `params` - Optional query params such as `from`, `to`
pub async fn mutual_fund_historical(
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
    async fn test_etf_available_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/symbol/available-etfs")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<AvailableSymbol> = client
            .get("/api/v3/symbol/available-etfs", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("SPY"));
    }
}
