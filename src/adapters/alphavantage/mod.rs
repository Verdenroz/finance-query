//! Alpha Vantage API client for financial data.
//!
//! Requires the **`alphavantage`** feature flag and a free API key from
//! <https://www.alphavantage.co/support/#api-key>.
//!
//! Call [`init`] once at startup before using any query functions.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::{Providers, Provider, Capability, Interval, TimeRange};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Route capabilities to Alpha Vantage with Yahoo as fallback
//! let providers = Providers::builder()
//!     .route(Capability::QUOTE, &[Provider::AlphaVantage, Provider::Yahoo])
//!     .route(Capability::CHART, &[Provider::AlphaVantage, Provider::Yahoo])
//!     .route(Capability::ECONOMIC, &[Provider::AlphaVantage])
//!     .build().await?;
//!
//! let ticker = providers.ticker("AAPL").build().await?;
//! let quote = ticker.quote().await?;
//! let chart = ticker.chart(Interval::OneDay, TimeRange::OneMonth).await?;
//!
//! let gdp = providers.economic("REAL_GDP").series().await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub(crate) mod models;

pub(crate) mod commodities;
pub(crate) mod corporate;
pub(crate) mod crypto;
pub(crate) mod economic;
pub(crate) mod forex;
pub(crate) mod fundamentals;
pub(crate) mod options;
pub(crate) mod quote;
pub(crate) mod technicals;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::AlphaVantageClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

// Re-export public query functions (used by alphavantage provider)
pub use commodities::*;
pub use corporate::*;
pub use crypto::*;
pub use economic::*;
pub use forex::*;
pub use fundamentals::*;
pub use options::*;
pub use quote::*;

/// Alpha Vantage free-tier rate limit: 25 requests/day.
/// Premium: 75 req/min = 1.25 req/sec.
/// Default to a conservative 1.0 req/sec.
const AV_RATE_PER_SEC: f64 = 1.0;

struct AlphaVantageSingleton {
    api_key: String,
    timeout: Duration,
    limiter: Arc<RateLimiter>,
}

static AV_SINGLETON: OnceLock<AlphaVantageSingleton> = OnceLock::new();

/// Initialize the global Alpha Vantage client with an API key.
///
/// Must be called once before using any query functions. Subsequent calls return an error.
///
/// # Arguments
///
/// * `api_key` - Your Alpha Vantage API key (free at <https://www.alphavantage.co/support/#api-key>)
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if already initialized.
#[allow(dead_code)]
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the Alpha Vantage client with a custom timeout.
#[allow(dead_code)]
pub fn init_with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<()> {
    AV_SINGLETON
        .set(AlphaVantageSingleton {
            api_key: api_key.into(),
            timeout,
            limiter: Arc::new(RateLimiter::new(AV_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "alphavantage".to_string(),
            reason: "Alpha Vantage client already initialized".to_string(),
        })
}

/// Build a fresh client from the singleton state.
///
/// Used internally by all query functions.
pub(crate) fn build_client() -> Result<client::AlphaVantageClient> {
    if AV_SINGLETON.get().is_none() {
        let key = std::env::var("ALPHAVANTAGE_API_KEY").map_err(|_| {
            FinanceError::InvalidParameter {
                param: "alphavantage".to_string(),
                reason: "ALPHAVANTAGE_API_KEY not set. Call alphavantage::init(key) or set ALPHAVANTAGE_API_KEY env var."
                    .to_string(),
            }
        })?;
        // init() may have raced ahead; if set fails, use the init()-provided value
        let _ = AV_SINGLETON.set(AlphaVantageSingleton {
            api_key: key,
            timeout: Duration::from_secs(30),
            limiter: Arc::new(RateLimiter::new(AV_RATE_PER_SEC)),
        });
    }
    let s = AV_SINGLETON.get().unwrap(); // SAFETY: set above or init() already called
    AlphaVantageClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))
}

/// Build a test client pointing at a mock server URL.
#[cfg(test)]
pub(crate) fn build_test_client(base_url: &str) -> Result<client::AlphaVantageClient> {
    AlphaVantageClientBuilder::new("test-key")
        .timeout(Duration::from_secs(5))
        .base_url(base_url)
        .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
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
