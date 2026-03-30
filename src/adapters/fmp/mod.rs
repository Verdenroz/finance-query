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
//! use finance_query::adapters::fmp;
//! use finance_query::adapters::fmp::Period;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! fmp::init("YOUR_API_KEY")?;
//!
//! // Real-time quote
//! let quotes = fmp::quote("AAPL").await?;
//!
//! // Income statement
//! let income = fmp::income_statement("AAPL", Period::Quarter, Some(4)).await?;
//!
//! // Company profile
//! let profile = fmp::company_profile("AAPL").await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod models;

pub mod fundamentals;
pub mod analysis;
pub mod company;
pub mod prices;
pub mod crypto;
pub mod forex;
pub mod commodities;
pub mod etf_mutual_funds;
pub mod indexes;
pub mod screener;
pub mod advanced;
pub mod bulk;
pub mod stock_list;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::FmpClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use fundamentals::*;
pub use analysis::*;
pub use company::*;
pub use prices::*;
pub use crypto::*;
pub use forex::*;
pub use commodities::*;
pub use etf_mutual_funds::*;
pub use indexes::*;
pub use screener::*;
pub use advanced::*;
pub use bulk::*;
pub use stock_list::*;
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
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the FMP client with a custom timeout.
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
    let s = FMP_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "fmp".to_string(),
            reason: "FMP not initialized. Call fmp::init(api_key) first.".to_string(),
        })?;
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
