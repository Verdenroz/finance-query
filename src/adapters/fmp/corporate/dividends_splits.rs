//! Dividend and stock split history endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// A single historical dividend record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendHistoryDTO {
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
pub struct DividendHistoryResponseDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Dividend history records.
    pub historical: Vec<DividendHistoryDTO>,
}

/// A single historical stock split record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SplitHistoryDTO {
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
pub struct SplitHistoryResponseDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Split history records.
    pub historical: Vec<SplitHistoryDTO>,
}

// ============================================================================
// Canonical conversion functions
// ============================================================================

/// Convert dividend and split DTOs into a canonical ChartEvents.
fn dividends_splits_to_events(
    divs: DividendHistoryResponseDTO,
    splits: SplitHistoryResponseDTO,
) -> crate::models::chart::events::ChartEvents {
    use crate::models::chart::events::{ChartEvents, DividendEvent, SplitEvent};
    let mut chart_events = ChartEvents::default();
    chart_events.dividends = divs
        .historical
        .into_iter()
        .filter_map(|d| {
            let ts = chrono::NaiveDate::parse_from_str(d.date.as_deref()?, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)?
                .and_utc()
                .timestamp();
            Some((
                ts.to_string(),
                DividendEvent {
                    date: ts,
                    amount: d.adj_dividend.or(d.dividend).unwrap_or(0.0),
                },
            ))
        })
        .collect();
    chart_events.splits = splits
        .historical
        .into_iter()
        .filter_map(|s| {
            let ts = chrono::NaiveDate::parse_from_str(s.date.as_deref()?, "%Y-%m-%d")
                .ok()?
                .and_hms_opt(0, 0, 0)?
                .and_utc()
                .timestamp();
            let numerator = s.numerator.unwrap_or(1.0);
            let denominator = s.denominator.unwrap_or(1.0);
            Some((
                ts.to_string(),
                SplitEvent {
                    date: ts,
                    numerator,
                    denominator,
                    split_ratio: format!("{}:{}", numerator, denominator),
                },
            ))
        })
        .collect();
    chart_events
}

/// Fetch canonical chart events (dividends + splits) for a symbol.
pub async fn fetch_canonical_events(
    symbol: &str,
) -> Result<crate::models::chart::events::ChartEvents> {
    let divs = historical_dividends(symbol).await?;
    let splits = historical_splits(symbol).await?;
    Ok(dividends_splits_to_events(divs, splits))
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch historical dividend data for a symbol.
pub async fn historical_dividends(symbol: &str) -> Result<DividendHistoryResponseDTO> {
    let client = build_client()?;
    let historical = client
        .get("/stable/dividends", &[("symbol", symbol)])
        .await?;
    Ok(DividendHistoryResponseDTO {
        symbol: Some(symbol.to_string()),
        historical,
    })
}

/// Fetch historical stock split data for a symbol.
pub async fn historical_splits(symbol: &str) -> Result<SplitHistoryResponseDTO> {
    let client = build_client()?;
    let historical = client.get("/stable/splits", &[("symbol", symbol)]).await?;
    Ok(SplitHistoryResponseDTO {
        symbol: Some(symbol.to_string()),
        historical,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_historical_dividends_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/dividends")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                        {
                            "date": "2024-02-09",
                            "label": "February 09, 24",
                            "adjDividend": 0.24,
                            "dividend": 0.24,
                            "recordDate": "2024-02-12",
                            "paymentDate": "2024-02-15",
                            "declarationDate": "2024-02-01"
                        }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let path = "/stable/dividends";
        let resp: Vec<DividendHistoryDTO> = client.get(path, &[]).await.unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].adj_dividend.unwrap() - 0.24).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_historical_splits_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/splits")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                        {
                            "date": "2020-08-31",
                            "label": "August 31, 20",
                            "numerator": 4.0,
                            "denominator": 1.0
                        }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let path = "/stable/splits";
        let resp: Vec<SplitHistoryDTO> = client.get(path, &[]).await.unwrap();
        assert_eq!(resp.len(), 1);
        assert!((resp[0].numerator.unwrap() - 4.0).abs() < 0.001);
    }
}
