//! DefiLlama HTTP client.
//!
//! Keyless and unauthenticated. TVL data lives on `api.llama.fi`; stablecoin
//! supplies live on a separate host, so both bases are configurable.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{ChainResponse, ProtocolResponse, StablecoinsResponse};
use crate::adapters::common::keyless_http_client;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const LLAMA_BASE: &str = "https://api.llama.fi";
pub(super) const STABLECOINS_BASE: &str = "https://stablecoins.llama.fi";

pub(super) struct DefiLlamaClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
    stablecoins_url: String,
}

impl DefiLlamaClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
        stablecoins_url: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
            stablecoins_url: stablecoins_url.into(),
        })
    }

    /// Fetch a protocol's metadata and full TVL history.
    pub(super) async fn protocol(&self, slug: &str) -> Result<ProtocolResponse> {
        let url = format!(
            "{}/protocol/{}",
            self.base_url,
            crate::adapters::common::encode_path_segment(slug)
        );
        self.get(&url, Some(slug)).await
    }

    /// Fetch aggregate TVL for every chain.
    pub(super) async fn chains(&self) -> Result<Vec<ChainResponse>> {
        let url = format!("{}/v2/chains", self.base_url);
        self.get(&url, None).await
    }

    /// Fetch circulating supply for every tracked stablecoin.
    pub(super) async fn stablecoins(&self) -> Result<StablecoinsResponse> {
        let url = format!("{}/stablecoins?includePrices=false", self.stablecoins_url);
        self.get(&url, None).await
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        subject: Option<&str>,
    ) -> Result<T> {
        self.limiter.acquire().await;
        debug!("DefiLlama request: {url}");

        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            return Err(match status {
                // DefiLlama answers an unknown protocol slug with a 400 and a
                // plain-text body, not a structured error.
                StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => match subject {
                    Some(slug) => FinanceError::SymbolNotFound {
                        symbol: Some(slug.to_string()),
                        context: "DefiLlama knows no protocol by this slug".to_string(),
                    },
                    None => FinanceError::ExternalApiError {
                        api: "DefiLlama".to_string(),
                        status: status.as_u16(),
                    },
                },
                StatusCode::TOO_MANY_REQUESTS => FinanceError::RateLimited { retry_after: None },
                s => FinanceError::ExternalApiError {
                    api: "DefiLlama".to_string(),
                    status: s.as_u16(),
                },
            });
        }

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "defillama.response".to_string(),
            context: format!("unrecognised DefiLlama payload: {e}"),
        })
    }
}
