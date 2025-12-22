use crate::auth::YahooAuth;
use crate::constants::{Interval, TimeRange, api_params};
use crate::error::{Result, YahooError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Configuration for Yahoo Finance client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// HTTP request timeout
    #[allow(dead_code)]
    pub timeout: Duration,
    /// Optional proxy URL
    #[allow(dead_code)]
    pub proxy: Option<String>,
    /// Language code for API requests (e.g., "en-US", "ja-JP", "de-DE")
    pub lang: String,
    /// Region code for API requests (e.g., "US", "JP", "DE")
    pub region: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: crate::constants::timeouts::DEFAULT_TIMEOUT,
            proxy: None,
            lang: api_params::DEFAULT_LANG.to_string(),
            region: api_params::DEFAULT_REGION.to_string(),
        }
    }
}

impl ClientConfig {
    /// Create a new builder for ClientConfig
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::ClientConfig;
    /// use std::time::Duration;
    ///
    /// let config = ClientConfig::builder()
    ///     .timeout(Duration::from_secs(30))
    ///     .lang("ja-JP")
    ///     .region("JP")
    ///     .build();
    /// ```
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }
}

/// Builder for ClientConfig
///
/// Provides a fluent API for constructing ClientConfig instances.
///
/// # Example
///
/// ```
/// use finance_query::ClientConfig;
/// use std::time::Duration;
///
/// let config = ClientConfig::builder()
///     .timeout(Duration::from_secs(30))
///     .proxy("http://proxy.example.com:8080")
///     .lang("de-DE")
///     .region("DE")
///     .build();
/// ```
#[derive(Debug)]
pub struct ClientConfigBuilder {
    timeout: Duration,
    proxy: Option<String>,
    lang: String,
    region: String,
}

impl ClientConfigBuilder {
    fn new() -> Self {
        let default = ClientConfig::default();
        Self {
            timeout: default.timeout,
            proxy: default.proxy,
            lang: default.lang,
            region: default.region,
        }
    }

    /// Set the HTTP request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the proxy URL
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set the country (automatically sets correct lang and region)
    ///
    /// This is the recommended way to configure regional settings as it ensures
    /// lang and region are correctly paired.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{ClientConfig, Country};
    ///
    /// let config = ClientConfig::builder()
    ///     .country(Country::Germany)
    ///     .build();
    /// ```
    pub fn country(mut self, country: crate::constants::Country) -> Self {
        self.lang = country.lang().to_string();
        self.region = country.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    ///
    /// For standard countries, prefer using `.country()` instead to ensure
    /// correct lang/region pairing.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Build the ClientConfig
    pub fn build(self) -> ClientConfig {
        ClientConfig {
            timeout: self.timeout,
            proxy: self.proxy,
            lang: self.lang,
            region: self.region,
        }
    }
}

/// Yahoo Finance API client
///
/// This client handles authentication and provides methods to fetch data from Yahoo Finance.
pub struct YahooClient {
    /// HTTP client with cookie store enabled
    http: Arc<RwLock<reqwest::Client>>,
    /// Authentication data (crumb token)
    auth: Arc<RwLock<YahooAuth>>,
    /// Client configuration
    config: ClientConfig,
}

impl YahooClient {
    /// HTTP error mapping
    fn map_http_status(status: u16) -> YahooError {
        match status {
            401 => YahooError::AuthenticationFailed {
                context: "HTTP 401 Unauthorized".to_string(),
            },
            404 => YahooError::SymbolNotFound {
                symbol: None,
                context: "HTTP 404 Not Found".to_string(),
            },
            429 => YahooError::RateLimited { retry_after: None },
            status if status >= 500 => YahooError::ServerError {
                status,
                context: format!("HTTP {}", status),
            },
            _ => YahooError::UnexpectedResponse(format!("HTTP {}", status)),
        }
    }

    /// Create a new Yahoo Finance client
    ///
    /// This will perform authentication with Yahoo Finance immediately.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{YahooClient, ClientConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = YahooClient::new(ClientConfig::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(config: ClientConfig) -> Result<Self> {
        info!("Initializing Yahoo Finance client");

