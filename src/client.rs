use crate::auth::YahooAuth;
use crate::constants::{Country, Interval, TimeRange};
use crate::error::{Result, YahooError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

// ============================================================================
// Client Configuration Constants
// ============================================================================

/// Default HTTP request timeout
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default language for API requests
pub(crate) const DEFAULT_LANG: &str = "en-US";

/// Default region for API requests
pub(crate) const DEFAULT_REGION: &str = "US";

/// Merge parameter for timeseries - don't merge data
pub(crate) const API_PARAM_MERGE: &str = "false";

/// Pad timeseries - fill gaps in data
pub(crate) const API_PARAM_PAD_TIMESERIES: &str = "true";

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
            timeout: DEFAULT_TIMEOUT,
            proxy: None,
            lang: DEFAULT_LANG.to_string(),
            region: DEFAULT_REGION.to_string(),
        }
    }
}

impl ClientConfig {
    /// Create a new builder for ClientConfig
    ///
    /// # Example
    ///
    /// ```ignore
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
/// ```ignore
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
    /// ```ignore
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
    /// ```ignore
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
                    timeout_ms: DEFAULT_TIMEOUT.as_millis() as u64,
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
    /// ```ignore
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

    /// Make a POST request with JSON body and crumb authentication
    ///
    /// Used for endpoints that require POST with JSON payload (e.g., custom screeners)
    pub async fn request_post_with_crumb<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        let auth = self.auth.read().await;

        // Build URL with crumb
        let url_with_crumb = format!(
            "{}{}crumb={}",
            url,
            if url.contains('?') { "&" } else { "?" },
            auth.crumb
        );

        let request = auth
            .http_client
            .post(&url_with_crumb)
            .header("Content-Type", "application/json")
            .header("x-crumb", &auth.crumb)
            .json(body);

        debug!("Making POST request to {}", url_with_crumb);

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                YahooError::Timeout {
                    timeout_ms: DEFAULT_TIMEOUT.as_millis() as u64,
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

    /// Make a GET request with query parameters and crumb authentication
    pub async fn request_with_params<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<reqwest::Response> {
        let auth = self.auth.read().await;

        // Use the auth's HTTP client directly - it has the cookies from authentication
        let request = auth
            .http_client
            .get(url)
            .query(&[("crumb", &auth.crumb)])
            .query(params);

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
                    timeout_ms: DEFAULT_TIMEOUT.as_millis() as u64,
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
    /// ```ignore
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
    /// ```ignore
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
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use finance_query::SearchOptions;
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// // Simple search with defaults
    /// let results = client.search("Apple", &SearchOptions::default()).await?;
    ///
    /// // Search with custom options
    /// let options = SearchOptions::new()
    ///     .quotes_count(10)
    ///     .news_count(5)
    ///     .enable_research_reports(true);
    /// let results = client.search("NVDA", &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: &str,
        options: &crate::endpoints::search::SearchOptions,
    ) -> Result<crate::models::search::SearchResults> {
        let json = crate::endpoints::search::fetch(self, query, options).await?;
        Ok(crate::models::search::SearchResults::from_json(json)?)
    }

    /// Look up symbols by type (equity, ETF, index, etc.)
    ///
    /// Unlike search, lookup specializes in discovering tickers filtered by asset type.
    /// Optionally fetches logo URLs via an additional API call.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use finance_query::{LookupOptions, LookupType};
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// // Simple lookup with defaults
    /// let results = client.lookup("Apple", &LookupOptions::default()).await?;
    ///
    /// // Lookup equities only with logos
    /// let options = LookupOptions::new()
    ///     .lookup_type(LookupType::Equity)
    ///     .count(10)
    ///     .include_logo(true);
    /// let results = client.lookup("NVDA", &options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lookup(
        &self,
        query: &str,
        options: &crate::endpoints::lookup::LookupOptions,
    ) -> Result<crate::models::lookup::LookupResults> {
        let json = crate::endpoints::lookup::fetch(self, query, options).await?;
        Ok(crate::models::lookup::LookupResults::from_json(json)?)
    }

    /// Get recommended/similar quotes for a symbol
    ///
    /// # Example
    ///
    /// ```ignore
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
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::{StatementType, Frequency};
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

    /// Fetch quote type data including company ID (quartrId)
    ///
    /// # Example
    ///
    /// ```ignore
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

    /// Get options chain for a symbol
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let options = client.get_options("AAPL", None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_options(&self, symbol: &str, date: Option<i64>) -> Result<serde_json::Value> {
        crate::endpoints::options::fetch(self, symbol, date).await
    }

    /// Get data from a predefined Yahoo Finance screener
    ///
    /// Fetches stocks/funds matching predefined criteria such as day gainers,
    /// day losers, most actives, most shorted stocks, growth stocks, and more.
    ///
    /// # Arguments
    ///
    /// * `screener_type` - The predefined screener type to use
    /// * `count` - Number of results to return (max 250)
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::ScreenerType;
    /// let gainers = client.get_screener(ScreenerType::DayGainers, 25).await?;
    /// let losers = client.get_screener(ScreenerType::DayLosers, 25).await?;
    /// let actives = client.get_screener(ScreenerType::MostActives, 25).await?;
    /// let shorted = client.get_screener(ScreenerType::MostShortedStocks, 25).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_screener(
        &self,
        screener_type: crate::constants::screener_types::ScreenerType,
        count: u32,
    ) -> Result<crate::models::screeners::ScreenerResults> {
        crate::endpoints::screeners::fetch(self, screener_type, count).await
    }

