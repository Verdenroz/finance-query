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
//! use finance_query::adapters::alphavantage;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! alphavantage::init("YOUR_API_KEY")?;
//!
//! // Core stocks
//! let quote = alphavantage::global_quote("AAPL").await?;
//! let daily = alphavantage::time_series_daily("MSFT", None).await?;
//!
//! // Fundamentals
//! let overview = alphavantage::company_overview("AAPL").await?;
//!
//! // Forex & Crypto
//! let rate = alphavantage::exchange_rate("USD", "EUR").await?;
//! let btc = alphavantage::crypto_daily("BTC", "USD").await?;
//!
//! // Commodities & Economic indicators
//! let oil = alphavantage::commodity_wti(None).await?;
//! let gdp = alphavantage::real_gdp(None).await?;
//!
//! // Technical indicators
//! use finance_query::adapters::alphavantage::models::{AvInterval, SeriesType};
//! let sma = alphavantage::sma("AAPL", AvInterval::Daily, 20, SeriesType::Close).await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod models;

mod commodities;
mod core_stocks;
mod crypto;
mod economic_indicators;
mod forex;
mod fundamentals;
mod intelligence;
mod options;
mod technical_indicators;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::AlphaVantageClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

// Re-export all public query functions
pub use commodities::*;
pub use core_stocks::*;
pub use crypto::*;
pub use economic_indicators::*;
pub use forex::*;
pub use fundamentals::*;
pub use intelligence::*;
pub use options::*;
pub use technical_indicators::*;

pub use models::*;

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
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the Alpha Vantage client with a custom timeout.
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
    let s = AV_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "alphavantage".to_string(),
            reason: "Alpha Vantage not initialized. Call alphavantage::init(api_key) first."
                .to_string(),
        })?;
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
