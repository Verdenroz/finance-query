//! Forex snapshot endpoints: all tickers, single ticker, top movers.
#![allow(dead_code)]

use crate::adapters::common::encode_path_segment;
use crate::error::Result;

use super::super::build_client;
use super::super::models::*;

/// Fetch snapshots for all forex tickers.
///
/// * `tickers` - Optional comma-separated list of tickers to filter
pub async fn forex_snapshots_all(tickers: Option<&str>) -> Result<SnapshotsResponseDTO> {
    let client = build_client()?;
    let path = "/v2/snapshot/locale/global/markets/forex/tickers";
    let params: Vec<(&str, &str)> = match tickers {
        Some(t) => vec![("tickers", t)],
        None => vec![],
    };
    client
        .get_as(path, &params, "forex_snapshots", "forex snapshots response")
        .await
}

/// Fetch snapshot for a single forex ticker.
///
/// * `ticker` - Forex ticker symbol with `C:` prefix (e.g., `"C:EURUSD"`)
pub async fn forex_snapshot(ticker: &str) -> Result<SingleSnapshotResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/global/markets/forex/tickers/{}",
        ticker
    );
    client
        .get_as(&path, &[], "forex_snapshot", "forex snapshot response")
        .await
}

/// Fetch top forex movers (gainers or losers).
///
/// * `direction` - `"gainers"` or `"losers"`
pub async fn forex_top_movers(direction: &str) -> Result<SnapshotsResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/global/markets/forex/{}",
        encode_path_segment(direction)
    );
    client
        .get_as(&path, &[], "forex_top_movers", "forex top movers response")
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_forex_snapshot_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/snapshot/locale/global/markets/forex/tickers/C:EURUSD",
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
                        "ticker": "C:EURUSD",
                        "todaysChange": 0.0025,
                        "todaysChangePerc": 0.23,
                        "updated": 1705363200000000000_i64,
                        "day": { "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0, "vw": 1.1060 },
                        "prevDay": { "o": 1.1030, "h": 1.1070, "l": 1.1000, "c": 1.1055, "v": 48000.0 },
                        "lastQuote": { "bid_price": 1.1078, "ask_price": 1.1082, "bid_size": 1000.0, "ask_size": 1500.0 },
                        "min": { "o": 1.1075, "h": 1.1082, "l": 1.1070, "c": 1.1080, "v": 5000.0 }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/snapshot/locale/global/markets/forex/tickers/C:EURUSD",
                &[],
            )
            .await
            .unwrap();

        let resp: SingleSnapshotResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.ticker.unwrap();
        assert_eq!(snap.ticker.as_deref(), Some("C:EURUSD"));
        assert!((snap.todays_change.unwrap() - 0.0025).abs() < 0.0001);

        let day = snap.day.unwrap();
        assert!((day.open.unwrap() - 1.1050).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_forex_top_movers_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/snapshot/locale/global/markets/forex/gainers",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "tickers": [
                        {
                            "ticker": "C:USDJPY",
                            "todaysChange": 1.25,
                            "todaysChangePerc": 0.85,
                            "day": { "o": 148.50, "h": 150.00, "l": 148.20, "c": 149.75, "v": 100000.0 }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/snapshot/locale/global/markets/forex/gainers", &[])
            .await
            .unwrap();

        let resp: SnapshotsResponseDTO = serde_json::from_value(json).unwrap();
        let tickers = resp.tickers.unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].ticker.as_deref(), Some("C:USDJPY"));
        assert!((tickers[0].todays_change_perc.unwrap() - 0.85).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_forex_snapshots_all_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/v2/snapshot/locale/global/markets/forex/tickers",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded(
                    "tickers".into(),
                    "C:EURUSD,C:GBPUSD".into(),
                ),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "OK",
                    "count": 2,
                    "tickers": [
                        {
                            "ticker": "C:EURUSD",
                            "todaysChange": 0.0025,
                            "todaysChangePerc": 0.23,
                            "day": { "o": 1.1050, "h": 1.1100, "l": 1.1020, "c": 1.1080, "v": 50000.0 }
                        },
                        {
                            "ticker": "C:GBPUSD",
                            "todaysChange": 0.0015,
                            "todaysChangePerc": 0.12,
                            "day": { "o": 1.2700, "h": 1.2750, "l": 1.2680, "c": 1.2730, "v": 30000.0 }
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw(
                "/v2/snapshot/locale/global/markets/forex/tickers",
                &[("tickers", "C:EURUSD,C:GBPUSD")],
            )
            .await
            .unwrap();

        let resp: SnapshotsResponseDTO = serde_json::from_value(json).unwrap();
        let tickers = resp.tickers.unwrap();
        assert_eq!(tickers.len(), 2);
        assert_eq!(tickers[0].ticker.as_deref(), Some("C:EURUSD"));
        assert_eq!(tickers[1].ticker.as_deref(), Some("C:GBPUSD"));
    }
}
