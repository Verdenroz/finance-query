//! Index endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;
use super::models::{FmpQuote, HistoricalPriceResponse};

/// A constituent of a major index (S&P 500, Nasdaq, Dow Jones).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexConstituent {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Sub-sector.
    #[serde(rename = "subSector")]
    pub sub_sector: Option<String>,
    /// Headquarters location.
    #[serde(rename = "headQuarter")]
    pub head_quarter: Option<String>,
    /// Date first added to the index.
    #[serde(rename = "dateFirstAdded")]
    pub date_first_added: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Year the company was founded.
    pub founded: Option<String>,
}

/// A historical change in index constituency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoricalConstituent {
    /// Date of the change.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Security that was added.
    #[serde(rename = "addedSecurity")]
    pub added_security: Option<String>,
    /// Ticker that was removed.
    #[serde(rename = "removedTicker")]
    pub removed_ticker: Option<String>,
    /// Security that was removed.
    #[serde(rename = "removedSecurity")]
    pub removed_security: Option<String>,
    /// Reason for the change.
    pub reason: Option<String>,
}

/// Fetch real-time quotes for all major indexes.
pub async fn major_indexes_quote() -> Result<Vec<FmpQuote>> {
    let client = build_client()?;
    client.get("/api/v3/quotes/index", &[]).await
}

/// Fetch current S&P 500 constituents.
pub async fn sp500_constituents() -> Result<Vec<IndexConstituent>> {
    let client = build_client()?;
    client.get("/api/v3/sp500_constituent", &[]).await
}

/// Fetch current Nasdaq constituents.
pub async fn nasdaq_constituents() -> Result<Vec<IndexConstituent>> {
    let client = build_client()?;
    client.get("/api/v3/nasdaq_constituent", &[]).await
}

/// Fetch current Dow Jones constituents.
pub async fn dow_constituents() -> Result<Vec<IndexConstituent>> {
    let client = build_client()?;
    client.get("/api/v3/dowjones_constituent", &[]).await
}

/// Fetch historical S&P 500 constituent changes.
pub async fn historical_sp500() -> Result<Vec<HistoricalConstituent>> {
    let client = build_client()?;
    client
        .get("/api/v3/historical/sp500_constituent", &[])
        .await
}

/// Fetch daily historical prices for an index.
///
/// * `symbol` - e.g., `"^GSPC"` (S&P 500)
/// * `params` - Optional query params such as `from`, `to`
pub async fn index_historical(
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
    async fn test_sp500_constituents_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/sp500_constituent")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "name": "Apple Inc.",
                        "sector": "Information Technology",
                        "subSector": "Technology Hardware",
                        "headQuarter": "Cupertino, CA",
                        "dateFirstAdded": "1982-11-30",
                        "cik": "0000320193",
                        "founded": "1976"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<IndexConstituent> =
            client.get("/api/v3/sp500_constituent", &[]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].sector.as_deref(), Some("Information Technology"));
    }
}
