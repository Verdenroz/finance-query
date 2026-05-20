//! Index snapshot endpoints.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};
use crate::models::indices::IndexQuote;

use super::super::build_client;

/// Session data within an index snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexSessionDTO {
    /// Change from previous close.
    pub change: Option<f64>,
    /// Change percent from previous close.
    pub change_percent: Option<f64>,
    /// Close value.
    pub close: Option<f64>,
    /// High value.
    pub high: Option<f64>,
    /// Low value.
    pub low: Option<f64>,
    /// Open value.
    pub open: Option<f64>,
    /// Previous close value.
    pub previous_close: Option<f64>,
}

/// A single index snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexSnapshotDTO {
    /// Current index value.
    pub value: Option<f64>,
    /// Name of the index.
    pub name: Option<String>,
    /// Type of the index.
    #[serde(rename = "type")]
    pub index_type: Option<String>,
    /// Ticker symbol (e.g., `"I:SPX"`).
    pub ticker: Option<String>,
    /// Market status.
    pub market_status: Option<String>,
    /// Session data.
    pub session: Option<IndexSessionDTO>,
}

/// Response wrapper for index snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IndexSnapshotResponseDTO {
    /// Response status.
    pub status: Option<String>,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Index snapshot results.
    pub results: Option<Vec<IndexSnapshotDTO>>,
}

/// Fetch snapshot for a single index ticker.
///
/// * `ticker` - Index ticker symbol with `I:` prefix (e.g., `"I:SPX"`)
pub async fn index_snapshot(ticker: &str) -> Result<IndexSnapshotResponseDTO> {
    let client = build_client()?;
    let path = "/v3/snapshot/indices";
    let params = [("ticker.any_of", ticker)];
    let json = client.get_raw(path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "index_snapshot".to_string(),
        context: format!("Failed to parse index snapshot response: {e}"),
    })
}

/// Fetch index quote (canonical) for a symbol.
pub async fn fetch_indices_quote_response(symbol: &str) -> Result<IndexQuote> {
    let resp = index_snapshot(symbol).await?;
    let snap = resp.results.and_then(|mut v| {
        if v.is_empty() {
            None
        } else {
            Some(v.remove(0))
        }
    });
    let session = snap.as_ref().and_then(|s| s.session.as_ref());
    Ok(IndexQuote {
        symbol: snap
            .as_ref()
            .and_then(|s| s.ticker.clone())
            .unwrap_or_else(|| symbol.to_string()),
        name: snap.as_ref().and_then(|s| s.name.clone()),
        price: snap.as_ref().and_then(|s| s.value),
        change: session.and_then(|s| s.change),
        change_percent: session.and_then(|s| s.change_percent),
        timestamp: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_index_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/snapshot/indices")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("ticker.any_of".into(), "I:SPX".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": [
                        {
                            "value": 4790.0,
                            "name": "S&P 500",
                            "type": "indices",
                            "ticker": "I:SPX",
                            "market_status": "open",
                            "session": {
                                "change": 20.0,
                                "change_percent": 0.42,
                                "close": 4790.0,
                                "high": 4800.0,
                                "low": 4760.0,
                                "open": 4770.0,
                                "previous_close": 4770.0
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/snapshot/indices", &[("ticker.any_of", "I:SPX")])
            .await
            .unwrap();

        let resp: IndexSnapshotResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ticker.as_deref(), Some("I:SPX"));
        assert!((results[0].value.unwrap() - 4790.0).abs() < 0.01);
        let session = results[0].session.as_ref().unwrap();
        assert!((session.change.unwrap() - 20.0).abs() < 0.01);
    }
}
