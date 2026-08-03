//! Frankfurter HTTP client — keyless ECB reference rates.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{FrankfurterError, FrankfurterTimeSeries};
use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const FRANKFURTER_BASE: &str = "https://api.frankfurter.dev/v1";

/// Calendar days of history requested to find the latest rate *and* the one
/// before it in a single call. Wide enough to clear a long bank holiday
/// weekend — the ECB does not publish on non-working days.
const LOOKBACK_DAYS: i64 = 10;

pub(super) struct FrankfurterClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl FrankfurterClient {
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

    /// Fetch the recent published rates for `base`→`quote`, oldest first.
    ///
    /// One open-ended range request rather than a `latest` call, so the
    /// previous close needed for the day's change comes back in the same
    /// round trip.
    pub(super) async fn recent_rates(&self, base: &str, quote: &str) -> Result<Vec<(String, f64)>> {
        self.limiter.acquire().await;

        let start = (chrono::Utc::now() - chrono::Duration::days(LOOKBACK_DAYS))
            .format("%Y-%m-%d")
            .to_string();
        let url = format!("{}/{start}..", self.base_url);
        debug!("Frankfurter request: {url} ({base}->{quote})");

        let resp = self
            .http
            .get(&url)
            .query(&[("base", base), ("symbols", quote)])
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            return Err(Self::map_error(status, &bytes, base, quote));
        }

        let parsed: FrankfurterTimeSeries =
            serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
                field: "frankfurter.rates".to_string(),
                context: format!("unrecognised Frankfurter envelope: {e}"),
            })?;

        // `BTreeMap` keys are ISO dates, so iteration is already chronological.
        let series: Vec<(String, f64)> = parsed
            .rates
            .into_iter()
            .filter_map(|(date, day)| day.get(quote).map(|rate| (date, *rate)))
            .collect();

        if series.is_empty() {
            return Err(FinanceError::SymbolNotFound {
                symbol: Some(format!("{base}{quote}")),
                context: format!(
                    "Frankfurter published no {} rate against {} in the last {LOOKBACK_DAYS} days",
                    quote, parsed.base
                ),
            });
        }
        Ok(series)
    }

    /// Map a non-2xx response, preferring Frankfurter's own explanation.
    ///
    /// It distinguishes an unknown currency (404) from a structurally invalid
    /// pair (422), so the two map to different errors.
    fn map_error(status: StatusCode, body: &[u8], base: &str, quote: &str) -> FinanceError {
        let detail = serde_json::from_slice::<FrankfurterError>(body)
            .ok()
            .and_then(|e| e.message);
        match status {
            StatusCode::NOT_FOUND => FinanceError::SymbolNotFound {
                symbol: Some(format!("{base}{quote}")),
                context: detail.unwrap_or_else(|| {
                    "Frankfurter does not publish one of these currencies".to_string()
                }),
            },
            StatusCode::UNPROCESSABLE_ENTITY => FinanceError::InvalidParameter {
                param: "currency pair".to_string(),
                reason: detail.unwrap_or_else(|| format!("{base}/{quote} is not a valid pair")),
            },
            s => status_error("Frankfurter", s),
        }
    }
}
