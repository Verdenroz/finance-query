use super::auth::YahooAuth;
use crate::constants::{Interval, Region, TimeRange};
use crate::error::{FinanceError, Result};
use std::time::Duration;
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
    pub timeout: Duration,
    /// Optional proxy URL
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
    ///     .region_code("JP")
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
///     .region_code("DE")
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

    /// Set the region (automatically sets correct lang and code)
    ///
    /// This is the recommended way to configure regional settings as it ensures
    /// lang and region code are correctly paired.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use finance_query::{ClientConfig, Region};
    ///
    /// let config = ClientConfig::builder()
    ///     .region(Region::Germany)
    ///     .build();
    /// ```
    pub fn region(mut self, region: crate::constants::Region) -> Self {
        self.lang = region.lang().to_string();
        self.region = region.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP", "de-DE")
    ///
    /// For standard countries, prefer using `.region()` instead to ensure
    /// correct lang/region pairing.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE")
    ///
    /// For standard countries, prefer using `.region()` instead to ensure
    /// correct lang/region pairing.
    pub fn region_code(mut self, region: impl Into<String>) -> Self {
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
    /// Authentication data (crumb + HTTP client with cookies).
    /// Immutable after construction — no lock needed.
    auth: YahooAuth,
    /// Client configuration
    config: ClientConfig,
}

impl YahooClient {
    /// Check response status and return it if successful, or map the error code
    fn check_response(response: reqwest::Response) -> Result<reqwest::Response> {
        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_http_status(status.as_u16()));
        }
        Ok(response)
    }

    /// HTTP error mapping
    fn map_http_status(status: u16) -> FinanceError {
        match status {
            401 => FinanceError::AuthenticationFailed {
                context: "HTTP 401 Unauthorized".to_string(),
            },
            404 => FinanceError::SymbolNotFound {
                symbol: None,
                context: "HTTP 404 Not Found".to_string(),
            },
            429 => FinanceError::RateLimited { retry_after: None },
            status if status >= 500 => FinanceError::ServerError {
                status,
                context: format!("HTTP {}", status),
            },
            _ => FinanceError::UnexpectedResponse(format!("HTTP {}", status)),
        }
    }

    /// Map reqwest errors to FinanceError, using configured timeout for error messages
    fn map_request_error(&self, e: reqwest::Error) -> FinanceError {
        if e.is_timeout() {
            FinanceError::Timeout {
                timeout_ms: self.config.timeout.as_millis() as u64,
            }
        } else {
            FinanceError::HttpError(e)
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

        Ok(Self { auth, config })
    }

    /// Make a GET request to Yahoo Finance with authentication
    ///
    /// This automatically:
    /// - Adds the crumb token as a query parameter
    /// - Includes cookies via reqwest's cookie store
    /// - Sets proper headers
    pub async fn request_with_crumb(&self, url: &str) -> Result<reqwest::Response> {
        let request = self
            .auth
            .http_client
            .get(url)
            .query(&[("crumb", &self.auth.crumb)]);

        debug!("Making request to {}", url);

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;

        Self::check_response(response)
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
        let json = match crate::adapters::yahoo::quote::quotes::fetch_with_fields(
            self,
            &[symbol],
            Some(&["logoUrl", "companyLogoUrl"]),
            false,
            true,
        )
        .await
        {
            Ok(j) => j,
            Err(_) => return (None, None),
        };

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
        // Build URL with crumb
        let url_with_crumb = format!(
            "{}{}crumb={}",
            url,
            if url.contains('?') { "&" } else { "?" },
            self.auth.crumb
        );

        let request = self
            .auth
            .http_client
            .post(&url_with_crumb)
            .header("Content-Type", "application/json")
            .header("x-crumb", &self.auth.crumb)
            .json(body);

        debug!("Making POST request to {}", url_with_crumb);

        let response = request
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;

        Self::check_response(response)
    }

    /// Make a GET request with query parameters and crumb authentication
    pub async fn request_with_params<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<reqwest::Response> {
        let request = self
            .auth
            .http_client
            .get(url)
            .query(&[("crumb", &self.auth.crumb)])
            .query(params);

        debug!(
            "Making request to {} (lang={}, region={})",
            url, self.config.lang, self.config.region
        );

        let response = request
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;

        Self::check_response(response)
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
    #[cfg(test)]
    pub(crate) async fn get_quotes(&self, symbols: &[&str]) -> Result<serde_json::Value> {
        crate::adapters::yahoo::quote::quotes::fetch(self, symbols).await
    }

    /// Fetch chart data for a symbol
    ///
    /// Returns a canonical [`Chart`](crate::models::chart::Chart) with OHLCV candles,
    /// metadata, and optional events (dividends, splits, capital gains).
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
    ) -> Result<crate::models::chart::Chart> {
        use crate::error::FinanceError;

        super::common::validate_symbol(symbol)?;
        tracing::info!(
            "Fetching chart for {} ({}, {})",
            symbol,
            interval.as_str(),
            range.as_str()
        );

        let url = super::endpoints::api::chart(symbol);
        let params = [
            ("interval", interval.as_str()),
            ("range", range.as_str()),
            ("events", "div|split|capitalGain"),
        ];
        let response = self.request_with_params(&url, &params).await?;
        let json: serde_json::Value = response.json().await?;

        let chart_response = crate::models::chart::response::ChartResponse::from_json(json)
            .map_err(FinanceError::JsonParseError)?;
        let results = chart_response
            .chart
            .result
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "no chart result".into(),
            })?;
        let result = results
            .first()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "empty chart results".into(),
            })?;

        let meta = &result.meta;
        let candles: Vec<crate::models::chart::Candle> = result
            .to_candles()
            .into_iter()
            .map(|c| crate::models::chart::Candle {
                timestamp: c.timestamp,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume.max(0),
                adj_close: None,
                provider_id: Some(crate::Provider::Yahoo),
            })
            .collect();

        let chart_meta = crate::models::chart::ChartMeta {
            currency: meta.currency.clone(),
            symbol: meta.symbol.clone(),
            exchange_name: meta.exchange_name.clone(),
            full_exchange_name: meta.full_exchange_name.clone(),
            instrument_type: meta.instrument_type.clone(),
            first_trade_date: meta.first_trade_date,
            regular_market_time: meta.regular_market_time,
            gmt_offset: meta.gmt_offset,
            timezone: meta.timezone.clone(),
            exchange_timezone_name: meta.exchange_timezone_name.clone(),
            regular_market_price: meta.regular_market_price,
            fifty_two_week_high: meta.fifty_two_week_high,
            fifty_two_week_low: meta.fifty_two_week_low,
            regular_market_day_high: meta.regular_market_day_high,
            regular_market_day_low: meta.regular_market_day_low,
            regular_market_volume: meta.regular_market_volume,
            chart_previous_close: meta.chart_previous_close,
            data_granularity: meta.data_granularity.clone(),
            provider_id: Some(crate::Provider::Yahoo),
            ..Default::default()
        };

        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: chart_meta,
            candles,
            interval: None,
            range: None,
            provider_id: Some(crate::Provider::Yahoo),
        })
    }

    /// Fetch chart data for a symbol using absolute date boundaries
    ///
    /// Unlike [`get_chart`](Self::get_chart) which uses a relative [`TimeRange`],
    /// this accepts explicit Unix timestamps for `start` and `end`.
    pub async fn get_chart_range(
        &self,
        symbol: &str,
        interval: Interval,
        start: i64,
        end: i64,
    ) -> Result<crate::models::chart::Chart> {
        use crate::error::FinanceError;

        super::common::validate_symbol(symbol)?;
        tracing::info!(
            "Fetching chart for {} ({}, period1={}, period2={})",
            symbol,
            interval.as_str(),
            start,
            end
        );

        let url = super::endpoints::api::chart(symbol);
        let start_str = start.to_string();
        let end_str = end.to_string();
        let params = [
            ("interval", interval.as_str()),
            ("period1", start_str.as_str()),
            ("period2", end_str.as_str()),
            ("events", "div|split|capitalGain"),
        ];
        let response = self.request_with_params(&url, &params).await?;
        let json: serde_json::Value = response.json().await?;

        let chart_response = crate::models::chart::response::ChartResponse::from_json(json)
            .map_err(FinanceError::JsonParseError)?;
        let results = chart_response
            .chart
            .result
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "no chart result".into(),
            })?;
        let result = results
            .first()
            .ok_or_else(|| FinanceError::SymbolNotFound {
                symbol: Some(symbol.to_string()),
                context: "empty chart results".into(),
            })?;

        let meta = &result.meta;
        let candles: Vec<crate::models::chart::Candle> = result
            .to_candles()
            .into_iter()
            .map(|c| crate::models::chart::Candle {
                timestamp: c.timestamp,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume.max(0),
                adj_close: None,
                provider_id: Some(crate::Provider::Yahoo),
            })
            .collect();

        let chart_meta = crate::models::chart::ChartMeta {
            currency: meta.currency.clone(),
            symbol: meta.symbol.clone(),
            exchange_name: meta.exchange_name.clone(),
            full_exchange_name: meta.full_exchange_name.clone(),
            instrument_type: meta.instrument_type.clone(),
            first_trade_date: meta.first_trade_date,
            regular_market_time: meta.regular_market_time,
            gmt_offset: meta.gmt_offset,
            timezone: meta.timezone.clone(),
            exchange_timezone_name: meta.exchange_timezone_name.clone(),
            regular_market_price: meta.regular_market_price,
            fifty_two_week_high: meta.fifty_two_week_high,
            fifty_two_week_low: meta.fifty_two_week_low,
            regular_market_day_high: meta.regular_market_day_high,
            regular_market_day_low: meta.regular_market_day_low,
            regular_market_volume: meta.regular_market_volume,
            chart_previous_close: meta.chart_previous_close,
            data_granularity: meta.data_granularity.clone(),
            provider_id: Some(crate::Provider::Yahoo),
            ..Default::default()
        };

        Ok(crate::models::chart::Chart {
            symbol: symbol.to_string(),
            meta: chart_meta,
            candles,
            interval: None,
            range: None,
            provider_id: Some(crate::Provider::Yahoo),
        })
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
        options: &crate::adapters::yahoo::discovery::search::SearchOptions,
    ) -> Result<crate::models::discovery::search::SearchResults> {
        let json = crate::adapters::yahoo::discovery::search::fetch(self, query, options).await?;
        Ok(crate::models::discovery::search::SearchResults::from_json(
            json,
        )?)
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
        options: &crate::adapters::yahoo::discovery::lookup::LookupOptions,
    ) -> Result<crate::models::discovery::lookup::LookupResults> {
        let json = crate::adapters::yahoo::discovery::lookup::fetch(self, query, options).await?;
        Ok(crate::models::discovery::lookup::LookupResults::from_json(
            json,
        )?)
    }

    /// Get recommended/similar quotes for a symbol
    ///
    /// Returns a list of [`SimilarSymbol`](crate::models::corporate::recommendation::SimilarSymbol)
    /// entries that Yahoo suggests as similar to the given symbol.
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
    pub async fn get_recommendations(
        &self,
        symbol: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::corporate::recommendation::SimilarSymbol>> {
        use crate::adapters::yahoo::endpoints::api;
        use crate::models::corporate::recommendation::response::RecommendationResponse;

        crate::adapters::yahoo::common::validate_symbol(symbol)?;
        tracing::info!("Fetching similar quotes for: {}", symbol);

        let url = api::recommendations(symbol);
        let params = [("count", limit.to_string())];
        let response = self.request_with_params(&url, &params).await?;
        let json = response.json().await?;
        let recs: RecommendationResponse = serde_json::from_value(json)?;
        Ok(recs
            .finance
            .result
            .into_iter()
            .flat_map(|r| r.recommended_symbols)
            .take(limit as usize)
            .collect())
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
    ) -> Result<crate::models::fundamentals::FinancialStatement> {
        use crate::adapters::yahoo::client::{API_PARAM_MERGE, API_PARAM_PAD_TIMESERIES};

        super::common::validate_symbol(symbol)?;
        tracing::info!(
            "Fetching {} {} financials for: {}",
            frequency.as_str(),
            statement_type.as_str(),
            symbol
        );

        let fields = statement_type.get_fields();
        let types: Vec<String> = fields.iter().map(|&f| frequency.prefix(f)).collect();
        let types_str = types.join(",");

        let url = super::endpoints::api::financials(symbol);
        let config = self.config();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let period1 = now - (10 * 365 * 24 * 60 * 60);

        let params = [
            ("merge", API_PARAM_MERGE),
            ("padTimeSeries", API_PARAM_PAD_TIMESERIES),
            ("period1", &period1.to_string()),
            ("period2", &now.to_string()),
            ("type", types_str.as_str()),
            ("lang", config.lang.as_str()),
            ("region", config.region.as_str()),
        ];
        let response = self.request_with_params(&url, &params).await?;
        let json: serde_json::Value = response.json().await?;

        crate::models::fundamentals::FinancialStatement::from_response(
            &json,
            symbol,
            statement_type,
            frequency,
        )
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
    #[cfg(test)]
    pub(crate) async fn get_quote_type(&self, symbol: &str) -> Result<serde_json::Value> {
        crate::adapters::yahoo::quote::quote_type::fetch(self, symbol).await
    }

    /// Get options chain for a symbol
    ///
    /// Returns a canonical [`Options`](crate::models::options::Options) model with
    /// both call and put contracts for each available expiration date.
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
    pub async fn get_options(
        &self,
        symbol: &str,
        date: Option<i64>,
    ) -> Result<crate::models::options::Options> {
        use crate::Provider;
        use crate::models::options::OptionContract;

        super::common::validate_symbol(symbol)?;
        tracing::info!("Fetching options for: {}", symbol);

        let url = super::endpoints::api::options(symbol);
        let response = if let Some(date) = date {
            let params = [("date", date.to_string())];
            self.request_with_params(&url, &params).await?
        } else {
            self.request_with_crumb(&url).await?
        };
        let json: serde_json::Value = response.json().await?;

        let chain = json
            .get("optionChain")
            .and_then(|oc| oc.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first());

        let expiration_dates = chain
            .and_then(|c| c.get("expirationDates"))
            .and_then(|d| d.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
            .unwrap_or_default();

        fn map_contracts(arr: &serde_json::Value) -> Vec<OptionContract> {
            arr.as_array()
                .map(|items| {
                    items
                        .iter()
                        .map(|c| OptionContract {
                            contract_symbol: c["contractSymbol"].as_str().unwrap_or("").to_string(),
                            strike: c["strike"].as_f64().unwrap_or(0.0),
                            currency: c["currency"].as_str().map(String::from),
                            last_price: c["lastPrice"].as_f64(),
                            change: c["change"].as_f64(),
                            percent_change: None,
                            volume: c["volume"].as_u64().map(|v| v as i64),
                            open_interest: c["openInterest"].as_u64().map(|v| v as i64),
                            bid: c["bid"].as_f64(),
                            ask: c["ask"].as_f64(),
                            contract_size: None,
                            expiration: Some(c["expiration"].as_i64().unwrap_or(0)),
                            last_trade_date: None,
                            implied_volatility: c["impliedVolatility"].as_f64(),
                            in_the_money: c["inTheMoney"].as_bool(),
                        })
                        .collect()
                })
                .unwrap_or_default()
        }

        let calls = chain
            .and_then(|c| c.get("calls"))
            .map(map_contracts)
            .unwrap_or_default();

        let puts = chain
            .and_then(|c| c.get("puts"))
            .map(map_contracts)
            .unwrap_or_default();

        Ok(crate::providers::build_options(
            symbol.to_string(),
            Provider::Yahoo,
            expiration_dates,
            calls,
            puts,
        ))
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
    /// use finance_query::Screener;
    /// let gainers = client.get_screener(Screener::DayGainers, 25).await?;
    /// let losers = client.get_screener(Screener::DayLosers, 25).await?;
    /// let actives = client.get_screener(Screener::MostActives, 25).await?;
    /// let shorted = client.get_screener(Screener::MostShortedStocks, 25).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_screener(
        &self,
        screener_type: crate::constants::screeners::Screener,
        count: u32,
    ) -> Result<crate::models::discovery::screeners::ScreenerResults> {
        let url = crate::adapters::yahoo::endpoints::builders::screener(screener_type, count);
        let response = self.request_with_crumb(&url).await?;
        let json: serde_json::Value = response.json().await?;
        crate::models::discovery::screeners::ScreenerResults::from_response(&json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "screeners".to_string(),
                context: e,
            }
        })
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
    /// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt};
    ///
    /// // Find US stocks with high volume sorted by market cap
    /// let query = EquityScreenerQuery::new()
    ///     .size(25)
    ///     .sort_by(EquityField::IntradayMarketCap, false)
    ///     .add_condition(EquityField::Region.eq_str("us"))
    ///     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0));
    ///
    /// let result = client.custom_screener(query).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn custom_screener<F: crate::models::discovery::screeners::ScreenerField>(
        &self,
        query: crate::models::discovery::screeners::ScreenerQuery<F>,
    ) -> Result<crate::models::discovery::screeners::ScreenerResults> {
        let url = crate::adapters::yahoo::endpoints::builders::custom_screener();
        let response = self.request_post_with_crumb(&url, &query).await?;
        let json: serde_json::Value = response.json().await?;
        crate::models::discovery::screeners::ScreenerResults::from_custom_response(&json).map_err(
            |e| crate::error::FinanceError::ResponseStructureError {
                field: "custom_screener".to_string(),
                context: e,
            },
        )
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
    /// use finance_query::Sector;
    /// let sector = client.get_sector(Sector::Technology).await?;
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
        sector_type: crate::constants::sectors::Sector,
    ) -> Result<crate::models::market::sectors::SectorData> {
        let url = crate::adapters::yahoo::endpoints::builders::sector(sector_type.as_api_path());
        let response = self.request_with_crumb(&url).await?;
        let json: serde_json::Value = response.json().await?;
        crate::models::market::sectors::SectorData::from_response(&json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "sector".to_string(),
                context: e,
            }
        })
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
    ) -> Result<crate::models::market::industries::IndustryData> {
        let url = crate::adapters::yahoo::endpoints::builders::industry(industry_key);
        let response = self.request_with_crumb(&url).await?;
        let json: serde_json::Value = response.json().await?;
        crate::models::market::industries::IndustryData::from_response(&json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "industry".to_string(),
                context: e,
            }
        })
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
    ) -> Result<crate::models::market::hours::MarketHours> {
        let config = self.config();
        let region = region.unwrap_or(&config.region);
        let params = [
            ("formatted", "true"),
            ("key", "finance"),
            ("region", region),
        ];
        let response = self
            .request_with_params(crate::adapters::yahoo::endpoints::api::MARKET_TIME, &params)
            .await?;
        let json: serde_json::Value = response.json().await?;
        crate::models::market::hours::MarketHours::from_response(&json).map_err(|e| {
            crate::error::FinanceError::ResponseStructureError {
                field: "hours".to_string(),
                context: e,
            }
        })
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
    pub async fn get_currencies(&self) -> Result<Vec<crate::models::market::currencies::Currency>> {
        let config = self.config();
        tracing::info!(
            "Fetching currencies (lang={}, region={})",
            config.lang,
            config.region
        );
        let params = [
            ("lang", config.lang.as_str()),
            ("region", config.region.as_str()),
        ];
        let response = self
            .request_with_params(crate::adapters::yahoo::endpoints::api::CURRENCIES, &params)
            .await?;
        let json: serde_json::Value = response.json().await?;
        Ok(crate::models::market::currencies::Currency::from_response(
            json,
        )?)
    }

    /// Get market summary
    ///
    /// Returns market summary with major indices, currencies, and commodities.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional region for localization. If None, uses client's configured lang/code.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::Region;
    /// // Use client's default config
    /// let summary = client.get_market_summary(None).await?;
    /// // Or specify a region
    /// let summary = client.get_market_summary(Some(Region::France)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_market_summary(
        &self,
        region: Option<Region>,
    ) -> Result<Vec<crate::models::market::market_summary::MarketSummaryQuote>> {
        let config = self.config();
        let (lang, region) = match region {
            Some(r) => (r.lang(), r.region()),
            None => (config.lang.as_str(), config.region.as_str()),
        };
        tracing::info!("Fetching market summary (lang={}, region={})", lang, region);
        let params = [("lang", lang), ("region", region)];
        let response = self
            .request_with_params(
                crate::adapters::yahoo::endpoints::api::MARKET_SUMMARY,
                &params,
            )
            .await?;
        let json: serde_json::Value = response.json().await?;
        Ok(crate::models::market::market_summary::MarketSummaryQuote::from_response(json)?)
    }

    /// Get trending tickers for a region
    ///
    /// Returns trending stocks for a specific region.
    ///
    /// # Arguments
    ///
    /// * `region` - Optional region for localization. If None, uses client's configured lang/region.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// use finance_query::Region;
    /// // Use client's default config
    /// let trending = client.get_trending(None).await?;
    /// // Or specify a region
    /// let trending = client.get_trending(Some(Region::France)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_trending(
        &self,
        region: Option<Region>,
    ) -> Result<Vec<crate::models::discovery::trending::TrendingQuote>> {
        let config = self.config();
        let (lang, region) = match region {
            Some(r) => (r.lang(), r.region()),
            None => (config.lang.as_str(), config.region.as_str()),
        };
        tracing::info!(
            "Fetching trending tickers (lang={}, region={})",
            lang,
            region
        );
        let url = crate::adapters::yahoo::endpoints::api::trending(region);
        let params = [("lang", lang), ("region", region)];
        let response = self.request_with_params(&url, &params).await?;
        let json: serde_json::Value = response.json().await?;
        Ok(crate::models::discovery::trending::TrendingQuote::from_response(json)?)
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
        let chart = result.unwrap();
        assert!(!chart.candles.is_empty(), "Chart should have candles");
        assert_eq!(chart.symbol, "AAPL");
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_search() {
        use crate::adapters::yahoo::discovery::search::SearchOptions;
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
        let sims = result.unwrap();
        assert!(!sims.is_empty(), "Should have at least one similar symbol");
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
