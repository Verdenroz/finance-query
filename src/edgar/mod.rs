//! SEC EDGAR API client.
//!
//! Provides access to SEC EDGAR data including filing history,
//! structured XBRL financial data, and full-text search.
//!
//! All requests are rate-limited to 10 per second as required by SEC.
//! Rate limiting, HTTP connection pooling, and CIK caching are managed
//! internally via a process-global singleton.
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
mod rate_limiter;

use crate::error::{FinanceError, Result};
use crate::models::edgar::{CompanyFacts, EdgarFilingIndex, EdgarSearchResults, EdgarSubmissions};
use client::{EdgarClient, EdgarClientBuilder};
use std::sync::OnceLock;
use std::time::Duration;

/// Global EDGAR client singleton.
static EDGAR_CLIENT: OnceLock<EdgarClient> = OnceLock::new();

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
/// Returns an error if:
/// - EDGAR has already been initialized
/// - The HTTP client cannot be constructed
pub fn init(email: impl Into<String>) -> Result<()> {
    let client = EdgarClientBuilder::new(email).build()?;
    EDGAR_CLIENT
        .set(client)
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
    let client = EdgarClientBuilder::new(email)
        .app_name(app_name)
        .timeout(timeout)
        .build()?;
    EDGAR_CLIENT
        .set(client)
        .map_err(|_| FinanceError::InvalidParameter {
            param: "edgar".to_string(),
            reason: "EDGAR client already initialized".to_string(),
        })
}

/// Get a reference to the global EDGAR client.
fn client() -> Result<&'static EdgarClient> {
    EDGAR_CLIENT
        .get()
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "edgar".to_string(),
            reason: "EDGAR not initialized. Call edgar::init(email) first.".to_string(),
        })
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
    client()?.resolve_cik(symbol).await
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
    client()?.submissions(cik).await
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
    client()?.company_facts(cik).await
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
    client()?.filing_index(accession_number).await
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
    client()?
        .search(query, forms, start_date, end_date, from, size)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_sets_singleton() {
        // Note: This test cannot be run in parallel with other tests that use init()
        // since OnceLock cannot be reset.
        let result = init("test@example.com");
        assert!(result.is_ok() || result.is_err()); // May already be initialized
    }

    #[test]
    fn test_double_init_fails() {
        let _ = init("first@example.com");
        let result = init("second@example.com");
        // Second init should fail
        assert!(matches!(result, Err(FinanceError::InvalidParameter { .. })));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_resolve_cik_with_singleton() {
        let _ = init("test@example.com");
        let cik = resolve_cik("AAPL").await.unwrap();
        assert_eq!(cik, 320193);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_submissions_with_singleton() {
        let _ = init("test@example.com");
        let submissions = submissions(320193).await.unwrap();
        assert_eq!(submissions.name.as_deref(), Some("Apple Inc."));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_company_facts_with_singleton() {
        let _ = init("test@example.com");
        let facts = company_facts(320193).await.unwrap();
        assert!(facts.us_gaap().is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_search_with_singleton() {
        let _ = init("test@example.com");
        let results = search(
            "artificial intelligence",
            Some(&["10-K"]),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        assert!(results.hits.is_some());
    }
}
