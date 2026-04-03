//! Crypto snapshot endpoints: all tickers, single ticker, top movers.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch snapshots for all crypto tickers.
///
/// * `tickers` - Optional comma-separated list of tickers to filter (e.g., `"X:BTCUSD,X:ETHUSD"`)
pub async fn crypto_snapshots_all(tickers: Option<&str>) -> Result<SnapshotsResponse> {
    let client = build_client()?;
    let path = "/v2/snapshot/locale/global/markets/crypto/tickers";
    let params: Vec<(&str, &str)> = match tickers {
        Some(t) => vec![("tickers", t)],
        None => vec![],
    };
    let json = client.get_raw(path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_snapshots".to_string(),
        context: format!("Failed to parse crypto snapshots response: {e}"),
    })
}

/// Fetch snapshot for a single crypto ticker.
///
/// * `ticker` - Crypto ticker symbol with `X:` prefix (e.g., `"X:BTCUSD"`)
pub async fn crypto_snapshot(ticker: &str) -> Result<SingleSnapshotResponse> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/global/markets/crypto/tickers/{}",
        ticker
    );
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_snapshot".to_string(),
        context: format!("Failed to parse crypto snapshot response: {e}"),
    })
}

/// Fetch top gainers or losers for crypto.
///
/// * `direction` - `"gainers"` or `"losers"`
pub async fn crypto_top_movers(direction: &str) -> Result<SnapshotsResponse> {
    let client = build_client()?;
    let path = format!("/v2/snapshot/locale/global/markets/crypto/{}", direction);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "crypto_top_movers".to_string(),
        context: format!("Failed to parse crypto top movers response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crypto_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/snapshot/locale/global/markets/crypto/tickers/X:BTCUSD",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "request_id": "abc123",
                    "status": "OK",
                    "ticker": {
                        "ticker": "X:BTCUSD",
                        "todaysChange": 1200.0,
                        "todaysChangePerc": 2.85,
                        "updated": 1705363200000000000_i64,
                        "day": { "o": 42000.0, "h": 43500.0, "l": 41800.0, "c": 43200.0, "v": 12345.67 },
                        "prevDay": { "o": 41500.0, "h": 42200.0, "l": 41000.0, "c": 42000.0, "v": 11000.0 },
                        "lastTrade": { "price": 43200.0, "size": 0.25, "exchange": 2, "sip_timestamp": 1705363200000000000_i64 },
                        "min": { "o": 43100.0, "h": 43250.0, "l": 43050.0, "c": 43200.0, "v": 50.5 }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/snapshot/locale/global/markets/crypto/tickers/X:BTCUSD",
                &[],
            )
            .await
            .unwrap();

        let resp: SingleSnapshotResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.ticker.unwrap();
        assert_eq!(snap.ticker.as_deref(), Some("X:BTCUSD"));
        assert!((snap.todays_change.unwrap() - 1200.0).abs() < 0.01);
    }
}
