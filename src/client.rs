use crate::auth::YahooAuth;
use crate::constants::{Interval, TimeRange};
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
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: crate::constants::timeouts::DEFAULT_TIMEOUT,
            proxy: None,
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
}

impl YahooClient {
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
        let http = self.http.read().await;

        // Build request with crumb (cookies are automatically handled by reqwest)
        let request = http.get(url).query(&[("crumb", &auth.crumb)]);

        debug!("Making request to {}", url);

        // Send request
        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                YahooError::Timeout
            } else {
                YahooError::HttpError(e)
            }
        })?;

        // Check response status
        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                401 => Err(YahooError::AuthenticationFailed),
                404 => Err(YahooError::SymbolNotFound("Unknown symbol".to_string())),
                429 => Err(YahooError::RateLimited),
                _ => Err(YahooError::UnexpectedResponse(format!("HTTP {}", status))),
            };
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

    /// Make a GET request with query parameters and crumb authentication
    pub async fn request_with_params<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        params: &T,
    ) -> Result<reqwest::Response> {
        let auth = self.auth.read().await;
        let http = self.http.read().await;

        let request = http.get(url).query(&[("crumb", &auth.crumb)]).query(params);

        debug!("Making request to {}", url);

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                YahooError::Timeout
            } else {
                YahooError::HttpError(e)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                401 => Err(YahooError::AuthenticationFailed),
                404 => Err(YahooError::SymbolNotFound("Unknown symbol".to_string())),
                429 => Err(YahooError::RateLimited),
                _ => Err(YahooError::UnexpectedResponse(format!("HTTP {}", status))),
            };
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
    pub async fn get_quotes(&self, symbols: &[&str]) -> Result<serde_json::Value> {
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
    pub async fn search(&self, query: &str, hits: u32) -> Result<serde_json::Value> {
        crate::endpoints::search::fetch(self, query, hits).await
    }

    /// Get similar/recommended quotes for a symbol
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = finance_query::YahooClient::new(Default::default()).await?;
    /// let similar = client.get_similar_quotes("AAPL", 5).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_similar_quotes(&self, symbol: &str, limit: u32) -> Result<serde_json::Value> {
        crate::endpoints::recommendations::fetch(self, symbol, limit).await
    }

    /// Fetch fundamentals timeseries data (financial statements)
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
    /// let financials = client.get_fundamentals_timeseries("AAPL", 0, 9999999999, &types).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_fundamentals_timeseries(
        &self,
        symbol: &str,
        period1: i64,
        period2: i64,
        types: &[&str],
    ) -> Result<serde_json::Value> {
        crate::endpoints::timeseries::fetch(self, symbol, period1, period2, types).await
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
    pub async fn get_quote_type(&self, symbol: &str) -> Result<serde_json::Value> {
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
        let json = result.unwrap();
        assert!(json.get("quotes").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_similar_quotes() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client.get_similar_quotes("AAPL", 5).await;
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
    async fn test_get_fundamentals_timeseries() {
        let client = YahooClient::new(ClientConfig::default()).await.unwrap();
        let result = client
            .get_fundamentals_timeseries(
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
