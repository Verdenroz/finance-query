//! Stock snapshot endpoints: full market, single ticker, top movers.

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::quote::{FormattedValue, Price, QuoteSummaryResponse};

use super::build_client;
use super::models::*;

#[allow(dead_code)] // unrouted: tick-level trades land with #250
pub mod trades;
pub mod unified;

/// Fetch snapshot for a single stock ticker.
pub async fn stock_snapshot(ticker: &str) -> Result<SingleSnapshotResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/us/markets/stocks/tickers/{}",
        encode_path_segment(ticker)
    );
    client
        .get_as(&path, &[], "snapshot", "snapshot response")
        .await
}

/// Build a canonical quote response from one ticker snapshot.
///
/// Shared by the single-ticker and batch paths so both surface identical fields.
fn snapshot_to_canonical(symbol: &str, snap: Option<&TickerSnapshotDTO>) -> QuoteSummaryResponse {
    let day = snap.and_then(|t| t.day.as_ref());
    let price = Price {
        regular_market_price: day.and_then(|d| d.close).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_open: day.and_then(|d| d.open).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_day_high: day.and_then(|d| d.high).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_day_low: day.and_then(|d| d.low).map(|v| FormattedValue {
            raw: Some(v),
            fmt: None,
            long_fmt: None,
        }),
        regular_market_volume: day.and_then(|d| d.volume).map(|v| FormattedValue {
            raw: Some(v as i64),
            fmt: None,
            long_fmt: None,
        }),
        ..Default::default()
    };
    QuoteSummaryResponse {
        symbol: symbol.to_string(),
        price: Some(price),
        ..Default::default()
    }
}

/// Fetch quote summary response (canonical) for a stock ticker.
pub async fn fetch_quote_response(symbol: &str) -> Result<QuoteSummaryResponse> {
    let snap = stock_snapshot(symbol).await?;
    Ok(snapshot_to_canonical(symbol, snap.ticker.as_ref()))
}

/// Fetch canonical quotes for many tickers in one snapshot request.
///
/// Polygon's snapshot endpoint takes a comma-separated `tickers` filter, so an
/// N-symbol batch costs one request instead of N against the 5 req/sec tier.
pub async fn fetch_quotes_batch_response(
    symbols: &[&str],
) -> Result<Vec<(String, QuoteSummaryResponse)>> {
    let joined = symbols.join(",");
    let snaps = stock_snapshots_all(Some(&joined)).await?;
    Ok(snaps
        .tickers
        .unwrap_or_default()
        .iter()
        .filter_map(|t| {
            let symbol = t.ticker.clone()?;
            let quote = snapshot_to_canonical(&symbol, Some(t));
            Some((symbol, quote))
        })
        .collect())
}

/// Fetch snapshots for all US stock tickers.
///
/// * `tickers` - Optional comma-separated list of tickers to filter
pub async fn stock_snapshots_all(tickers: Option<&str>) -> Result<SnapshotsResponseDTO> {
    let client = build_client()?;
    let path = "/v2/snapshot/locale/us/markets/stocks/tickers";
    let params: Vec<(&str, &str)> = match tickers {
        Some(t) => vec![("tickers", t)],
        None => vec![],
    };
    client
        .get_as(path, &params, "snapshots", "snapshots response")
        .await
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/snapshot/locale/us/markets/stocks/tickers/AAPL", &[])
            .await
            .unwrap();

        let resp: SingleSnapshotResponseDTO = serde_json::from_value(json).unwrap();
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v2/snapshot/locale/us/markets/stocks/gainers", &[])
            .await
            .unwrap();

        let resp: SnapshotsResponseDTO = serde_json::from_value(json).unwrap();
        let tickers = resp.tickers.unwrap();
        assert_eq!(tickers.len(), 1);
        assert_eq!(tickers[0].ticker.as_deref(), Some("XYZ"));
        assert!((tickers[0].todays_change_perc.unwrap() - 15.5).abs() < 0.01);
    }
}
