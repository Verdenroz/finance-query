//! Binance public market data — keyless exchange-grade crypto.
//!
//! Requires the **`binance`** feature flag.
//!
//! Talks to `data-api.binance.vision`, the market-data-only host: same public
//! endpoints as `api.binance.com`, but no account or trading routes exist
//! there, so there is no key to configure and none to leak.
//!
//! Serves two capabilities:
//! - `CRYPTO` — rolling 24-hour quote per spot market.
//! - `CHART` — arbitrary-interval OHLCV klines, which CoinGecko cannot give
//!   and the keyed providers charge for.
//!
//! # Regional availability
//!
//! Binance geo-blocks some regions (notably US retail) with HTTP 451. That is
//! why Kraken exists as a sibling route; a `Fetch::Sequential` chain of the
//! two covers both.

pub(crate) mod chart;
pub(crate) mod client;
pub(crate) mod crypto;
pub(crate) mod models;
pub(crate) mod symbols;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::BinanceClient;

/// Self-imposed pacing. Binance meters by request weight (6000/minute per IP);
/// the endpoints used here are weight 1–2, so this stays far inside the quota.
const BINANCE_RATE_PER_SEC: f64 = 10.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = BINANCE_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<BinanceClient> {
    BinanceClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::BINANCE_BASE)
}

pub(crate) use chart::{fetch_chart_range_response, fetch_chart_response};
pub(crate) use crypto::fetch_crypto_quote_response;

#[cfg(test)]
mod tests {
    use super::chart::{interval_code, interval_secs, to_candles};
    use super::crypto::to_quote;
    use super::models::Kline;
    use super::*;
    use crate::constants::Interval;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> BinanceClient {
        BinanceClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn ticker_payload() -> String {
        serde_json::json!({
            "symbol": "BTCUSDT",
            "priceChange": "487.60000000",
            "priceChangePercent": "0.776",
            "weightedAvgPrice": "63363.45155704",
            "prevClosePrice": "62830.01000000",
            "lastPrice": "63317.60000000",
            "lastQty": "0.00050000",
            "bidPrice": "63317.60000000",
            "askPrice": "63317.61000000",
            "openPrice": "62830.00000000",
            "highPrice": "63796.33000000",
            "lowPrice": "62830.00000000",
            "volume": "8345.37574000",
            "quoteVolume": "528791811.42675440",
            "openTime": 1785632332001_i64,
            "closeTime": 1785718732001_i64,
            "count": 1349944
        })
        .to_string()
    }

    fn kline(open_time: i64, close: f64) -> Kline {
        Kline {
            open_time,
            open: close - 1.0,
            high: close + 2.0,
            low: close - 3.0,
            close,
            volume: 12.75,
        }
    }

    #[test]
    fn every_interval_but_three_months_maps_to_a_binance_code() {
        assert_eq!(interval_code(Interval::OneMinute), Some("1m"));
        assert_eq!(interval_code(Interval::OneHour), Some("1h"));
        assert_eq!(interval_code(Interval::OneDay), Some("1d"));
        assert_eq!(interval_code(Interval::OneWeek), Some("1w"));
        // Binance's month code is uppercase; "1m" is one minute.
        assert_eq!(interval_code(Interval::OneMonth), Some("1M"));
        assert_eq!(interval_code(Interval::ThreeMonths), None);
    }

    #[test]
    fn candles_convert_milliseconds_to_seconds() {
        let candles = to_candles(vec![kline(1_785_628_800_000, 63570.0)]);
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].timestamp, 1_785_628_800);
        assert_eq!(candles[0].close, 63570.0);
        // Crypto has no corporate actions, so adj_close mirrors close.
        assert_eq!(candles[0].adj_close, Some(63570.0));
        assert_eq!(candles[0].provider_id, Some(crate::Provider::Binance));
    }

    #[test]
    fn interval_seconds_match_the_interval_codes() {
        assert_eq!(interval_secs(Interval::OneMinute), 60);
        assert_eq!(interval_secs(Interval::OneDay), 86_400);
        assert_eq!(interval_secs(Interval::OneWeek), 604_800);
    }

    #[tokio::test]
    async fn ticker_maps_to_a_canonical_crypto_quote() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v3/ticker/24hr")
            .match_query(mockito::Matcher::UrlEncoded(
                "symbol".into(),
                "BTCUSDT".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(ticker_payload())
            .create_async()
            .await;

        let raw = test_client(&server.url())
            .ticker_24hr("BTCUSDT")
            .await
            .unwrap();
        let quote = to_quote("bitcoin", raw);

        assert_eq!(quote.id, "bitcoin");
        assert_eq!(quote.symbol, "BTC");
        assert_eq!(quote.name, "Bitcoin");
        assert_eq!(quote.price, Some(63317.6));
        assert_eq!(quote.change_24h, Some(487.6));
        assert_eq!(quote.change_percent_24h, Some(0.776));
        assert_eq!(quote.high_24h, Some(63796.33));
        assert_eq!(quote.low_24h, Some(62830.0));
        // Quote-asset volume, the figure comparable to other providers.
        assert_eq!(quote.volume_24h, Some(528791811.4267544));
        // An exchange knows flow, not supply.
        assert_eq!(quote.market_cap, None);
        assert_eq!(quote.circulating_supply, None);
    }

    #[tokio::test]
    async fn unlisted_market_maps_to_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v3/ticker/24hr")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "code": -1121, "msg": "Invalid symbol." }).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker_24hr("NOTAMARKET")
            .await
            .unwrap_err();
        match err {
            FinanceError::SymbolNotFound { context, .. } => {
                assert!(context.contains("Invalid symbol"), "got {context}");
            }
            other => panic!("expected SymbolNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn geo_block_explains_itself() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v3/ticker/24hr")
            .match_query(mockito::Matcher::Any)
            .with_status(451)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker_24hr("BTCUSDT")
            .await
            .unwrap_err();
        match err {
            FinanceError::ApiError(msg) => {
                assert!(
                    msg.contains("Kraken"),
                    "the error should name a way out: {msg}"
                );
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ban_status_maps_to_rate_limited() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v3/ticker/24hr")
            .match_query(mockito::Matcher::Any)
            // 418: Binance's "you ignored a 429".
            .with_status(418)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .ticker_24hr("BTCUSDT")
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::RateLimited { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn klines_parse_from_the_wire_shape() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/api/v3/klines")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[[1785628800000,"62823.65000000","63796.33000000","62806.58000000","63570.00000000","8387.01154000",1785715199999,"531238376.42127370",1286064,"4566.33112000","289282425.13673770","0"]]"#,
            )
            .create_async()
            .await;

        let klines = test_client(&server.url())
            .klines("BTCUSDT", "1d", 0, 1785715199999, 1000)
            .await
            .unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].close, 63570.0);
    }
}
