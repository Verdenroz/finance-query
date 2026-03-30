//! Stock list and tradable symbol endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

/// An entry from the stock/ETF list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockListEntry {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company or security name.
    pub name: Option<String>,
    /// Current price.
    pub price: Option<f64>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Short exchange name.
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
    /// Security type (e.g., `"stock"`, `"etf"`).
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

/// Fetch the full list of stocks.
pub async fn symbols_list() -> Result<Vec<StockListEntry>> {
    let client = build_client()?;
    client.get("/api/v3/stock/list", &[]).await
}

/// Fetch all tradable symbols.
pub async fn tradable_symbols_list() -> Result<Vec<StockListEntry>> {
    let client = build_client()?;
    client.get("/api/v3/available-traded/list", &[]).await
}

/// Fetch the full list of ETFs.
pub async fn etf_list() -> Result<Vec<StockListEntry>> {
    let client = build_client()?;
    client.get("/api/v3/etf/list", &[]).await
}

/// Fetch symbols that have financial statements available.
pub async fn financial_statement_symbols() -> Result<Vec<String>> {
    let client = build_client()?;
    client
        .get("/api/v3/financial-statement-symbol-lists", &[])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_symbols_list_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/stock/list")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "name": "Apple Inc.",
                        "price": 175.50,
                        "exchange": "NASDAQ Global Select",
                        "exchangeShortName": "NASDAQ",
                        "type": "stock"
                    },
                    {
                        "symbol": "MSFT",
                        "name": "Microsoft Corporation",
                        "price": 380.20,
                        "exchange": "NASDAQ Global Select",
                        "exchangeShortName": "NASDAQ",
                        "type": "stock"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<StockListEntry> = client
            .get("/api/v3/stock/list", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].type_.as_deref(), Some("stock"));
    }
}
