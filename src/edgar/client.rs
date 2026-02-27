//! SEC EDGAR HTTP client with rate limiting and CIK caching.
//!
//! Provides access to EDGAR APIs: submissions, company facts, and full-text search.
//! Handles the SEC-required User-Agent header and 10 req/sec rate limit internally.

use crate::endpoints::edgar as urls;
use crate::error::{FinanceError, Result};
use crate::models::edgar::{CompanyFacts, EdgarFilingIndex, EdgarSearchResults, EdgarSubmissions};
use crate::rate_limiter::RateLimiter;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Builder for constructing an [`EdgarClient`].
///
/// The SEC requires all automated requests to include a User-Agent header
/// with a contact email address. This builder enforces that requirement.
pub(super) struct EdgarClientBuilder {
    email: String,
    app_name: String,
    timeout: Duration,
}

impl EdgarClientBuilder {
    /// Create a new builder with the required contact email.
    ///
    /// The email is included in the User-Agent header as required by SEC EDGAR.
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            app_name: "finance-query".to_string(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set the application name (default: "finance-query").
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// Set the HTTP request timeout (default: 30 seconds).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build a standalone [`EdgarClient`] with its own rate limiter and CIK cache.
    ///
    /// Used by unit tests that construct clients directly. For the process-global
    /// singleton, use [`build_with_shared_state`](Self::build_with_shared_state).
    #[cfg(test)]
    pub fn build(self) -> Result<EdgarClient> {
        self.build_with_shared_state(
            Arc::new(RateLimiter::new(10.0)),
            Arc::new(RwLock::new(None)),
        )
    }

    /// Build using shared `Arc<RateLimiter>` and CIK cache from the singleton.
    ///
    /// The `reqwest::Client` is runtime-bound and must be rebuilt per request
    /// to avoid hyper `DispatchGone` errors across `#[tokio::test]` runtimes.
    /// The rate limiter and CIK cache persist across calls.
    pub(super) fn build_with_shared_state(
        self,
        rate_limiter: Arc<RateLimiter>,
        cik_cache: Arc<RwLock<Option<HashMap<String, u64>>>>,
    ) -> Result<EdgarClient> {
        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("{}/{} ({})", self.app_name, version, self.email);

        let http = reqwest::Client::builder()
            .user_agent(&user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(FinanceError::HttpError)?;

        Ok(EdgarClient {
            http,
            rate_limiter,
            cik_cache,
        })
    }
}

/// SEC EDGAR API client.
///
/// Handles rate limiting (10 req/sec) and CIK caching internally.
/// Constructed per-call via the singleton in [`super`], or standalone via
/// [`EdgarClientBuilder::build`] for tests.
pub(super) struct EdgarClient {
    http: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    cik_cache: Arc<RwLock<Option<HashMap<String, u64>>>>,
}

impl EdgarClient {
    /// Make a rate-limited GET request to an EDGAR endpoint.
    async fn get(&self, url: &str) -> Result<reqwest::Response> {
        self.rate_limiter.acquire().await;
        debug!("EDGAR GET {}", url);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(FinanceError::HttpError)?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            return Err(Self::map_status(status, url));
        }
        Ok(response)
    }

    /// Make a rate-limited GET request with query parameters.
    async fn get_with_params<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<reqwest::Response> {
        self.rate_limiter.acquire().await;
        debug!("EDGAR GET {} (with params)", url);
        let response = self
            .http
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(FinanceError::HttpError)?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            return Err(Self::map_status(status, url));
        }
        Ok(response)
    }

