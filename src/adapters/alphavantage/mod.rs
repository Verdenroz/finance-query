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
//! use finance_query::format::Raw;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Route capabilities to Alpha Vantage with Yahoo as fallback
//! let providers = Providers::builder()
//!     .route(Capability::QUOTE, [Provider::AlphaVantage, Provider::Yahoo])
//!     .route(Capability::CHART, [Provider::AlphaVantage, Provider::Yahoo])
//!     .route(Capability::ECONOMIC, [Provider::AlphaVantage])
//!     .build().await?;
//!
//! let ticker = providers.ticker("AAPL").build().await?;
//! let quote = ticker.quote::<Raw>().await?;
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

use crate::adapters::singleton::{provider_build_client, provider_singleton_state};
use crate::error::Result;
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

provider_singleton_state!(
    name = AlphaVantageSingleton,
    static_name = AV_SINGLETON,
    rate_const = AV_RATE_PER_SEC,
    provider_key = "alphavantage",
    already_init_reason = "Alpha Vantage client already initialized",
);

provider_build_client!(
    name = AlphaVantageSingleton,
    static_name = AV_SINGLETON,
    rate_const = AV_RATE_PER_SEC,
    provider_key = "alphavantage",
    env_var = "ALPHAVANTAGE_API_KEY",
    env_missing_reason = "ALPHAVANTAGE_API_KEY not set. Call alphavantage::init(key) or set ALPHAVANTAGE_API_KEY env var.",
    builder = client::AlphaVantageClientBuilder,
    client_ty = client::AlphaVantageClient,
);

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
    set_singleton(api_key, timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;

    #[test]
    fn test_init_errors_on_double_init() {
        let _ = init("test-key-1");
        let result = init("test-key-2");
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }
}
