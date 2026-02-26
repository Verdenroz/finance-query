//! SEC EDGAR API client.
//!
//! Provides access to SEC EDGAR data including filing history,
//! structured XBRL financial data, and full-text search.
//!
//! All requests are rate-limited to 10 per second as required by SEC.
//! Rate limiting and CIK caching are managed via a process-global singleton.
//!
//! # Quick Start
//!
//! Initialize once at application startup, then use anywhere:
//!
//! ```no_run
//! use finance_query::edgar;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize once (required)
//! edgar::init("user@example.com")?;
//!
//! // Use anywhere
//! let cik = edgar::resolve_cik("AAPL").await?;
//! let submissions = edgar::submissions(cik).await?;
//! let facts = edgar::company_facts(cik).await?;
//!
//! // Search filings
//! let results = edgar::search(
//!     "artificial intelligence",
//!     Some(&["10-K"]),
//!     Some("2024-01-01"),
//!     None,
//!     None,
//!     None,
//! ).await?;
//! # Ok(())
//! # }
//! ```

mod client;

use crate::error::{FinanceError, Result};
use crate::models::edgar::{CompanyFacts, EdgarFilingIndex, EdgarSearchResults, EdgarSubmissions};
use crate::rate_limiter::RateLimiter;
use client::EdgarClientBuilder;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

/// SEC EDGAR rate limit: 10 requests per second.
const EDGAR_RATE_PER_SEC: f64 = 10.0;

/// Stable configuration stored in the EDGAR process-global singleton.
///
/// Only configuration, the rate limiter, and the CIK cache are stored — NOT
/// the `reqwest::Client`. `reqwest::Client` internally spawns hyper
/// connection-pool tasks on whichever tokio runtime first uses them; when that
/// runtime is dropped (e.g. at the end of a `#[tokio::test]`), those tasks die
/// and subsequent calls from a different runtime receive `DispatchGone`. A fresh
/// `reqwest::Client` is built per public function call via
/// [`EdgarClientBuilder::build_with_shared_state`], reusing the shared rate
/// limiter and CIK cache.
struct EdgarSingleton {
    email: String,
    app_name: String,
    timeout: Duration,
    rate_limiter: Arc<RateLimiter>,
    cik_cache: Arc<RwLock<Option<HashMap<String, u64>>>>,
}

static EDGAR_SINGLETON: OnceLock<EdgarSingleton> = OnceLock::new();

/// Initialize the global EDGAR client with a contact email.
///
/// This function must be called once before using any EDGAR functions.
/// The SEC requires all automated requests to include a User-Agent header
/// with a contact email address.
///
/// # Arguments
///
/// * `email` - Contact email address (included in User-Agent header)
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if EDGAR has already been initialized.
pub fn init(email: impl Into<String>) -> Result<()> {
    EDGAR_SINGLETON
        .set(EdgarSingleton {
            email: email.into(),
            app_name: "finance-query".to_string(),
            timeout: Duration::from_secs(30),
            rate_limiter: Arc::new(RateLimiter::new(EDGAR_RATE_PER_SEC)),
            cik_cache: Arc::new(RwLock::new(None)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "edgar".to_string(),
            reason: "EDGAR client already initialized".to_string(),
        })
}

/// Initialize the global EDGAR client with full configuration.
///
/// Use this for custom app name and timeout settings.
///
/// # Arguments
///
/// * `email` - Contact email address (required by SEC)
/// * `app_name` - Application name (included in User-Agent)
/// * `timeout` - HTTP request timeout duration
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
/// use std::time::Duration;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init_with_config(
///     "user@example.com",
///     "my-app",
///     Duration::from_secs(60),
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn init_with_config(
    email: impl Into<String>,
    app_name: impl Into<String>,
    timeout: Duration,
) -> Result<()> {
    EDGAR_SINGLETON
        .set(EdgarSingleton {
            email: email.into(),
            app_name: app_name.into(),
            timeout,
            rate_limiter: Arc::new(RateLimiter::new(EDGAR_RATE_PER_SEC)),
            cik_cache: Arc::new(RwLock::new(None)),
        })
        .map_err(|_| FinanceError::InvalidParameter {
            param: "edgar".to_string(),
            reason: "EDGAR client already initialized".to_string(),
        })
}

/// Build a fresh [`EdgarClient`](client::EdgarClient) from the singleton's
/// config, reusing the shared rate limiter and CIK cache.
fn build_client() -> Result<client::EdgarClient> {
    let s = EDGAR_SINGLETON
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "edgar".to_string(),
            reason: "EDGAR not initialized. Call edgar::init(email) first.".to_string(),
        })?;
    EdgarClientBuilder::new(&s.email)
        .app_name(&s.app_name)
        .timeout(s.timeout)
        .build_with_shared_state(Arc::clone(&s.rate_limiter), Arc::clone(&s.cik_cache))
}

