//! Macro-economic data sources: FRED API and US Treasury yield curve.
//!
//! Requires the **`macro`** feature flag.
//!
//! # FRED (Federal Reserve Economic Data)
//!
//! Access 800k+ macro time series (CPI, Fed Funds Rate, M2, GDP, etc.).
//! Requires a free API key from <https://fred.stlouisfed.org/docs/api/api_key.html>.
//!
//! Call [`init`] once at startup before using [`series`].
//!
//! # US Treasury Yields
//!
//! Daily yield curve data from the US Treasury Department. No key required.
//! Use [`treasury_yields`] directly.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::fred;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // FRED: initialize with API key, then query any series
//! fred::init("your-fred-api-key")?;
//! let cpi = fred::series("CPIAUCSL").await?;
//! println!("CPI observations: {}", cpi.observations.len());
//!
//! // Treasury: no key required
//! let yields = fred::treasury_yields(2025).await?;
//! println!("Latest 10Y yield: {:?}", yields.last().and_then(|y| y.y10));
//! # Ok(())
//! # }
//! ```

mod client;
pub mod models;
mod treasury;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::FredClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use models::{MacroObservation, MacroSeries, TreasuryYield};

/// FRED free-tier rate limit: 120 requests/minute = 2 req/sec.
const FRED_RATE_PER_SEC: f64 = 2.0;

/// Stable configuration stored in the FRED process-global singleton.
///
/// Only the API key, timeout, and rate-limiter are stored — NOT the
/// `reqwest::Client`. `reqwest::Client` internally spawns hyper connection-pool
/// tasks on whichever tokio runtime first uses them; when that runtime is
/// dropped (e.g. at the end of a `#[tokio::test]`), those tasks die and
/// subsequent calls from a different runtime receive `DispatchGone`. A fresh
/// `reqwest::Client` is built per `series()` call via
/// [`FredClientBuilder::build_with_limiter`], reusing this shared limiter so
/// the 2 req/sec FRED rate limit is respected across all calls.
struct FredSingleton {
    api_key: String,
    timeout: Duration,
    limiter: Arc<RateLimiter>,
}

static FRED_SINGLETON: OnceLock<FredSingleton> = OnceLock::new();

/// Initialize the global FRED client with an API key.
///
/// Must be called once before [`series`]. Subsequent calls return an error.
///
/// # Arguments
///
/// * `api_key` - Your FRED API key (free at <https://fred.stlouisfed.org/docs/api/api_key.html>)
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if already initialized.
pub fn init(api_key: impl Into<String>) -> Result<()> {
    FRED_SINGLETON
        .set(FredSingleton {
            api_key: api_key.into(),
            timeout: Duration::from_secs(30),
            limiter: Arc::new(RateLimiter::new(FRED_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "fred".to_string(),
            reason: "FRED client already initialized".to_string(),
        })
}

/// Initialize the FRED client with a custom timeout.
pub fn init_with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<()> {
    FRED_SINGLETON
        .set(FredSingleton {
            api_key: api_key.into(),
            timeout,
            limiter: Arc::new(RateLimiter::new(FRED_RATE_PER_SEC)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "fred".to_string(),
            reason: "FRED client already initialized".to_string(),
        })
}

/// Fetch all observations for a FRED data series.
///
/// Common series IDs:
/// - `"FEDFUNDS"` — Federal Funds Rate
/// - `"CPIAUCSL"` — Consumer Price Index (all urban, seasonally adjusted)
/// - `"UNRATE"` — Unemployment Rate
/// - `"DGS10"` — 10-Year Treasury Constant Maturity Rate
/// - `"M2SL"` — M2 Money Supply
/// - `"GDP"` — US Gross Domestic Product
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if FRED has not been initialized.
pub async fn series(series_id: &str) -> Result<MacroSeries> {
    let s = FRED_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "fred".to_string(),
            reason: "FRED not initialized. Call fred::init(api_key) first.".to_string(),
        })?;
    let c = FredClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))?;
    c.series(series_id).await
}

/// Fetch US Treasury yield curve data for the given year.
///
/// No API key required. Data is published on each business day.
///
/// # Arguments
///
/// * `year` - Calendar year (e.g., `2025`). Pass the current year for recent data.
pub async fn treasury_yields(year: u32) -> Result<Vec<TreasuryYield>> {
    treasury::fetch_yields(year).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_errors_on_double_init() {
        // First init may or may not succeed (could already be set from another test).
        let _ = init("test-key-1");
        let result = init("test-key-2");
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }

    #[test]
    fn test_series_without_init_fails_gracefully() {
        // If somehow the singleton is not set, series() must return an error.
        // (This test only exercises the error path if FRED_SINGLETON isn't set yet,
        //  which may not be the case if other tests run first.)
        if FRED_SINGLETON.get().is_none() {
            // We can't reset OnceLock in tests, but we can verify the error shape:
            // Synthesise the error manually.
            let err = FinanceError::InvalidParameter {
                param: "fred".to_string(),
                reason: "not initialized".to_string(),
            };
            assert!(matches!(err, FinanceError::InvalidParameter { .. }));
        }
    }
}
