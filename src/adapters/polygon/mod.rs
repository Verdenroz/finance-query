//! Polygon.io API client for financial data.
//!
//! Requires the **`polygon`** feature flag and an API key from
//! <https://polygon.io/>.
//!
//! Call [`init`] once at startup before using any query functions.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::{Providers, Provider, Capability, Interval, TimeRange};
//! use finance_query::format::Raw;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Route chart and quote data to Polygon with Yahoo as fallback
//! let providers = Providers::builder()
//!     .route(Capability::CHART, &[Provider::Polygon, Provider::Yahoo])
//!     .route(Capability::QUOTE, &[Provider::Polygon, Provider::Yahoo])
//!     .build().await?;
//!
//! let ticker = providers.ticker("AAPL").build().await?;
//! let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
//! let quote = ticker.quote::<Raw>().await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub(crate) mod models;

// Capability-mapped subdirectory modules
mod chart; // CHART
mod corporate; // CORPORATE
mod discovery; // DISCOVERY
mod economic; // ECONOMIC
mod filings; // FILINGS
mod fundamentals; // FUNDAMENTALS
mod market; // MARKET
mod quote; // QUOTE
mod sentiment; // SENTIMENT
mod technicals; // TECHNICALS

// Asset-class subdirectory modules
mod crypto; // CRYPTO
mod forex; // FOREX
mod futures; // FUTURES
mod indices; // INDICES
mod options; // OPTIONS

pub(crate) mod websocket;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::PolygonClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

// Capability modules
pub use chart::*;
pub use corporate::*;
pub use discovery::*;
pub use fundamentals::*;
pub use options::snapshots::fetch_options_response;
pub use quote::*;

// Asset-class modules
pub use crypto::snapshots::*;
pub use forex::quotes::*;
pub use futures::snapshots::*;
pub use indices::snapshots::*;

// Other capability modules
pub use economic::*;
pub use filings::*;

/// Polygon.io free-tier rate limit: 5 req/sec.
const PG_RATE_PER_SEC: f64 = 5.0;

struct PolygonSingleton {
    api_key: String,
    timeout: Duration,
    limiter: Arc<RateLimiter>,
}

static PG_SINGLETON: OnceLock<PolygonSingleton> = OnceLock::new();

/// Initialize the global Polygon client with an API key.
///
/// Must be called once before using any query functions. Subsequent calls return an error.
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if already initialized.
#[allow(dead_code)]
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the Polygon client with a custom timeout.
#[allow(dead_code)]
pub fn init_with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<()> {
    PG_SINGLETON
        .set(PolygonSingleton {
            api_key: api_key.into(),
            timeout,
            limiter: Arc::new(RateLimiter::new(PG_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason: "Polygon client already initialized".to_string(),
        })
}

/// Build a fresh client from the singleton state.
pub(crate) fn build_client() -> Result<client::PolygonClient> {
    if PG_SINGLETON.get().is_none()
        && let Ok(key) = std::env::var("POLYGON_API_KEY")
    {
        let _ = PG_SINGLETON.set(PolygonSingleton {
            api_key: key,
            timeout: Duration::from_secs(30),
            limiter: Arc::new(RateLimiter::new(PG_RATE_PER_SEC)),
        });
    }
    let s = PG_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason:
                "POLYGON_API_KEY not set. Call polygon::init(key) or set POLYGON_API_KEY env var."
                    .to_string(),
        })?;
    PolygonClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))
}

/// Build a test client pointing at a mock server URL.
#[cfg(test)]
pub(crate) fn build_test_client(base_url: &str) -> Result<client::PolygonClient> {
    PolygonClientBuilder::new("test-key")
        .timeout(Duration::from_secs(5))
        .base_url(base_url)
        .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
}

/// Internal: read the configured API key. Used by the websocket module.
pub(crate) fn api_key() -> Result<String> {
    PG_SINGLETON
        .get()
        .map(|s| s.api_key.clone())
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason: "Polygon not initialized. Call polygon::init(api_key) first.".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_errors_on_double_init() {
        let _ = init("test-key-1");
        let result = init("test-key-2");
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }
}