        // Authenticate with the provided configuration (timeout, proxy)
        let auth = YahooAuth::authenticate_with_config(&config).await?;

        // Use the authenticated HTTP client (which has the cookies and config applied)
        let http = auth.http_client.clone();

        Ok(Self {
            http: Arc::new(RwLock::new(http)),
            auth: Arc::new(RwLock::new(auth)),
            config: config.clone(),
        })
    }

    /// Make a GET request to Yahoo Finance with authentication
    ///
    /// This automatically:
    /// - Adds the crumb token as a query parameter
    /// - Includes cookies via reqwest's cookie store
    /// - Sets proper headers
    pub async fn request_with_crumb(&self, url: &str) -> Result<reqwest::Response> {
        let auth = self.auth.read().await;

        // Use the auth's HTTP client directly - it has the cookies from authentication
        // NOT self.http because that's a clone without the cookie store
        let request = auth.http_client.get(url).query(&[("crumb", &auth.crumb)]);

        debug!("Making request to {}", url);

        // Send request
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                YahooError::Timeout {
                    timeout_ms: crate::constants::timeouts::DEFAULT_TIMEOUT.as_millis() as u64,
                }
            } else {
                YahooError::HttpError(e)
            }
        })?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_http_status(status.as_u16()));
        }

        Ok(response)
    }

    /// Refresh authentication if needed
    ///
    /// This checks if the current authentication is expired and refreshes it if necessary.
    #[allow(dead_code)]
    pub async fn refresh_auth_if_needed(&self) -> Result<()> {
        let auth = self.auth.read().await;

        if auth.is_expired() && auth.can_refresh() {
            drop(auth); // Release read lock

            // Acquire write locks and refresh
            let mut auth = self.auth.write().await;
            let mut http = self.http.write().await;

            // Double-check in case another task already refreshed
            if auth.is_expired() && auth.can_refresh() {
                info!("Refreshing Yahoo Finance authentication");
                let new_auth = YahooAuth::authenticate().await?;
                *http = new_auth.http_client.clone();
                *auth = new_auth;
            }
        }

        Ok(())
    }

    /// Get a clone of the underlying HTTP client
    ///
    /// This can be useful for advanced use cases where you need direct access to the client.
    /// Note: This is an async method because the client is behind a RwLock.
    #[allow(dead_code)]
    pub async fn http_client(&self) -> reqwest::Client {
        self.http.read().await.clone()
    }

    /// Get the client configuration
    ///
    /// Returns a reference to the client's configuration, including language and region settings.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Fetch logo URLs for a symbol
    ///
    /// Returns (logoUrl, companyLogoUrl) if available, None for each if not found or on error.
    /// This uses the /v7/finance/quote endpoint with selective fields for efficiency.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol (e.g., "AAPL", "TSLA")
    ///
    /// # Returns
    ///
    /// Tuple of (logoUrl, companyLogoUrl), each as Option<String>
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use finance_query::{YahooClient, ClientConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = YahooClient::new(ClientConfig::default()).await?;
    /// let (logo_url, company_logo_url) = client.get_logo_url("AAPL").await;
    /// if let Some(url) = logo_url {
    ///     println!("Logo URL: {}", url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_logo_url(&self, symbol: &str) -> (Option<String>, Option<String>) {
        // Use existing fetch_with_fields from quotes.rs
        let json = match crate::endpoints::quotes::fetch_with_fields(
            self,
            &[symbol],
            Some(&["logoUrl", "companyLogoUrl"]),
            false, // no formatting needed
            true,  // include logo params (imgHeights, imgWidths, imgLabels)
        )
        .await
        {
            Ok(j) => j,
            Err(_) => return (None, None),
        };

        // Extract both URLs from response
        let result = match json
            .get("quoteResponse")
            .and_then(|qr| qr.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
        {
            Some(r) => r,
            None => return (None, None),
        };

        let logo_url = result
            .get("logoUrl")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let company_logo_url = result
            .get("companyLogoUrl")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        (logo_url, company_logo_url)
    }

    /// Make a GET request with query parameters and crumb authentication
    pub async fn request_with_params<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<reqwest::Response> {
        let auth = self.auth.read().await;
        let http = self.http.read().await;

        let request = http.get(url).query(&[("crumb", &auth.crumb)]).query(params);

        // Log the full request URL with all parameters.
        // (This is critical for debugging Yahoo endpoints where query params like `type` are strict.)
        if let Some(full_url) = request
            .try_clone()
            .and_then(|r| r.build().ok())
            .map(|r| r.url().to_string())
        {
            debug!("Full request URL: {}", full_url);
            info!(
                "Request to: {} (lang={}, region={})",
                full_url, self.config.lang, self.config.region
            );
        } else {
            debug!("Making request to {}", url);
            info!(
                "Request to: {} (lang={}, region={})",
                url, self.config.lang, self.config.region
            );
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                YahooError::Timeout {
                    timeout_ms: crate::constants::timeouts::DEFAULT_TIMEOUT.as_millis() as u64,
                }
            } else {
                YahooError::HttpError(e)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_http_status(status.as_u16()));
        }

        Ok(response)
    }

    /// Fetch batch quotes for multiple symbols
    ///
    /// This uses the /v7/finance/quote endpoint which is more efficient for batch requests.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let quotes = client.get_quotes(&["AAPL", "GOOGL", "MSFT"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub(crate) async fn get_quotes(&self, symbols: &[&str]) -> Result<serde_json::Value> {
        crate::endpoints::quotes::fetch(self, symbols).await
    }

    /// Fetch chart data for a symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Interval, TimeRange};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let chart = client.get_chart("AAPL", Interval::OneDay, TimeRange::OneMonth).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_chart(
        &self,
        symbol: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<serde_json::Value> {
        crate::endpoints::chart::fetch(self, symbol, interval, range).await
    }

    /// Search for quotes and news
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let results = client.search("Apple", 6).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: &str,
        hits: u32,
    ) -> Result<crate::models::search::SearchResponse> {
        let json = crate::endpoints::search::fetch(self, query, hits).await?;
        Ok(crate::models::search::SearchResponse::from_json(json)?)
    }

    /// Get recommended/similar quotes for a symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let recommendations = client.get_recommendations("AAPL", 5).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_recommendations(&self, symbol: &str, limit: u32) -> Result<serde_json::Value> {
        crate::endpoints::recommendations::fetch(self, symbol, limit).await
    }

    /// Fetch fundamentals timeseries data (financial statements)
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol
    /// * `statement_type` - Type of statement (Income, Balance, CashFlow)
    /// * `frequency` - Annual or Quarterly
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::constants::{StatementType, Frequency};
    /// let statement = client.get_financials("AAPL", StatementType::Income, Frequency::Annual).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_financials(
        &self,
        symbol: &str,
        statement_type: crate::constants::StatementType,
        frequency: crate::constants::Frequency,
    ) -> Result<crate::models::financials::FinancialStatement> {
        crate::endpoints::financials::fetch(self, symbol, statement_type, frequency).await
    }

    /// Fetch raw fundamentals timeseries data (for advanced use cases)
    ///
    /// This returns raw JSON from Yahoo Finance. For most use cases, prefer `get_financials()`
    /// which returns a parsed `FinancialStatement`.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Stock symbol
    /// * `period1` - Start Unix timestamp
    /// * `period2` - End Unix timestamp
    /// * `types` - List of fundamental types (e.g., "annualTotalRevenue", "quarterlyNetIncome")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let types = vec!["annualTotalRevenue", "annualNetIncome"];
    /// let raw_data = client.get_fundamentals_raw("AAPL", 0, 9999999999, &types).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_fundamentals_raw(
        &self,
        symbol: &str,
        period1: i64,
        period2: i64,
        types: &[&str],
    ) -> Result<serde_json::Value> {
        crate::endpoints::financials::fetch_raw(self, symbol, period1, period2, types).await
    }

    /// Fetch quote type data including company ID (quartrId)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let quote_type = client.get_quote_type("AAPL").await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub(crate) async fn get_quote_type(&self, symbol: &str) -> Result<serde_json::Value> {
        crate::endpoints::quote_type::fetch(self, symbol).await
    }

    /// Fetch quote summary with specified modules
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let modules = vec!["price", "summaryDetail"];
    /// let summary = client.get_quote_summary("AAPL", &modules).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_quote_summary(
        &self,
        symbol: &str,
        modules: &[&str],
    ) -> Result<serde_json::Value> {
        crate::endpoints::quote_summary::fetch(self, symbol, modules).await
    }

    /// Get news articles for symbols
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let news = client.get_news(&["AAPL", "MSFT"], 10).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_news(&self, symbols: &[&str], count: u32) -> Result<serde_json::Value> {
        crate::endpoints::news::fetch(self, symbols, count).await
    }

    /// Get options chain for a symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let options = client.get_options("AAPL", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_options(&self, symbol: &str, date: Option<i64>) -> Result<serde_json::Value> {
        crate::endpoints::options::fetch(self, symbol, date).await
    }

    /// Get market movers (gainers, losers, or most active)
    ///
    /// # Arguments
    ///
    /// * `screener_id` - The screener ID: "DAY_GAINERS", "DAY_LOSERS", or "MOST_ACTIVES"
    /// * `count` - Number of results to return
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let gainers = client.get_movers("DAY_GAINERS", 25).await?;
    /// let losers = client.get_movers("DAY_LOSERS", 25).await?;
    /// let actives = client.get_movers("MOST_ACTIVES", 25).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_movers(
        &self,
        screener_id: &str,
        count: u32,
    ) -> Result<crate::models::movers::MoversResponse> {
        crate::endpoints::movers::fetch(self, screener_id, count).await
    }

    /// Get earnings call transcript
    ///
    /// # Arguments
    ///
    /// * `event_id` - Event ID for the earnings call
    /// * `company_id` - Company ID (quartrId from quote_type endpoint)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let transcript = client.get_earnings_transcript("12345", "0P00000000").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_earnings_transcript(
        &self,
        event_id: &str,
        company_id: &str,
    ) -> Result<serde_json::Value> {
        crate::endpoints::earnings_transcript::fetch(self, event_id, company_id).await
    }
}

