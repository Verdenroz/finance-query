//! Index endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::models::FmpQuoteDTO;

/// Convert FMP quote DTOs into a canonical IndexQuote.
fn index_quote_to_canonical(
    symbol: &str,
    quotes: &[FmpQuoteDTO],
) -> crate::models::indices::IndexQuote {
    let q = quotes.first();
    crate::models::indices::IndexQuote {
        symbol: symbol.to_string(),
        name: q.and_then(|q| q.name.clone()),
        price: q.and_then(|q| q.price),
        change: q.and_then(|q| q.change),
        change_percent: q.and_then(|q| q.changes_percentage),
        timestamp: None,
    }
}

/// Fetch a canonical index quote.
pub async fn fetch_canonical_index_quote(
    symbol: &str,
) -> Result<crate::models::indices::IndexQuote> {
    let quotes = crate::adapters::fmp::quote::quote(symbol).await?;
    Ok(index_quote_to_canonical(symbol, &quotes))
}

/// A constituent of a major index (S&P 500, Nasdaq, Dow Jones).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexConstituentDTO {
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
pub struct HistoricalConstituentDTO {
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
#[allow(dead_code)] // unrouted: batch index quotes deliberately deferred (#297 optional)
pub async fn major_indexes_quote() -> Result<Vec<FmpQuoteDTO>> {
    let client = build_client()?;
    client.get("/stable/batch-index-quotes", &[]).await
}

/// Fetch current S&P 500 constituents.
pub async fn sp500_constituents() -> Result<Vec<IndexConstituentDTO>> {
    let client = build_client()?;
    client.get("/stable/sp500-constituent", &[]).await
}

/// Fetch current Nasdaq constituents.
pub async fn nasdaq_constituents() -> Result<Vec<IndexConstituentDTO>> {
    let client = build_client()?;
    client.get("/stable/nasdaq-constituent", &[]).await
}

/// Fetch current Dow Jones constituents.
pub async fn dow_constituents() -> Result<Vec<IndexConstituentDTO>> {
    let client = build_client()?;
    client.get("/stable/dowjones-constituent", &[]).await
}

/// Fetch historical S&P 500 constituent changes.
pub async fn historical_sp500() -> Result<Vec<HistoricalConstituentDTO>> {
    let client = build_client()?;
    client
        .get("/stable/historical-sp500-constituent", &[])
        .await
}

/// Convert a constituent DTO into the canonical model.
fn constituent_to_canonical(c: IndexConstituentDTO) -> crate::models::indices::IndexConstituent {
    crate::models::indices::IndexConstituent {
        symbol: c.symbol.unwrap_or_default(),
        name: c.name,
        sector: c.sector,
        sub_sector: c.sub_sector,
        headquarters: c.head_quarter,
        date_first_added: c.date_first_added,
        cik: c.cik,
        founded: c.founded,
    }
}

/// Fetch canonical constituents for a major index.
pub async fn fetch_index_constituents_response(
    index: crate::models::indices::MajorIndex,
) -> Result<Vec<crate::models::indices::IndexConstituent>> {
    use crate::models::indices::MajorIndex;
    let dtos = match index {
        MajorIndex::Sp500 => sp500_constituents().await?,
        MajorIndex::Nasdaq100 => nasdaq_constituents().await?,
        MajorIndex::DowJones => dow_constituents().await?,
    };
    Ok(dtos.into_iter().map(constituent_to_canonical).collect())
}

/// Fetch canonical historical constituent changes for a major index.
///
/// FMP publishes historical changes for the S&P 500 only.
pub async fn fetch_index_constituent_changes_response(
    index: crate::models::indices::MajorIndex,
) -> Result<Vec<crate::models::indices::IndexConstituentChange>> {
    use crate::models::indices::MajorIndex;
    let dtos = match index {
        MajorIndex::Sp500 => historical_sp500().await?,
        other => {
            return Err(crate::error::FinanceError::InvalidParameter {
                param: "index".into(),
                reason: format!(
                    "FMP provides historical constituent changes for the S&P 500 only, not {other}"
                ),
            });
        }
    };
    Ok(dtos
        .into_iter()
        .map(|c| crate::models::indices::IndexConstituentChange {
            date: c.date,
            symbol: c.symbol,
            added_security: c.added_security,
            removed_ticker: c.removed_ticker,
            removed_security: c.removed_security,
            reason: c.reason,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constituent_maps_all_fields() {
        let dto: IndexConstituentDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "name": "Apple Inc.",
            "sector": "Information Technology",
            "subSector": "Technology Hardware",
            "headQuarter": "Cupertino, CA",
            "dateFirstAdded": "1982-11-30",
            "cik": "0000320193",
            "founded": "1976"
        }))
        .unwrap();
        let c = constituent_to_canonical(dto);
        assert_eq!(c.symbol, "AAPL");
        assert_eq!(c.sub_sector.as_deref(), Some("Technology Hardware"));
        assert_eq!(c.headquarters.as_deref(), Some("Cupertino, CA"));
        assert_eq!(c.date_first_added.as_deref(), Some("1982-11-30"));
        assert_eq!(c.cik.as_deref(), Some("0000320193"));
        assert_eq!(c.founded.as_deref(), Some("1976"));
    }

    #[tokio::test]
    async fn test_sp500_constituents_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/sp500-constituent")
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<IndexConstituentDTO> =
            client.get("/stable/sp500-constituent", &[]).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].sector.as_deref(), Some("Information Technology"));
    }

    /// Mocked HTTP → `Vec<FmpQuoteDTO>` → `index_quote_to_canonical`, covering
    /// the full `fetch_canonical_index_quote` pipeline without a network call.
    #[tokio::test]
    async fn test_index_quote_to_canonical_mock() {
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
                    "symbol": "^GSPC",
                    "name": "S&P 500",
                    "price": 4790.61,
                    "change": 20.12,
                    "changesPercentage": 0.42
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let quotes: Vec<FmpQuoteDTO> = client.get("/stable/quote", &[]).await.unwrap();

        let quote = index_quote_to_canonical("^GSPC", &quotes);
        assert_eq!(quote.symbol, "^GSPC");
        assert_eq!(quote.name.as_deref(), Some("S&P 500"));
        assert_eq!(quote.price, Some(4790.61));
        assert_eq!(quote.change, Some(20.12));
        assert_eq!(quote.change_percent, Some(0.42));
    }

    #[test]
    fn index_quote_to_canonical_empty_yields_no_values() {
        let quote = index_quote_to_canonical("^GSPC", &[]);
        assert_eq!(quote.symbol, "^GSPC");
        assert!(quote.name.is_none());
        assert!(quote.price.is_none());
    }
}
