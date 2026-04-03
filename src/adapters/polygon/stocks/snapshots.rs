//! Stock snapshot endpoints: full market, single ticker, top movers.

use crate::error::{FinanceError, Result};

use super::super::build_client;
use super::super::models::*;

/// Fetch snapshot for a single stock ticker.
pub async fn stock_snapshot(ticker: &str) -> Result<SingleSnapshotResponse> {
    let client = build_client()?;
    let path = format!("/v2/snapshot/locale/us/markets/stocks/tickers/{}", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "snapshot".to_string(),
        context: format!("Failed to parse snapshot response: {e}"),
    })
}

/// Fetch snapshots for all US stock tickers.
///
/// * `tickers` - Optional comma-separated list of tickers to filter
pub async fn stock_snapshots_all(tickers: Option<&str>) -> Result<SnapshotsResponse> {
    let client = build_client()?;
    let path = "/v2/snapshot/locale/us/markets/stocks/tickers";
    let params: Vec<(&str, &str)> = match tickers {
        Some(t) => vec![("tickers", t)],
        None => vec![],
    };
    let json = client.get_raw(path, &params).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "snapshots".to_string(),
        context: format!("Failed to parse snapshots response: {e}"),
    })
}

/// Fetch top gainers or losers snapshot.
///
/// * `direction` - `"gainers"` or `"losers"`
pub async fn stock_top_movers(direction: &str) -> Result<SnapshotsResponse> {
    let client = build_client()?;
    let path = format!("/v2/snapshot/locale/us/markets/stocks/{}", direction);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| FinanceError::ResponseStructureError {
        field: "top_movers".to_string(),
        context: format!("Failed to parse top movers response: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stock_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/snapshot/locale/us/markets/stocks/tickers/AAPL")
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
                        "ticker": "AAPL",
                        "todaysChange": 1.5,
                        "todaysChangePerc": 0.81,
                        "updated": 1705363200000000000_i64,
                        "day": { "o": 185.09, "h": 187.01, "l": 184.35, "c": 186.19, "v": 65076600.0, "vw": 185.87 },
                        "prevDay": { "o": 184.0, "h": 185.5, "l": 183.5, "c": 184.69, "v": 55000000.0 },
                        "lastTrade": { "price": 186.19, "size": 100.0, "exchange": 4, "sip_timestamp": 1705363200000000000_i64 },
                        "lastQuote": { "bid_price": 186.18, "ask_price": 186.25, "bid_size": 2.0, "ask_size": 3.0 },
                        "min": { "o": 186.0, "h": 186.25, "l": 185.90, "c": 186.19, "v": 1500000.0 }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/snapshot/locale/us/markets/stocks/tickers/AAPL", &[])
            .await
            .unwrap();

        let resp: SingleSnapshotResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.ticker.unwrap();
        assert_eq!(snap.ticker.as_deref(), Some("AAPL"));
        assert!((snap.todays_change.unwrap() - 1.5).abs() < 0.01);

        let day = snap.day.unwrap();
        assert!((day.open.unwrap() - 185.09).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_top_movers_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v2/snapshot/locale/us/markets/stocks/gainers")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apiKey".into(),
                "test-key".into(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "tickers": [
                        {
                            "ticker": "XYZ",
                            "todaysChange": 5.0,
                            "todaysChangePerc": 15.5,
                            "day": { "o": 30.0, "h": 38.0, "l": 29.5, "c": 37.2, "v": 10000000.0 }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/snapshot/locale/us/markets/stocks/gainers", &[])
            .await
            .unwrap();

        let resp: SnapshotsResponse = serde_json::from_value(json).unwrap();
        let tickers = resp.tickers.unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].ticker.as_deref(), Some("XYZ"));
        assert!((tickers[0].todays_change_perc.unwrap() - 15.5).abs() < 0.01);
    }
}
