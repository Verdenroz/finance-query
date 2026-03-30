//! Dividend and stock split history endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// A single historical dividend record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendHistory {
    /// Ex-dividend date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Human-readable label.
    pub label: Option<String>,
    /// Adjusted dividend amount.
    #[serde(rename = "adjDividend")]
    pub adj_dividend: Option<f64>,
    /// Unadjusted dividend amount.
    pub dividend: Option<f64>,
    /// Record date.
    #[serde(rename = "recordDate")]
    pub record_date: Option<String>,
    /// Payment date.
    #[serde(rename = "paymentDate")]
    pub payment_date: Option<String>,
    /// Declaration date.
    #[serde(rename = "declarationDate")]
    pub declaration_date: Option<String>,
}

/// Historical dividend response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendHistoryResponse {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Dividend history records.
    pub historical: Vec<DividendHistory>,
}

/// A single historical stock split record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SplitHistory {
    /// Split date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Human-readable label.
    pub label: Option<String>,
    /// Split numerator.
    pub numerator: Option<f64>,
    /// Split denominator.
    pub denominator: Option<f64>,
}

/// Historical stock split response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SplitHistoryResponse {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Split history records.
    pub historical: Vec<SplitHistory>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch historical dividend data for a symbol.
pub async fn historical_dividends(symbol: &str) -> Result<DividendHistoryResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/stock_dividend/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch historical stock split data for a symbol.
pub async fn historical_splits(symbol: &str) -> Result<SplitHistoryResponse> {
    let client = build_client()?;
    let path = format!("/api/v3/historical-price-full/stock_split/{symbol}");
    client.get(&path, &[]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_historical_dividends_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/api/v3/historical-price-full/stock_dividend/AAPL",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "symbol": "AAPL",
                    "historical": [
                        {
                            "date": "2024-02-09",
                            "label": "February 09, 24",
                            "adjDividend": 0.24,
                            "dividend": 0.24,
                            "recordDate": "2024-02-12",
                            "paymentDate": "2024-02-15",
                            "declarationDate": "2024-02-01"
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let path = "/api/v3/historical-price-full/stock_dividend/AAPL";
        let resp: DividendHistoryResponse = client.get(path, &[]).await.unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert_eq!(resp.historical.len(), 1);
        assert!((resp.historical[0].adj_dividend.unwrap() - 0.24).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_historical_splits_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/api/v3/historical-price-full/stock_split/AAPL",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!({
                    "symbol": "AAPL",
                    "historical": [
                        {
                            "date": "2020-08-31",
                            "label": "August 31, 20",
                            "numerator": 4.0,
                            "denominator": 1.0
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let path = "/api/v3/historical-price-full/stock_split/AAPL";
        let resp: SplitHistoryResponse = client.get(path, &[]).await.unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert_eq!(resp.historical.len(), 1);
        assert!((resp.historical[0].numerator.unwrap() - 4.0).abs() < 0.001);
    }
}
