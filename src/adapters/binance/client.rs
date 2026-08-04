//! Binance public market-data HTTP client.
//!
//! Talks to `data-api.binance.vision`, the market-data-only host: it serves
//! the same public endpoints as `api.binance.com` but carries no account or
//! trading routes, so no key exists to leak.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{BinanceError, Kline, Ticker24hr};
use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const BINANCE_BASE: &str = "https://data-api.binance.vision";

/// Binance's per-request candle cap.
pub(super) const MAX_KLINES: u32 = 1000;

pub(super) struct BinanceClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl BinanceClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
        })
    }

    /// Fetch rolling 24-hour statistics for one market.
    pub(super) async fn ticker_24hr(&self, symbol: &str) -> Result<Ticker24hr> {
        self.limiter.acquire().await;
        let url = format!("{}/api/v3/ticker/24hr", self.base_url);
        debug!("Binance request: ticker/24hr {symbol}");

        let resp = self
            .http
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await?;
        let bytes = Self::body(resp, symbol).await?;

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "binance.ticker24hr".to_string(),
            context: format!("unrecognised Binance ticker payload: {e}"),
        })
    }

    /// Fetch up to [`MAX_KLINES`] candles for one market starting at
    /// `start_ms`, ending no later than `end_ms`.
    pub(super) async fn klines(
        &self,
        symbol: &str,
        interval: &str,
        start_ms: i64,
        end_ms: i64,
        limit: u32,
    ) -> Result<Vec<Kline>> {
        self.limiter.acquire().await;
        let url = format!("{}/api/v3/klines", self.base_url);
        debug!("Binance request: klines {symbol} {interval} from {start_ms}");

        let start = start_ms.max(0).to_string();
        let end = end_ms.to_string();
        let limit = limit.clamp(1, MAX_KLINES).to_string();
        let resp = self
            .http
            .get(&url)
            .query(&[
                ("symbol", symbol),
                ("interval", interval),
                ("startTime", &start),
                ("endTime", &end),
                ("limit", &limit),
            ])
            .send()
            .await?;
        let bytes = Self::body(resp, symbol).await?;

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "binance.klines".to_string(),
            context: format!("unrecognised Binance kline payload: {e}"),
        })
    }

    /// Read a response body, turning a non-2xx into the right error.
    async fn body(resp: reqwest::Response, symbol: &str) -> Result<Vec<u8>> {
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if status.is_success() {
            return Ok(bytes.to_vec());
        }

        let detail = serde_json::from_slice::<BinanceError>(&bytes).ok();
        Err(match status {
            // Binance answers an unlisted market with 400 / code -1121.
            StatusCode::BAD_REQUEST => FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: detail
                    .and_then(|e| e.msg)
                    .unwrap_or_else(|| "Binance rejected this market symbol".to_string()),
            },
            // 418 is Binance's "you ignored a 429 and are now banned"; 429
            // itself falls through to the shared mapping below.
            StatusCode::IM_A_TEAPOT => FinanceError::RateLimited { retry_after: None },
            // 451 is the geo-block: Binance restricts some regions (US retail).
            StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => FinanceError::ApiError(
                "Binance is not available from this region; route CRYPTO to Kraken or CoinGecko"
                    .to_string(),
            ),
            s => status_error("Binance", s),
        })
    }
}
