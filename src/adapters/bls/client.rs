//! BLS Public Data API HTTP client.
//!
//! Dual-mode: with no `BLS_API_KEY` the keyless v1 route is used (25
//! queries/day per IP, ~3 years of history); with a key the v2 route is used
//! (500 queries/day, 20 years, plus series catalog metadata).

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tracing::debug;

use super::models::{BlsResponse, BlsSeries};
use crate::adapters::common::{check_status, keyless_http_client};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const BLS_BASE: &str = "https://api.bls.gov/publicAPI";

/// Years of history requested on the keyed v2 route — its documented maximum
/// per request.
const V2_YEARS: i32 = 20;

/// Which API tier a client talks to, decided by whether a key is configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Tier {
    /// Keyless v1: ~3 years of history, 25 queries/day per IP.
    V1,
    /// Keyed v2: 20 years of history, 500 queries/day, catalog metadata.
    V2(String),
}

impl Tier {
    /// Pick the tier from the environment: a key upgrades to v2.
    pub(super) fn from_env() -> Self {
        match std::env::var("BLS_API_KEY") {
            Ok(key) if !key.trim().is_empty() => Self::V2(key),
            _ => Self::V1,
        }
    }

    pub(super) fn version(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V2(_) => "v2",
        }
    }
}

pub(super) struct BlsClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
    tier: Tier,
}

impl BlsClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
        tier: Tier,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
            tier,
        })
    }

    /// Remove the registration key from a message BLS wrote, which quotes the
    /// submitted key back when it rejects one.
    fn scrub(&self, message: &str) -> String {
        match &self.tier {
            Tier::V1 => message.to_string(),
            Tier::V2(key) => crate::adapters::common::redact_key(message, key),
        }
    }

    /// Fetch one series by its native BLS id (e.g. `"CUUR0000SA0"`).
    ///
    /// Observations come back newest-first, as BLS orders them.
    pub(super) async fn series(&self, series_id: &str) -> Result<BlsSeries> {
        self.limiter.acquire().await;

        let url = match &self.tier {
            // v1 has no request body; the series id is the last path segment.
            Tier::V1 => format!(
                "{}/v1/timeseries/data/{}",
                self.base_url,
                crate::adapters::common::encode_path_segment(series_id)
            ),
            Tier::V2(_) => format!("{}/v2/timeseries/data/", self.base_url),
        };
        debug!("BLS request ({}): {series_id}", self.tier.version());

        let request = match &self.tier {
            Tier::V1 => self.http.get(&url),
            Tier::V2(key) => {
                let end = chrono::Utc::now().format("%Y").to_string();
                let start = (chrono::Utc::now().format("%Y").to_string())
                    .parse::<i32>()
                    .map(|y| (y - V2_YEARS + 1).to_string())
                    .unwrap_or_else(|_| end.clone());
                self.http.post(&url).json(&serde_json::json!({
                    "seriesid": [series_id],
                    "registrationkey": key,
                    "startyear": start,
                    "endyear": end,
                    "catalog": true,
                }))
            }
        };

        let resp = request.send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        check_status("BLS", status)?;

        let parsed: BlsResponse =
            serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
                field: "bls.response".to_string(),
                context: format!("unrecognised BLS envelope: {e}"),
            })?;

        if parsed.status != "REQUEST_SUCCEEDED" {
            let context = self.scrub(&format!("{}: {}", parsed.status, parsed.message.join("; ")));
            let normalized = context.to_ascii_lowercase();
            if normalized.contains("registration key") || normalized.contains("registrationkey") {
                return Err(FinanceError::AuthenticationFailed { context });
            }
            return Err(FinanceError::MacroDataError {
                provider: "BLS".to_string(),
                context,
            });
        }

        let series = parsed
            .results
            .and_then(|r| r.series.into_iter().next())
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "bls.Results.series".to_string(),
                context: "BLS returned no series block".to_string(),
            })?;

        // An unknown series id still reports REQUEST_SUCCEEDED, with the
        // complaint in `message` and an empty data array.
        if series.data.is_empty() {
            return Err(FinanceError::SymbolNotFound {
                symbol: Some(series_id.to_string()),
                context: if parsed.message.is_empty() {
                    "BLS returned no observations for this series".to_string()
                } else {
                    self.scrub(&parsed.message.join("; "))
                },
            });
        }
        Ok(series)
    }
}