/// Blocking/synchronous Yahoo Finance client
///
/// This is a wrapper around the async `YahooClient` that provides a blocking interface.
/// It creates its own tokio runtime internally to execute async operations synchronously.
///
/// Use this when you need a simple blocking API and don't want to deal with async/await.
/// For async code, use `YahooClient` directly.
pub struct BlockingYahooClient {
    inner: YahooClient,
    runtime: tokio::runtime::Runtime,
}

#[allow(dead_code)]
impl BlockingYahooClient {
    /// Create a new blocking Yahoo Finance client with the given configuration
    pub fn new(config: ClientConfig) -> Result<Self> {
        // Create a minimal tokio runtime for blocking operations
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| YahooError::InternalError(e.to_string()))?;

        let inner = runtime.block_on(YahooClient::new(config))?;

        Ok(Self { inner, runtime })
    }

    /// Get the inner client reference
    pub fn inner(&self) -> &YahooClient {
        &self.inner
    }

    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.inner.config
    }

    /// Get quotes for multiple symbols (blocking)
    pub fn get_quotes(&self, symbols: &[&str]) -> Result<serde_json::Value> {
        self.runtime.block_on(self.inner.get_quotes(symbols))
    }

    /// Get chart data (blocking)
    pub fn get_chart(
        &self,
        symbol: &str,
        interval: crate::constants::Interval,
        range: crate::constants::TimeRange,
    ) -> Result<serde_json::Value> {
        self.runtime
            .block_on(self.inner.get_chart(symbol, interval, range))
    }

    /// Search for symbols (blocking)
    pub fn search(&self, query: &str, hits: u32) -> Result<crate::models::search::SearchResponse> {
        self.runtime.block_on(self.inner.search(query, hits))
    }

    /// Get stock recommendations (blocking)
    pub fn get_recommendations(&self, symbol: &str, limit: u32) -> Result<serde_json::Value> {
        self.runtime
            .block_on(self.inner.get_recommendations(symbol, limit))
    }

    /// Get financial statement data (blocking)
    pub fn get_financials(
        &self,
        symbol: &str,
        statement_type: crate::constants::StatementType,
        frequency: crate::constants::Frequency,
    ) -> Result<crate::models::financials::FinancialStatement> {
        self.runtime
            .block_on(self.inner.get_financials(symbol, statement_type, frequency))
    }

    /// Get raw fundamentals timeseries data (blocking)
    pub fn get_fundamentals_raw(
        &self,
        symbol: &str,
        period1: i64,
        period2: i64,
        types: &[&str],
    ) -> Result<serde_json::Value> {
        self.runtime.block_on(
            self.inner
                .get_fundamentals_raw(symbol, period1, period2, types),
        )
    }

    /// Get quote type information (blocking)
    pub fn get_quote_type(&self, symbol: &str) -> Result<serde_json::Value> {
        self.runtime.block_on(self.inner.get_quote_type(symbol))
    }

    /// Get quote summary data (blocking)
    pub fn get_quote_summary(&self, symbol: &str, modules: &[&str]) -> Result<serde_json::Value> {
        self.runtime
            .block_on(self.inner.get_quote_summary(symbol, modules))
    }

    /// Get news articles (blocking)
    pub fn get_news(&self, symbols: &[&str], count: u32) -> Result<serde_json::Value> {
        self.runtime.block_on(self.inner.get_news(symbols, count))
    }

    /// Get options chain (blocking)
    pub fn get_options(&self, symbol: &str, date: Option<i64>) -> Result<serde_json::Value> {
        self.runtime.block_on(self.inner.get_options(symbol, date))
    }

    /// Get logo URLs for a symbol (blocking)
    ///
    /// Returns (logoUrl, companyLogoUrl) if available, None for each if not found or on error.
    /// This uses the /v7/finance/quote endpoint with selective fields.
    pub fn get_logo_url(&self, symbol: &str) -> (Option<String>, Option<String>) {
        self.runtime.block_on(self.inner.get_logo_url(symbol))
    }

    /// Get market movers (blocking)
    pub fn get_movers(
        &self,
        screener_id: &str,
        count: u32,
    ) -> Result<crate::models::movers::MoversResponse> {
        self.runtime
            .block_on(self.inner.get_movers(screener_id, count))
    }

    /// Get earnings call transcript (blocking)
    pub fn get_earnings_transcript(
        &self,
        event_id: &str,
        company_id: &str,
    ) -> Result<serde_json::Value> {
        self.runtime
            .block_on(self.inner.get_earnings_transcript(event_id, company_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore by default as it makes real network requests
    async fn test_client_creation() {
        let client = YahooClient::new(ClientConfig::default()).await;
        assert!(client.is_ok());
    }

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, crate::constants::timeouts::DEFAULT_TIMEOUT);
        assert!(config.proxy.is_none());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_quotes() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client.get_quotes(&["AAPL", "GOOGL"]).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteResponse").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_chart() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client
            .get_chart("AAPL", Interval::OneDay, TimeRange::OneMonth)
            .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("chart").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_search() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client.search("Apple", 5).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.quotes.is_empty(), "Should have search results");
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_recommendations() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client.get_recommendations("AAPL", 5).await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("finance").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_quote_type() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client.get_quote_type("AAPL").await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteType").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_quote_summary() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client
            .get_quote_summary("AAPL", &["price", "summaryDetail"])
            .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("quoteSummary").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_financials() {
        use crate::constants::{Frequency, StatementType};
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client
            .get_financials("AAPL", StatementType::Income, Frequency::Annual)
            .await;
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement.symbol, "AAPL");
        assert!(statement.statement.contains_key("TotalRevenue"));
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_fundamentals_raw() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client
            .get_fundamentals_raw(
                "AAPL",
                0,
                9999999999,
                &["annualTotalRevenue", "annualNetIncome"],
            )
            .await;
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.get("timeseries").is_some());
    }
}
