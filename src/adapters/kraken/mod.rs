//! Kraken public market data — keyless crypto, reachable from the US.
//!
//! Requires the **`kraken`** feature flag.
//!
//! The US-accessible complement to the Binance adapter: `api.kraken.com`'s
//! public endpoints need no key and impose no geo-block, so a
//! `Fetch::Sequential` chain of the two gives `CRYPTO` a fallback that works
//! everywhere.
//!
//! Serves `CRYPTO` (24-hour ticker) and `CHART` (OHLC candles).
//!
//! # Kraken's own conventions
//!
//! Bitcoin is `XBT`, Dogecoin is `XDG`, and older pairs carry `X`/`Z` asset
//! class prefixes (`XXBTZUSD`). All of that is translated in
//! [`symbols`], so callers pass normal tickers.

pub(crate) mod chart;
pub(crate) mod client;
pub(crate) mod crypto;
pub(crate) mod models;
pub(crate) mod symbols;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::KrakenClient;

/// Self-imposed pacing. Kraken's public counter allows roughly one call per
/// second sustained for unauthenticated clients.
const KRAKEN_RATE_PER_SEC: f64 = 1.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = KRAKEN_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<KrakenClient> {
    KrakenClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::KRAKEN_BASE)
}

pub(crate) use chart::{fetch_chart_range_response, fetch_chart_response};
pub(crate) use crypto::fetch_crypto_quote_response;

#[cfg(test)]
mod tests {
    use super::chart::{interval_minutes, to_candles};
    use super::crypto::to_quote;
    use super::models::KrakenCandle;
    use super::*;
    use crate::constants::Interval;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> KrakenClient {
        KrakenClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn ticker_payload() -> String {
        // Verbatim shape from api.kraken.com, including the legacy pair key.
        serde_json::json!({
            "error": [],
            "result": {
                "XXBTZUSD": {
                    "a": ["63243.80000", "1", "1.000"],
                    "b": ["63243.70000", "1", "1.000"],
                    "c": ["63243.80000", "0.00092800"],
                    "v": ["55.74378179", "810.57235439"],
                    "p": ["63264.56913", "63240.68062"],
                    "t": [2105, 37019],
                    "l": ["63150.00000", "62759.30000"],
                    "h": ["63513.40000", "63712.10000"],
                    "o": "63500.00000"
                }
            }
        })
        .to_string()
    }

    fn ohlc_payload() -> String {
        serde_json::json!({
            "error": [],
            "result": {
                "XXBTZUSD": [
                    [1723507200, "59359.5", "61555.0", "58441.2", "60600.0", "60028.1",
                     "2442.99193825", 32890],
                    [1723593600, "60600.1", "61773.4", "58500.0", "58693.6", "60047.6",
                     "2297.26406239", 28085]
                ],
                "last": 1723593600_i64
            }
        })
        .to_string()
    }

    #[test]
    fn every_interval_but_three_months_maps_to_kraken_minutes() {
        assert_eq!(interval_minutes(Interval::OneMinute), Some(1));
        assert_eq!(interval_minutes(Interval::OneHour), Some(60));
        assert_eq!(interval_minutes(Interval::OneDay), Some(1_440));
        assert_eq!(interval_minutes(Interval::OneWeek), Some(10_080));
        assert_eq!(interval_minutes(Interval::OneMonth), Some(21_600));
        assert_eq!(interval_minutes(Interval::ThreeMonths), None);
    }

    #[test]
    fn kraken_rejects_intervals_outside_its_fixed_bucket_set() {
        assert_eq!(interval_minutes(Interval::TwoMinutes), None);
        assert_eq!(interval_minutes(Interval::NinetyMinutes), None);
        assert_eq!(interval_minutes(Interval::FiveDays), None);
    }

    #[test]
    fn candles_keep_kraken_second_timestamps() {
        let candles = to_candles(vec![KrakenCandle {
            time: 1_723_507_200,
            open: 59359.5,
            high: 61555.0,
            low: 58441.2,
            close: 60600.0,
            volume: 2442.99,
        }]);
        assert_eq!(candles[0].timestamp, 1_723_507_200);
        assert_eq!(candles[0].close, 60600.0);
        assert_eq!(candles[0].adj_close, Some(60600.0));
        assert_eq!(candles[0].provider_id, Some(crate::Provider::Kraken));
    }

    #[tokio::test]
    async fn ticker_maps_to_a_canonical_quote_using_rolling_24h_fields() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/Ticker")
            .match_query(mockito::Matcher::UrlEncoded("pair".into(), "XBTUSD".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticker_payload())
            .create_async()
            .await;

        let raw = test_client(&server.url()).ticker("XBTUSD").await.unwrap();
        let quote = to_quote("bitcoin", "BTC", &raw);

        assert_eq!(quote.id, "bitcoin");
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.name, "Bitcoin");
        assert_eq!(quote.price, Some(63243.8));
        // Index 1 of each packed array is the rolling 24-hour figure, not today's.
        assert_eq!(quote.volume_24h, Some(810.57235439));
        assert_eq!(quote.high_24h, Some(63712.1));
        assert_eq!(quote.low_24h, Some(62759.3));
        // Change is against today's open — the only reference Kraken publishes.
        assert_eq!(quote.change_24h, Some(63243.8 - 63500.0));
        assert_eq!(quote.market_cap, None);
    }

    #[tokio::test]
    async fn ohlc_candles_are_read_past_the_last_cursor() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/OHLC")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ohlc_payload())
            .create_async()
            .await;

        // The result map holds the candles *and* an integer "last" cursor;
        // typing it as one map of candle arrays would fail to parse.
        let candles = test_client(&server.url())
            .ohlc("XBTUSD", 1440, 0)
            .await
            .unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].time, 1_723_507_200);
        assert_eq!(candles[1].close, 58693.6);
    }

    #[tokio::test]
    async fn unknown_pair_maps_to_symbol_not_found_despite_http_200() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/Ticker")
            .match_query(mockito::Matcher::Any)
            // Kraken reports errors with a 200 and a populated `error` array.
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "error": ["EQuery:Unknown asset pair"] }).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker("NOTAPAIR")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn other_api_errors_carry_krakens_own_text() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/Ticker")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "error": ["EService:Unavailable"] }).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker("XBTUSD")
            .await
            .unwrap_err();
        match err {
            FinanceError::ApiError(msg) => assert!(msg.contains("EService"), "got {msg}"),
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/Ticker")
            .match_query(mockito::Matcher::Any)
            .with_status(520)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker("XBTUSD")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 520, .. }),
            "{err:?}"
        );
    }
}
