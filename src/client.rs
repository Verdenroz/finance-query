use crate::auth::YahooAuth;
use crate::error::{Result, YahooError};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Configuration for Yahoo Finance client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// HTTP request timeout
    pub timeout: Duration,
    /// Optional proxy URL
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
    pub async fn new(_config: ClientConfig) -> Result<Self> {
        info!("Initializing Yahoo Finance client");

        // Authenticate first - this returns an HTTP client with cookies
        let auth = YahooAuth::authenticate().await?;

        // Use the authenticated HTTP client (which has the cookies)
        // Note: The client from auth already has cookie_store enabled
        // TODO: Apply config settings (timeout, proxy) to the HTTP client
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
    pub async fn http_client(&self) -> reqwest::Client {
        self.http.read().await.clone()
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
}