    fn map_status(status: u16, url: &str) -> FinanceError {
        match status {
            403 => FinanceError::AuthenticationFailed {
                context: format!(
                    "EDGAR returned 403 Forbidden for {}. Ensure User-Agent includes a valid contact email.",
                    url
                ),
            },
            404 => FinanceError::SymbolNotFound {
                symbol: None,
                context: format!("EDGAR resource not found: {}", url),
            },
            429 => FinanceError::RateLimited {
                retry_after: Some(1),
            },
            status @ 500.. => FinanceError::ServerError {
                status,
                context: format!("EDGAR server error for {}", url),
            },
            _ => FinanceError::UnexpectedResponse(format!(
                "EDGAR returned unexpected status {} for {}",
                status, url
            )),
        }
    }

    // ========================================================================
    // CIK Resolution
    // ========================================================================

    /// Resolve a ticker symbol to its SEC CIK number.
    ///
    /// The ticker-to-CIK mapping is fetched once and cached for the lifetime
    /// of this client. Lookups are case-insensitive.
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
    pub async fn resolve_cik(&self, symbol: &str) -> Result<u64> {
        self.ensure_cik_map_loaded().await?;
        let cache = self.cik_cache.read().await;
        let map = cache.as_ref().unwrap();
        map.get(&symbol.to_uppercase())
            .copied()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "Symbol not found in SEC EDGAR CIK database".to_string(),
            })
    }

    /// Ensure the CIK map is loaded (double-checked locking).
    async fn ensure_cik_map_loaded(&self) -> Result<()> {
        // Quick read check
        {
            let cache = self.cik_cache.read().await;
            if cache.is_some() {
                return Ok(());
            }
        }

        // Acquire write lock
        let mut cache = self.cik_cache.write().await;

        // Double-check (another task may have loaded while we waited)
        if cache.is_some() {
            return Ok(());
        }

        // Fetch the full mapping
        let response = self.get(urls::COMPANY_TICKERS).await?;
        let json: serde_json::Value = response.json().await.map_err(FinanceError::HttpError)?;

        // Parse: {"0":{"cik_str":320193,"ticker":"AAPL","title":"Apple Inc"},...}
        let mut map = HashMap::new();
        if let Some(obj) = json.as_object() {
            for (_key, entry) in obj {
                if let (Some(ticker), Some(cik)) = (
                    entry.get("ticker").and_then(|t| t.as_str()),
                    entry.get("cik_str").and_then(|c| c.as_u64()).or_else(|| {
                        entry
                            .get("cik_str")
                            .and_then(|c| c.as_i64())
                            .map(|v| v as u64)
                    }),
                ) {
                    map.insert(ticker.to_uppercase(), cik);
                }
            }
        }

        info!("Loaded {} ticker-to-CIK mappings from SEC EDGAR", map.len());
        *cache = Some(map);
        Ok(())
    }

    // ========================================================================
    // Submissions API
    // ========================================================================

    /// Fetch filing history and company metadata for a CIK.
    ///
    /// Returns the most recent ~1000 filings inline, with references to
    /// additional history files for older filings.
    pub async fn submissions(&self, cik: u64) -> Result<EdgarSubmissions> {
        let url = urls::submissions(cik);
        let response = self.get(&url).await?;
        response.json().await.map_err(FinanceError::HttpError)
    }

    // ========================================================================
    // Company Facts API
    // ========================================================================

    /// Fetch structured XBRL financial data for a CIK.
    ///
    /// Returns all extracted XBRL facts organized by taxonomy (us-gaap, ifrs, dei).
    /// This can be a large response (several MB for major companies).
    pub async fn company_facts(&self, cik: u64) -> Result<CompanyFacts> {
        let url = urls::company_facts(cik);
        let response = self.get(&url).await?;
        response.json().await.map_err(FinanceError::HttpError)
    }

    /// Fetch the filing index for a specific accession number.
    pub async fn filing_index(&self, accession_number: &str) -> Result<EdgarFilingIndex> {
        let (cik, accession_no_dashes) = super::accession_parts(accession_number)?;
        let url = urls::filing_index(&cik, &accession_no_dashes);
        let response = self.get(&url).await?;
        response.json().await.map_err(FinanceError::HttpError)
    }

    // ========================================================================
    // Full-Text Search
    // ========================================================================

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
    ///     Some(0),   // First page
    ///     Some(100), // 100 results per page
    /// ).await?;
    /// if let Some(hits_container) = &results.hits {
    ///     for hit in &hits_container.hits {
    ///         if let Some(source) = &hit._source {
    ///             println!("{}: {:?}", source.form.as_deref().unwrap_or("?"), source.display_names);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: &str,
        forms: Option<&[&str]>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        from: Option<usize>,
        size: Option<usize>,
    ) -> Result<EdgarSearchResults> {
        let mut params: Vec<(&str, String)> = vec![("q", query.to_string())];

        // Add optional parameters
        let has_date_filter = start_date.is_some() || end_date.is_some();
        params.extend(
            [
                forms.map(|f| ("forms", f.join(","))),
                has_date_filter.then(|| ("dateRange", "custom".to_string())),
                start_date.map(|s| ("startdt", s.to_string())),
                end_date.map(|e| ("enddt", e.to_string())),
                from.map(|f| ("from", f.to_string())),
                size.map(|s| ("size", s.to_string())),
            ]
            .into_iter()
            .flatten(),
        );

        let response = self
            .get_with_params(urls::FULL_TEXT_SEARCH, &params)
            .await?;
        response.json().await.map_err(FinanceError::HttpError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let client = EdgarClientBuilder::new("test@example.com")
            .app_name("test-app")
            .build()
            .unwrap();
        // Client was constructed successfully — User-Agent was set
        // (reqwest doesn't expose the User-Agent after construction,
        // so we verify indirectly via successful build)
        drop(client);
    }

    #[test]
    fn test_map_status_codes() {
        assert!(matches!(
            EdgarClient::map_status(403, "test"),
            FinanceError::AuthenticationFailed { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(404, "test"),
            FinanceError::SymbolNotFound { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(429, "test"),
            FinanceError::RateLimited { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(500, "test"),
            FinanceError::ServerError { .. }
        ));
    }

    #[test]
    fn test_cik_parsing() {
        let json = r#"{
            "0": {"cik_str": 320193, "ticker": "AAPL", "title": "Apple Inc"},
            "1": {"cik_str": 789019, "ticker": "MSFT", "title": "MICROSOFT CORP"}
        }"#;

        let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
        let mut map = HashMap::new();
        if let Some(obj) = parsed.as_object() {
            for (_key, entry) in obj {
                if let (Some(ticker), Some(cik)) = (
                    entry.get("ticker").and_then(|t| t.as_str()),
                    entry.get("cik_str").and_then(|c| c.as_u64()),
                ) {
                    map.insert(ticker.to_uppercase(), cik);
                }
            }
        }

        assert_eq!(map.get("AAPL"), Some(&320193));
        assert_eq!(map.get("MSFT"), Some(&789019));
        assert_eq!(map.len(), 2);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_edgar_resolve_cik() {
        let client = EdgarClientBuilder::new("test@example.com").build().unwrap();
        let cik = client.resolve_cik("AAPL").await.unwrap();
        assert_eq!(cik, 320193);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_edgar_submissions() {
        let client = EdgarClientBuilder::new("test@example.com").build().unwrap();
        let submissions = client.submissions(320193).await.unwrap();
        assert_eq!(submissions.name.as_deref(), Some("Apple Inc."));
        assert!(submissions.filings.is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_edgar_company_facts() {
        let client = EdgarClientBuilder::new("test@example.com").build().unwrap();
        let facts = client.company_facts(320193).await.unwrap();
        assert!(facts.us_gaap().is_some());
        assert!(facts.entity_name.is_some());
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_edgar_search() {
        let client = EdgarClientBuilder::new("test@example.com").build().unwrap();
        let results = client
            .search(
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
        if let Some(hits_container) = &results.hits {
            assert!(hits_container.total.is_some());
        }
    }
}
