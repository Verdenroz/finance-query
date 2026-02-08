//! SEC EDGAR HTTP client with rate limiting and CIK caching.
//!
//! Provides access to EDGAR APIs: submissions, company facts, and full-text search.
//! Handles the SEC-required User-Agent header and 10 req/sec rate limit internally.

use super::rate_limiter::RateLimiter;
use crate::endpoints::edgar as urls;
use crate::error::{Result, YahooError};
use crate::models::edgar::{CompanyFacts, EdgarSearchResults, EdgarSubmissions};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Builder for constructing an [`EdgarClient`].
///
/// The SEC requires all automated requests to include a User-Agent header
/// with a contact email address. This builder enforces that requirement.
///
/// # Example
///
/// ```no_run
/// use finance_query::EdgarClientBuilder;
///
/// let client = EdgarClientBuilder::new("user@example.com")
///     .app_name("my-app")
///     .timeout(std::time::Duration::from_secs(60))
///     .build()
///     .unwrap();
/// ```
pub struct EdgarClientBuilder {
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

    /// Build the [`EdgarClient`].
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn build(self) -> Result<EdgarClient> {
        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("{}/{} ({})", self.app_name, version, self.email);

        let http = reqwest::Client::builder()
            .user_agent(&user_agent)
            .timeout(self.timeout)
            .build()
            .map_err(YahooError::HttpError)?;

        info!("EDGAR client initialized with User-Agent: {}", user_agent);

        Ok(EdgarClient {
            http,
            rate_limiter: Arc::new(RateLimiter::new(10.0)),
            cik_cache: Arc::new(RwLock::new(None)),
        })
    }
}

/// SEC EDGAR API client.
///
/// Handles rate limiting (10 req/sec) and CIK caching internally.
/// Must be constructed via [`EdgarClientBuilder`] which requires a contact email.
///
/// # Example
///
/// ```no_run
/// use finance_query::EdgarClientBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
/// let cik = edgar.resolve_cik("AAPL").await?;
/// println!("Apple CIK: {}", cik);
/// # Ok(())
/// # }
/// ```
pub struct EdgarClient {
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
            .map_err(YahooError::HttpError)?;

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
            .map_err(YahooError::HttpError)?;

        let status = response.status().as_u16();
        if !response.status().is_success() {
            return Err(Self::map_status(status, url));
        }
        Ok(response)
    }

    fn map_status(status: u16, url: &str) -> YahooError {
        match status {
            403 => YahooError::AuthenticationFailed {
                context: format!(
                    "EDGAR returned 403 Forbidden for {}. Ensure User-Agent includes a valid contact email.",
                    url
                ),
            },
            404 => YahooError::SymbolNotFound {
                symbol: None,
                context: format!("EDGAR resource not found: {}", url),
            },
            429 => YahooError::RateLimited {
                retry_after: Some(1),
            },
            status @ 500.. => YahooError::ServerError {
                status,
                context: format!("EDGAR server error for {}", url),
            },
            _ => YahooError::UnexpectedResponse(format!(
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
    /// use finance_query::EdgarClientBuilder;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
    /// let cik = edgar.resolve_cik("AAPL").await?;
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
            .ok_or_else(|| YahooError::SymbolNotFound {
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
        let json: serde_json::Value = response.json().await.map_err(YahooError::HttpError)?;

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
        response.json().await.map_err(YahooError::HttpError)
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
        response.json().await.map_err(YahooError::HttpError)
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
    /// use finance_query::EdgarClientBuilder;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
    /// let results = edgar.search(
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

        if let Some(forms) = forms {
            params.push(("forms", forms.join(",")));
        }
        if let Some(start) = start_date {
            params.push(("dateRange", "custom".to_string()));
            params.push(("startdt", start.to_string()));
        }
        if let Some(end) = end_date {
            if start_date.is_none() {
                params.push(("dateRange", "custom".to_string()));
            }
            params.push(("enddt", end.to_string()));
        }
        if let Some(from) = from {
            params.push(("from", from.to_string()));
        }
        if let Some(size) = size {
            params.push(("size", size.to_string()));
        }

        let response = self
            .get_with_params(urls::FULL_TEXT_SEARCH, &params)
            .await?;
        response.json().await.map_err(YahooError::HttpError)
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
            YahooError::AuthenticationFailed { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(404, "test"),
            YahooError::SymbolNotFound { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(429, "test"),
            YahooError::RateLimited { .. }
        ));
        assert!(matches!(
            EdgarClient::map_status(500, "test"),
            YahooError::ServerError { .. }
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
