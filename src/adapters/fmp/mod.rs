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
//! use finance_query::format::Raw;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Route fundamentals and quote data to FMP with Yahoo as fallback
//! let providers = Providers::builder()
//!     .route(Capability::FUNDAMENTALS, [Provider::Fmp, Provider::Yahoo])
//!     .route(Capability::QUOTE, [Provider::Fmp, Provider::Yahoo])
//!     .build().await?;
//!
//! let ticker = providers.ticker("AAPL").build().await?;
//! let quote = ticker.quote::<Raw>().await?;
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

use crate::adapters::singleton::{provider_build_client, provider_singleton_state};
use crate::error::Result;
use std::time::Duration;

pub use models::*;

/// FMP default rate limit: 5 req/sec.
const FMP_RATE_PER_SEC: f64 = 5.0;

provider_singleton_state!(
    name = FmpSingleton,
    static_name = FMP_SINGLETON,
    rate_const = FMP_RATE_PER_SEC,
    provider_key = "fmp",
    already_init_reason = "FMP client already initialized",
);

provider_build_client!(
    name = FmpSingleton,
    static_name = FMP_SINGLETON,
    rate_const = FMP_RATE_PER_SEC,
    provider_key = "fmp",
    env_var = "FMP_API_KEY",
    env_missing_reason = "FMP_API_KEY not set. Call fmp::init(key) or set FMP_API_KEY env var.",
    builder = client::FmpClientBuilder,
    client_ty = client::FmpClient,
);

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