    /// Execute a custom screener query
    ///
    /// Allows flexible filtering of stocks/funds/ETFs based on various criteria.
    ///
    /// # Arguments
    ///
    /// * `query` - The custom screener query to execute
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::screener_query::{ScreenerQuery, QueryCondition, Operator};
    ///
    /// // Find US stocks with high volume sorted by market cap
    /// let query = ScreenerQuery::new()
    ///     .size(25)
    ///     .sort_by("intradaymarketcap", false)
    ///     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
    ///     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000));
    ///
    /// let result = client.custom_screener(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn custom_screener(
        &self,
        query: crate::models::screeners::ScreenerQuery,
    ) -> Result<crate::models::screeners::ScreenerResults> {
        crate::endpoints::screeners::fetch_custom(self, query).await
    }

    /// Fetch detailed sector data from Yahoo Finance
    ///
    /// Returns comprehensive sector information including overview, performance,
    /// top companies, ETFs, mutual funds, industries, and research reports.
    ///
    /// # Arguments
    ///
    /// * `sector_type` - The sector to fetch data for
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::SectorType;
    /// let sector = client.get_sector(SectorType::Technology).await?;
    /// println!("Sector: {} ({} companies)", sector.name,
    ///     sector.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
    /// for company in sector.top_companies.iter().take(5) {
    ///     println!("  {} - {:?}", company.symbol, company.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_sector(
        &self,
        sector_type: crate::constants::sector_types::SectorType,
    ) -> Result<crate::models::sectors::Sector> {
        crate::endpoints::sectors::fetch(self, sector_type).await
    }

    /// Fetch detailed industry data from Yahoo Finance
    ///
    /// Returns comprehensive industry information including overview, performance,
    /// top companies, top performing companies, top growth companies, and research reports.
    ///
    /// # Arguments
    ///
    /// * `industry_key` - The industry key/slug (e.g., "semiconductors", "software-infrastructure")
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let industry = client.get_industry("semiconductors").await?;
    /// println!("Industry: {} ({} companies)", industry.name,
    ///     industry.overview.as_ref().map(|o| o.companies_count.unwrap_or(0)).unwrap_or(0));
    /// for company in industry.top_companies.iter().take(5) {
    ///     println!("  {} - {:?}", company.symbol, company.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_industry(
        &self,
        industry_key: &str,
    ) -> Result<crate::models::industries::Industry> {
        crate::endpoints::industries::fetch(self, industry_key).await
    }

    /// Get market hours/time data
    ///
    /// Returns the current status for various markets.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional region override (e.g., "US", "JP", "GB"). If None, uses client's configured region.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// // Use client's default region
    /// let hours = client.get_hours(None).await?;
    ///
    /// // Get Japan market hours
    /// let jp_hours = client.get_hours(Some("JP")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_hours(
        &self,
        region: Option<&str>,
    ) -> Result<crate::models::hours::MarketHours> {
        crate::endpoints::hours::fetch(self, region).await
    }

    /// Get list of available currencies
    ///
    /// Returns currency information from Yahoo Finance.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let currencies = client.get_currencies().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_currencies(&self) -> Result<Vec<crate::models::currencies::Currency>> {
        let json = crate::endpoints::currencies::fetch(self).await?;
        Ok(crate::models::currencies::Currency::from_response(json)?)
    }

    /// Get market summary
    ///
    /// Returns market summary with major indices, currencies, and commodities.
    ///
    /// # Arguments
    ///
    /// * `country` - Optional country for localization. If None, uses client's configured lang/region.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::Country;
    /// // Use client's default config
    /// let summary = client.get_market_summary(None).await?;
    /// // Or specify a country
    /// let summary = client.get_market_summary(Some(Country::Japan)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_market_summary(
        &self,
        country: Option<Country>,
    ) -> Result<Vec<crate::models::market_summary::MarketSummaryQuote>> {
        let json = crate::endpoints::market_summary::fetch(self, country).await?;
        Ok(crate::models::market_summary::MarketSummaryQuote::from_response(json)?)
    }

    /// Get trending tickers for a country
    ///
    /// Returns trending stocks for a specific country/region.
    ///
    /// # Arguments
    ///
    /// * `country` - Optional country for localization. If None, uses client's configured lang/region.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::Country;
    /// // Use client's default config
    /// let trending = client.get_trending(None).await?;
    /// // Or specify a country
    /// let trending = client.get_trending(Some(Country::Japan)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_trending(
        &self,
        country: Option<Country>,
    ) -> Result<Vec<crate::models::trending::TrendingQuote>> {
        let json = crate::endpoints::trending::fetch(self, country).await?;
        Ok(crate::models::trending::TrendingQuote::from_response(json)?)
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
        assert_eq!(config.timeout, DEFAULT_TIMEOUT);
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
        use crate::endpoints::search::SearchOptions;
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let options = SearchOptions::new().quotes_count(5);
        let result = client.search("Apple", &options).await;
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
}
