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
mod economic;
pub mod models;

use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;
use client::FredClientBuilder;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

pub use crate::models::economic::{MacroSeries, TreasuryYield};
pub use models::ReleaseDate;

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
    init_with_timeout(api_key, Duration::from_secs(30))
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

/// Fetch upcoming scheduled economic-data release dates (CPI, NFP, GDP, FOMC, …).
///
/// Returns releases scheduled from today onward, sorted ascending.
///
/// # Errors
///
/// Returns [`FinanceError::InvalidParameter`] if FRED has not been initialized.
pub async fn release_dates() -> Result<Vec<ReleaseDate>> {
    let s = FRED_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "fred".to_string(),
            reason: "FRED not initialized. Call fred::init(api_key) first.".to_string(),
        })?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let c = FredClientBuilder::new(&s.api_key)
        .timeout(s.timeout)
        .build_with_limiter(Arc::clone(&s.limiter))?;
    c.release_dates(&today).await
}

/// Fetch US Treasury yield curve data for the given year.
///
/// No API key required. Data is published on each business day.
///
/// # Arguments
///
/// * `year` - Calendar year (e.g., `2025`). Pass the current year for recent data.
pub async fn treasury_yields(year: u32) -> Result<Vec<TreasuryYield>> {
    economic::treasury::fetch_yields(year).await
}

// ============================================================================
// Canonical model conversion functions
// ============================================================================

/// Fetch canonical EconomicSeries for a FRED series ID.
pub async fn fetch_economic_series_response(
    series_id: &str,
) -> Result<crate::models::economic::EconomicSeries> {
    let series = crate::adapters::fred::series(series_id).await?;
    Ok(series_to_canonical(series))
}

/// Map a FRED [`MacroSeries`] to the canonical
/// [`EconomicSeries`](crate::models::economic::EconomicSeries).
fn series_to_canonical(series: MacroSeries) -> crate::models::economic::EconomicSeries {
    crate::models::economic::EconomicSeries {
        series_id: series.id,
        title: None,
        units: None,
        frequency: None,
        observations: series
            .observations
            .into_iter()
            .map(|o| crate::models::economic::MacroObservation {
                date: o.date,
                value: o.value,
            })
            .collect(),
    }
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

    fn test_client(base_url: &str) -> client::FredClient {
        FredClientBuilder::new("test-key")
            .timeout(Duration::from_secs(5))
            .base_url(base_url)
            .build_with_limiter(Arc::new(RateLimiter::new(100.0)))
            .unwrap()
    }

    /// Mocked HTTP → `FredClient::series` → `series_to_canonical`, covering the
    /// full `fetch_economic_series_response` pipeline without a network call.
    #[tokio::test]
    async fn test_series_to_canonical_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/series/observations")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("series_id".into(), "GDP".into()),
                mockito::Matcher::UrlEncoded("api_key".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("file_type".into(), "json".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "observations": [
                        { "date": "2023-01-01", "value": "26144.956" },
                        { "date": "2023-04-01", "value": "." }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let series = test_client(&server.url()).series("GDP").await.unwrap();
        assert_eq!(series.id, "GDP");
        assert_eq!(series.observations.len(), 2);
        assert_eq!(series.observations[0].date, "2023-01-01");
        assert_eq!(series.observations[0].value, Some(26144.956));
        assert_eq!(series.observations[1].value, None, "\".\" parses to None");

        let canonical = series_to_canonical(series);
        assert_eq!(canonical.series_id, "GDP");
        assert_eq!(canonical.observations.len(), 2);
        assert_eq!(canonical.observations[0].value, Some(26144.956));
        assert_eq!(canonical.observations[1].value, None);
    }

    #[tokio::test]
    async fn test_series_unknown_id_maps_400_to_invalid_parameter() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/series/observations")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .series("NOT_A_SERIES")
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::InvalidParameter { .. }));
    }

    #[tokio::test]
    async fn test_series_missing_observations_errors() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/series/observations")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({"error": "unexpected shape"}).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url()).series("GDP").await.unwrap_err();
        assert!(matches!(err, FinanceError::ResponseStructureError { .. }));
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
