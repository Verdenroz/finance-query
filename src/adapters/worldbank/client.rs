//! World Bank Open Data HTTP client.
//!
//! Keyless and unauthenticated. The World Bank publishes no hard rate limit,
//! so the client paces itself conservatively rather than relying on the
//! service to push back.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{WorldBankObservation, WorldBankResponse};
use crate::adapters::common::keyless_http_client;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const WORLDBANK_BASE: &str = "https://api.worldbank.org/v2";

/// One page big enough for any single-country indicator series (the longest
/// run to date is ~65 annual observations), so pagination never kicks in.
const PER_PAGE: u32 = 20_000;

pub(super) struct WorldBankClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl WorldBankClient {
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

    /// Fetch every observation of `indicator` for `country`, newest first (the
    /// order the API returns).
    ///
    /// `country` is an ISO-2/ISO-3 code, an aggregate code such as `WLD` or
    /// `EMU`, or `all`.
    pub(super) async fn indicator(
        &self,
        country: &str,
        indicator: &str,
    ) -> Result<Vec<WorldBankObservation>> {
        self.limiter.acquire().await;

        let url = format!(
            "{}/country/{}/indicator/{}",
            self.base_url,
            crate::adapters::common::encode_path_segment(country),
            crate::adapters::common::encode_path_segment(indicator),
        );
        debug!("World Bank request: {url}");

        let resp = self
            .http
            .get(&url)
            .query(&[
                ("format", "json"),
                ("per_page", &PER_PAGE.to_string()),
                ("page", "1"),
            ])
            .send()
            .await?;
        Self::check_status(resp.status())?;

        let bytes = resp.bytes().await?;
        let parsed: WorldBankResponse =
            serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
                field: "worldbank.response".to_string(),
                context: format!("unrecognised World Bank envelope: {e}"),
            })?;

        if let Some(messages) = parsed.0.message {
            let detail = messages
                .iter()
                .map(|m| m.describe())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(FinanceError::MacroDataError {
                provider: "World Bank".to_string(),
                context: format!("{country}/{indicator}: {detail}"),
            });
        }

        let observations = parsed.1.unwrap_or_default();
        if observations.is_empty() {
            return Err(FinanceError::SymbolNotFound {
                symbol: Some(format!("{country}/{indicator}")),
                context: "World Bank returned no observations for this country/indicator pair"
                    .to_string(),
            });
        }
        Ok(observations)
    }

    fn check_status(status: StatusCode) -> Result<()> {
        match status {
            s if s.is_success() => Ok(()),
            StatusCode::TOO_MANY_REQUESTS => Err(FinanceError::RateLimited { retry_after: None }),
            s => Err(FinanceError::ExternalApiError {
                api: "World Bank".to_string(),
                status: s.as_u16(),
            }),
        }
    }
}
