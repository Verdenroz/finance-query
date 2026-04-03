//! Futures snapshot endpoints.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use super::super::build_client;

/// Session data within a futures snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSession {
    /// Change from previous close.
    pub change: Option<f64>,
    /// Change percent from previous close.
    pub change_percent: Option<f64>,
    /// Close price.
    pub close: Option<f64>,
    /// High price.
    pub high: Option<f64>,
    /// Low price.
    pub low: Option<f64>,
    /// Open price.
    pub open: Option<f64>,
    /// Previous close price.
    pub previous_close: Option<f64>,
    /// Settlement price.
    pub settlement: Option<f64>,
    /// Volume.
    pub volume: Option<f64>,
}

/// A single futures snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSnapshot {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Name of the contract.
    pub name: Option<String>,
    /// Market status.
    pub market_status: Option<String>,
    /// Type.
    #[serde(rename = "type")]
    pub snapshot_type: Option<String>,
    /// Session data.
    pub session: Option<FuturesSession>,
    /// Last updated timestamp.
    pub last_updated: Option<i64>,
}

/// Response wrapper for futures snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FuturesSnapshotResponse {
    /// Response status.
    pub status: Option<String>,
    /// Request identifier.
    pub request_id: Option<String>,
    /// Snapshot results.
    pub results: Option<Vec<FuturesSnapshot>>,
}

/// Fetch snapshot for a futures ticker.
///
/// * `ticker` - Futures ticker symbol (e.g., `"ESZ4"`)
pub async fn futures_snapshot(ticker: &str) -> Result<FuturesSnapshotResponse> {
    let client = build_client()?;
    let path = format!("/v3/snapshot/futures/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "futures_snapshot".to_string(),
        context: format!("Failed to parse futures snapshot response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_futures_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/snapshot/futures/ESZ4")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "request_id": "abc123",
                    "results": [
                        {
                            "ticker": "ESZ4",
                            "name": "E-mini S&P 500 Dec 2024",
                            "market_status": "open",
                            "type": "futures",
                            "session": {
                                "change": 15.0,
                                "change_percent": 0.31,
                                "close": 4790.0,
                                "high": 4800.0,
                                "low": 4760.0,
                                "open": 4775.0,
                                "previous_close": 4775.0,
                                "settlement": 4785.0,
                                "volume": 1500000.0
                            },
                            "last_updated": 1705363200000000000_i64
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/snapshot/futures/ESZ4", &[])
            .await
            .unwrap();

        let resp: FuturesSnapshotResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let results = resp.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].ticker.as_deref(), Some("ESZ4"));
        let session = results[0].session.as_ref().unwrap();
        assert!((session.change.unwrap() - 15.0).abs() < 0.01);
        assert!((session.close.unwrap() - 4790.0).abs() < 0.01);
    }
}
