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
//! use finance_query::adapters::polygon;
//! use finance_query::adapters::polygon::Timespan;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! polygon::init("YOUR_API_KEY")?;
//!
//! // Stock aggregate bars
//! let bars = polygon::stock_aggregates("AAPL", 1, Timespan::Day, "2024-01-01", "2024-01-31", None).await?;
//!
//! // Snapshot
//! let snap = polygon::stock_snapshot("AAPL").await?;
//!
//! // Last trade
//! let trade = polygon::stock_last_trade("AAPL").await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod models;

pub mod stocks;
mod reference;

pub mod options;
pub mod forex;
pub mod crypto;
pub mod indices;
pub mod futures;

mod economy;
mod benzinga;
mod etf;
mod corporate_events;
mod alternative;
pub mod websocket;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::PolygonClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use stocks::*;
pub use options::*;
pub use forex::*;
pub use crypto::*;
pub use indices::*;
pub use futures::*;
pub use reference::*;
pub use economy::*;
pub use benzinga::*;
pub use etf::*;
pub use corporate_events::*;
pub use alternative::*;
pub use models::*;

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
pub fn init(api_key: impl Into<String>) -> Result<()> {
    init_with_timeout(api_key, Duration::from_secs(30))
}

/// Initialize the Polygon client with a custom timeout.
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
    let s = PG_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "polygon".to_string(),
            reason: "Polygon not initialized. Call polygon::init(api_key) first.".to_string(),
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
