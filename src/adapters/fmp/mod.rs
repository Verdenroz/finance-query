//! Financial Modeling Prep API client for financial data.
//!
//! Requires the **`fmp`** feature flag and an API key from
//! <https://financialmodelingprep.com/>.
//!
//! Call [`init`] once at startup before using any query functions.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::{Providers, Provider, Capability, StatementType, Frequency};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Route fundamentals and quote data to FMP with Yahoo as fallback
//! let providers = Providers::builder()
//!     .route(Capability::FUNDAMENTALS, &[Provider::Fmp, Provider::Yahoo])
//!     .route(Capability::QUOTE, &[Provider::Fmp, Provider::Yahoo])
//!     .build().await?;
//!
//! let ticker = providers.ticker("AAPL").build().await?;
//! let quote = ticker.quote().await?;
//! let income = ticker.financials(StatementType::Income, Frequency::Quarterly).await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub(crate) mod models;

// Capability-mapped subdirectory modules
pub(crate) mod commodities; // COMMODITIES
pub(crate) mod corporate; // CORPORATE
pub(crate) mod crypto; // CRYPTO
pub(crate) mod discovery; // DISCOVERY
pub(crate) mod forex; // FOREX
pub(crate) mod fundamentals; // FUNDAMENTALS
pub(crate) mod indices; // INDICES
pub(crate) mod market;
pub(crate) mod quote; // QUOTE
pub(crate) mod technicals; // TECHNICALS // MARKET

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::FmpClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use models::*;

/// FMP default rate limit: 5 req/sec.
const FMP_RATE_PER_SEC: f64 = 5.0;

struct FmpSingleton {
    api_key: String,
    timeout: Duration,
    limiter: Arc<RateLimiter>,
}

static FMP_SINGLETON: OnceLock<FmpSingleton> = OnceLock::new();

/// Initialize the global FMP client with an API key.
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

/// Initialize the FMP client with a custom timeout.
#[allow(dead_code)]
pub fn init_with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<()> {
    FMP_SINGLETON
        .set(FmpSingleton {
            api_key: api_key.into(),
            timeout,
            limiter: Arc::new(RateLimiter::new(FMP_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "fmp".to_string(),
            reason: "FMP client already initialized".to_string(),
        })
}

/// Build a fresh client from the singleton state.
pub(crate) fn build_client() -> Result<client::FmpClient> {
    if FMP_SINGLETON.get().is_none() {
        let key = std::env::var("FMP_API_KEY").map_err(|_| FinanceError::InvalidParameter {
            param: "fmp".to_string(),
            reason: "FMP_API_KEY not set. Call fmp::init(key) or set FMP_API_KEY env var."
                .to_string(),
        })?;
        // init() may have raced ahead; if set fails, use the init()-provided value
        let _ = FMP_SINGLETON.set(FmpSingleton {
            api_key: key,
            timeout: Duration::from_secs(30),
            limiter: Arc::new(RateLimiter::new(FMP_RATE_PER_SEC)),
        });
    }
    let s = FMP_SINGLETON.get().unwrap(); // SAFETY: set above or init() already called
    FmpClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))
}

/// Build a test client pointing at a mock server URL.
#[cfg(test)]
pub(crate) fn build_test_client(base_url: &str) -> Result<client::FmpClient> {
    FmpClientBuilder::new("test-key")
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
