//! Kraken public market-data HTTP client.
//!
//! `api.kraken.com/0/public/*` needs no key and is reachable from the US,
//! which is the reason this provider exists alongside Binance.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tracing::debug;

use super::models::{
    KrakenCandle, KrakenEnvelope, KrakenTicker, OhlcEntry, OhlcResult, TickerResult,
};
use crate::adapters::common::{check_status, keyless_http_client};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const KRAKEN_BASE: &str = "https://api.kraken.com/0/public";

pub(super) struct KrakenClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl KrakenClient {
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

    /// Fetch the ticker for one pair.
    ///
    /// Kraken keys the result by *its own* pair name, which often differs from
    /// the one requested (`XBTUSD` comes back as `XXBTZUSD`), so the single
    /// entry is taken positionally rather than looked up.
    pub(super) async fn ticker(&self, pair: &str) -> Result<KrakenTicker> {
        let result: TickerResult = self.get("Ticker", &[("pair", pair)], pair).await?;
        result
            .into_values()
            .next()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(pair.to_string()),
                context: "Kraken returned no ticker for this pair".to_string(),
            })
    }

    /// Fetch OHLC candles for one pair at `interval_minutes`, starting after
    /// `since` (unix seconds).
    pub(super) async fn ohlc(
        &self,
        pair: &str,
        interval_minutes: u32,
        since: i64,
    ) -> Result<Vec<KrakenCandle>> {
        let interval = interval_minutes.to_string();
        let since = since.max(0).to_string();
        let result: OhlcResult = self
            .get(
                "OHLC",
                &[("pair", pair), ("interval", &interval), ("since", &since)],
                pair,
            )
            .await?;

        // The map holds one market plus a `"last"` cursor; take the candles.
        result
            .into_values()
            .find_map(|entry| match entry {
                OhlcEntry::Candles(candles) => Some(candles),
                OhlcEntry::Cursor(_) => None,
            })
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(pair.to_string()),
                context: "Kraken returned no candles for this pair".to_string(),
            })
    }

    /// Issue a public GET and unwrap Kraken's `{error, result}` envelope.
    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        pair: &str,
    ) -> Result<T> {
        self.limiter.acquire().await;
        let url = format!("{}/{endpoint}", self.base_url);
        debug!("Kraken request: {endpoint} {pair}");

        let resp = self.http.get(&url).query(params).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        check_status("Kraken", status)?;

        let envelope: KrakenEnvelope<T> =
            serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
                field: format!("kraken.{endpoint}"),
                context: format!("unrecognised Kraken envelope: {e}"),
            })?;

        // Kraken answers a rejected request with HTTP 200 and a populated
        // `error` array, so the status code alone never means success.
        if !envelope.error.is_empty() {
            let detail = envelope.error.join("; ");
            return Err(if detail.contains("Unknown asset pair") {
                FinanceError::SymbolNotFound {
                    symbol: Some(pair.to_string()),
                    context: detail,
                }
            } else if detail.contains("Rate limit") {
                FinanceError::RateLimited { retry_after: None }
            } else {
                FinanceError::ApiError(format!("Kraken: {detail}"))
            });
        }

        envelope
            .result
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: format!("kraken.{endpoint}.result"),
                context: "Kraken reported no error but returned no result".to_string(),
            })
    }
}
