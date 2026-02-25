//! CoinGecko API client with rate limiting.
//!
//! Free public API: 30 requests/minute. No API key required.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::CoinQuote;
use super::rate_limiter::RateLimiter;
use crate::error::{FinanceError, Result};

const COINGECKO_BASE: &str = "https://api.coingecko.com/api/v3";
/// 30 req/min = 0.5 req/sec
const COINGECKO_RATE_PER_SEC: f64 = 0.5;

pub(crate) struct CoinGeckoClient {
    http: Client,
    limiter: Arc<RateLimiter>,
}

impl CoinGeckoClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("finance-query/2 (https://github.com/Verdenroz/finance-query)")
            .build()
            .map_err(FinanceError::HttpError)?;

        Ok(Self {
            http,
            limiter: Arc::new(RateLimiter::new(COINGECKO_RATE_PER_SEC)),
        })
    }

    /// Fetch top coins by market cap.
    ///
    /// # Arguments
    ///
    /// * `vs_currency` - Quote currency (e.g., `"usd"`, `"eur"`)
    /// * `count` - Number of coins to return (max 250 per request)
    pub async fn coins(&self, vs_currency: &str, count: usize) -> Result<Vec<CoinQuote>> {
        self.limiter.acquire().await;

        let per_page = count.min(250);
        let url = format!(
            "{COINGECKO_BASE}/coins/markets?vs_currency={vs_currency}&order=market_cap_desc&per_page={per_page}&page=1&sparkline=false"
        );

        debug!("CoinGecko request: coins(vs_currency={vs_currency}, count={count})");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(FinanceError::HttpError)?;
        self.check_status(&resp)?;
        resp.json().await.map_err(FinanceError::HttpError)
    }

    /// Fetch a single coin by its CoinGecko ID (e.g., `"bitcoin"`, `"ethereum"`).
    pub async fn coin(&self, id: &str, vs_currency: &str) -> Result<CoinQuote> {
        self.limiter.acquire().await;

        let url = format!(
            "{COINGECKO_BASE}/coins/markets?vs_currency={vs_currency}&ids={id}&order=market_cap_desc&per_page=1&page=1&sparkline=false"
        );

        debug!("CoinGecko request: coin(id={id})");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(FinanceError::HttpError)?;
        self.check_status(&resp)?;
        let mut list: Vec<CoinQuote> = resp.json().await.map_err(FinanceError::HttpError)?;

        list.pop().ok_or_else(|| FinanceError::SymbolNotFound {
            symbol: Some(id.to_string()),
            context: format!("CoinGecko returned no coin for id '{id}'"),
        })
    }

    fn check_status(&self, resp: &reqwest::Response) -> Result<()> {
        match resp.status() {
            StatusCode::OK => Ok(()),
            StatusCode::TOO_MANY_REQUESTS => Err(FinanceError::RateLimited {
                retry_after: Some(60),
            }),
            s => Err(FinanceError::ExternalApiError {
                api: "CoinGecko".to_string(),
                status: s.as_u16(),
            }),
        }
    }
}