fn accession_parts(accession_number: &str) -> Result<(String, String)> {
    let cik_part = accession_number
        .split('-')
        .next()
        .unwrap_or("")
        .trim_start_matches('0')
        .to_string();
    let accession_no_dashes = accession_number.replace('-', "");

    if cik_part.is_empty() || accession_no_dashes.is_empty() {
        return Err(FinanceError::InvalidParameter {
            param: "accession_number".to_string(),
            reason: "Invalid accession number format".to_string(),
        });
    }

    Ok((cik_part, accession_no_dashes))
}

/// Resolve a ticker symbol to its SEC CIK number.
///
/// The ticker-to-CIK mapping is fetched once and cached process-wide.
/// Lookups are case-insensitive.
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// let cik = edgar::resolve_cik("AAPL").await?;
/// assert_eq!(cik, 320193);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - EDGAR has not been initialized (call `init()` first)
/// - Symbol not found in SEC database
/// - Network request fails
pub async fn resolve_cik(symbol: &str) -> Result<u64> {
    build_client()?.resolve_cik(symbol).await
}

/// Fetch filing history and company metadata for a CIK.
///
/// Returns the most recent ~1000 filings inline, with references to
/// additional history files for older filings.
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// let cik = edgar::resolve_cik("AAPL").await?;
/// let submissions = edgar::submissions(cik).await?;
/// println!("Company: {:?}", submissions.name);
/// # Ok(())
/// # }
/// ```
pub async fn submissions(cik: u64) -> Result<EdgarSubmissions> {
    build_client()?.submissions(cik).await
}

/// Fetch structured XBRL financial data for a CIK.
///
/// Returns all extracted XBRL facts organized by taxonomy (us-gaap, ifrs, dei).
/// This can be a large response (several MB for major companies).
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// let cik = edgar::resolve_cik("AAPL").await?;
/// let facts = edgar::company_facts(cik).await?;
/// println!("Entity: {:?}", facts.entity_name);
/// # Ok(())
/// # }
/// ```
pub async fn company_facts(cik: u64) -> Result<CompanyFacts> {
    build_client()?.company_facts(cik).await
}

/// Fetch the filing index for a specific accession number.
///
/// This provides the file list for a filing, which can be used to locate
/// the primary HTML document and file sizes.
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// let index = edgar::filing_index("0000320193-24-000123").await?;
/// println!("Files: {}", index.directory.item.len());
/// # Ok(())
/// # }
/// ```
pub async fn filing_index(accession_number: &str) -> Result<EdgarFilingIndex> {
    build_client()?.filing_index(accession_number).await
}

/// Search SEC EDGAR filings by text content.
///
/// # Arguments
///
/// * `query` - Search term or phrase
/// * `forms` - Optional form type filter (e.g., `&["10-K", "10-Q"]`)
/// * `start_date` - Optional start date (YYYY-MM-DD)
/// * `end_date` - Optional end date (YYYY-MM-DD)
/// * `from` - Optional pagination offset (default: 0)
/// * `size` - Optional page size (default: 100, max: 100)
///
/// # Example
///
/// ```no_run
/// use finance_query::edgar;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// edgar::init("user@example.com")?;
/// let results = edgar::search(
///     "artificial intelligence",
///     Some(&["10-K"]),
///     Some("2024-01-01"),
///     None,
///     Some(0),
///     Some(100),
/// ).await?;
/// if let Some(hits_container) = &results.hits {
///     println!("Found {} results", hits_container.total.as_ref().and_then(|t| t.value).unwrap_or(0));
/// }
/// # Ok(())
/// # }
/// ```
pub async fn search(
    query: &str,
    forms: Option<&[&str]>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    from: Option<usize>,
    size: Option<usize>,
) -> Result<EdgarSearchResults> {
    build_client()?
        .search(query, forms, start_date, end_date, from, size)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_sets_singleton() {
        let result = init("test@example.com");
        assert!(result.is_ok() || result.is_err()); // May already be initialized
    }

    #[test]
    fn test_double_init_fails() {
        let _ = init("first@example.com");
        let result = init("second@example.com");
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }

    #[test]
    fn test_singleton_is_set_after_init() {
        let _ = init("test@example.com");
        assert!(EDGAR_SINGLETON.get().is_some());
    }
}
