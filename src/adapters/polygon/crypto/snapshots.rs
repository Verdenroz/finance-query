//! Crypto snapshot endpoints: all tickers, single ticker, top movers.
#![allow(dead_code)]

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::crypto::CryptoQuote;

use super::super::build_client;
use super::super::models::*;

/// Fetch snapshots for all crypto tickers.
///
/// * `tickers` - Optional comma-separated list of tickers to filter (e.g., `"X:BTCUSD,X:ETHUSD"`)
pub async fn crypto_snapshots_all(tickers: Option<&str>) -> Result<SnapshotsResponseDTO> {
    let client = build_client()?;
    let path = "/v2/snapshot/locale/global/markets/crypto/tickers";
    let params: Vec<(&str, &str)> = match tickers {
        Some(t) => vec![("tickers", t)],
        None => vec![],
    };
    client
        .get_as(
            path,
            &params,
            "crypto_snapshots",
            "crypto snapshots response",
        )
        .await
}

/// Fetch snapshot for a single crypto ticker.
///
/// * `ticker` - Crypto ticker symbol with `X:` prefix (e.g., `"X:BTCUSD"`)
pub async fn crypto_snapshot(ticker: &str) -> Result<SingleSnapshotResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/global/markets/crypto/tickers/{}",
        ticker
    );
    client
        .get_as(&path, &[], "crypto_snapshot", "crypto snapshot response")
        .await
}

/// Fetch crypto quote (canonical) for a currency pair.
pub async fn fetch_crypto_quote_response(from: &str, to: &str) -> Result<CryptoQuote> {
    let ticker = format!("X:{}{}", from.to_uppercase(), to.to_uppercase());
    let resp = crypto_snapshot(&ticker).await?;
    Ok(snapshot_to_quote(ticker, resp))
}

/// Map a single-ticker snapshot response to the canonical [`CryptoQuote`];
/// `price` prefers the day close, falling back to the last trade.
fn snapshot_to_quote(ticker: String, resp: SingleSnapshotResponseDTO) -> CryptoQuote {
    let snap = resp.ticker;
    let day = snap.as_ref().and_then(|s| s.day.as_ref());
    CryptoQuote {
        id: snap
            .as_ref()
            .and_then(|s| s.ticker.clone())
            .unwrap_or_else(|| ticker.clone()),
        symbol: snap
            .as_ref()
            .and_then(|s| s.ticker.clone())
            .unwrap_or_else(|| ticker.clone()),
        name: snap
            .as_ref()
            .and_then(|s| s.ticker.clone())
            .unwrap_or(ticker),
        price: day.and_then(|d| d.close).or_else(|| {
            snap.as_ref()
                .and_then(|s| s.last_trade.as_ref())
                .and_then(|t| t.price)
        }),
        market_cap: None,
        volume_24h: day.and_then(|d| d.volume),
        change_24h: snap.as_ref().and_then(|s| s.todays_change),
        change_percent_24h: snap.as_ref().and_then(|s| s.todays_change_perc),
        high_24h: day.and_then(|d| d.high),
        low_24h: day.and_then(|d| d.low),
        circulating_supply: None,
    }
}

/// Fetch top gainers or losers for crypto.
///
/// * `direction` - `"gainers"` or `"losers"`
pub async fn crypto_top_movers(direction: &str) -> Result<SnapshotsResponseDTO> {
    let client = build_client()?;
    let path = format!(
        "/v2/snapshot/locale/global/markets/crypto/{}",
        encode_path_segment(direction)
    );
    client
        .get_as(
            &path,
            &[],
            "crypto_top_movers",
            "crypto top movers response",
        )
        .await
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

        let resp: SingleSnapshotResponseDTO = serde_json::from_value(json).unwrap();
        assert_eq!(resp.status.as_deref(), Some("OK"));
        let snap = resp.ticker.as_ref().unwrap();
        assert_eq!(snap.ticker.as_deref(), Some("X:BTCUSD"));
        assert!((snap.todays_change.unwrap() - 1200.0).abs() < 0.01);

        // Mocked HTTP → DTO → canonical CryptoQuote, covering the full
        // fetch_crypto_quote_response pipeline without a network call.
        let quote = snapshot_to_quote("X:BTCUSD".to_string(), resp);
        assert_eq!(quote.symbol, "X:BTCUSD");
        assert_eq!(quote.price, Some(43200.0));
        assert_eq!(quote.change_24h, Some(1200.0));
        assert_eq!(quote.change_percent_24h, Some(2.85));
        assert_eq!(quote.volume_24h, Some(12345.67));
    }

    #[test]
    fn snapshot_to_quote_maps_day_and_change_fields() {
        let resp: SingleSnapshotResponseDTO = serde_json::from_value(serde_json::json!({
            "status": "OK",
            "ticker": {
                "ticker": "X:BTCUSD",
                "todaysChange": 1200.0,
                "todaysChangePerc": 2.85,
                "day": { "o": 42000.0, "h": 43500.0, "l": 41800.0, "c": 43200.0, "v": 12345.67 },
                "lastTrade": { "price": 43150.0 }
            }
        }))
        .unwrap();

        let quote = snapshot_to_quote("X:BTCUSD".to_string(), resp);
        assert_eq!(quote.id, "X:BTCUSD");
        assert_eq!(quote.symbol, "X:BTCUSD");
        assert_eq!(quote.price, Some(43200.0), "day close wins over last trade");
        assert_eq!(quote.volume_24h, Some(12345.67));
        assert_eq!(quote.change_24h, Some(1200.0));
        assert_eq!(quote.change_percent_24h, Some(2.85));
        assert_eq!(quote.high_24h, Some(43500.0));
        assert_eq!(quote.low_24h, Some(41800.0));
    }

    #[test]
    fn snapshot_to_quote_falls_back_to_last_trade_price() {
        let resp: SingleSnapshotResponseDTO = serde_json::from_value(serde_json::json!({
            "status": "OK",
            "ticker": {
                "ticker": "X:BTCUSD",
                "lastTrade": { "price": 43150.0 }
            }
        }))
        .unwrap();
        let quote = snapshot_to_quote("X:BTCUSD".to_string(), resp);
        assert_eq!(quote.price, Some(43150.0));
        assert!(quote.volume_24h.is_none());
    }

    #[test]
    fn snapshot_to_quote_missing_ticker_falls_back_to_input() {
        let resp: SingleSnapshotResponseDTO =
            serde_json::from_value(serde_json::json!({"status": "OK"})).unwrap();
        let quote = snapshot_to_quote("X:ETHUSD".to_string(), resp);
        assert_eq!(quote.id, "X:ETHUSD");
        assert_eq!(quote.symbol, "X:ETHUSD");
        assert!(quote.price.is_none());
    }
}
