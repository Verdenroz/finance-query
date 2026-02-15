use crate::client::ClientConfig;
use crate::endpoints::urls::{api, base};
use crate::error::{FinanceError, Result};
use reqwest::Proxy;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// ============================================================================
// Authentication Constants
// ============================================================================

/// User agent to use for requests (Chrome on Windows)
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Timeout for authentication requests
const AUTH_TIMEOUT: Duration = Duration::from_secs(15);

/// Minimum interval between auth refreshes (prevent excessive refreshing)
#[cfg(test)]
const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum age of auth before considering it stale
#[cfg(test)]
const AUTH_MAX_AGE: Duration = Duration::from_secs(3600); // 1 hour

/// Yahoo Finance authentication data
#[derive(Clone)]
pub struct YahooAuth {
    /// CSRF crumb token
    pub crumb: String,
    /// Last time auth was refreshed
    pub last_refresh: Instant,
    /// HTTP client with cookies
    pub(crate) http_client: reqwest::Client,
}

impl std::fmt::Debug for YahooAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YahooAuth")
            .field("crumb", &self.crumb)
            .field("last_refresh", &self.last_refresh)
            .finish()
    }
}

impl YahooAuth {
    /// Authenticate with Yahoo Finance using custom configuration
    ///
    /// Allows specifying timeout and proxy settings for the HTTP client.
    pub async fn authenticate_with_config(config: &ClientConfig) -> Result<Self> {
        info!("Starting Yahoo Finance authentication");

        // Create HTTP client with configuration
        let mut builder = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(config.timeout)
            .connect_timeout(AUTH_TIMEOUT)
            .user_agent(USER_AGENT);

        // Apply proxy if configured
        if let Some(proxy_url) = &config.proxy {
            debug!("Configuring proxy: {}", proxy_url);
            let proxy = Proxy::all(proxy_url)
                .map_err(|e| FinanceError::InternalError(format!("Invalid proxy URL: {}", e)))?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build().map_err(|e| {
            FinanceError::InternalError(format!("Failed to create HTTP client: {}", e))
        })?;

        // Visit fc.yahoo.com to establish session
        debug!("Visiting {} to establish session", base::YAHOO_FC);
        client.get(base::YAHOO_FC).send().await.map_err(|e| {
            FinanceError::InternalError(format!("Failed to establish session: {}", e))
        })?;

        // Try to get crumb from query1
        debug!("Attempting to fetch crumb from query1");
        let crumb = get_crumb(&client, api::CRUMB_QUERY1).await.map_err(|e| {
            warn!("Failed to fetch crumb: {}", e);
            FinanceError::AuthenticationFailed {
                context: format!("Failed to fetch crumb: {}", e),
            }
        })?;

        info!("Successfully authenticated with Yahoo Finance");
        Ok(Self {
            crumb,
            last_refresh: Instant::now(),
            http_client: client,
        })
    }

    /// Check if authentication is still valid
    #[cfg(test)]
    pub fn is_expired(&self) -> bool {
        self.last_refresh.elapsed() > AUTH_MAX_AGE
    }

    /// Check if enough time has passed to allow refresh
    #[cfg(test)]
    pub fn can_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= MIN_REFRESH_INTERVAL
    }
}

/// Fetch crumb token from Yahoo Finance
async fn get_crumb(client: &reqwest::Client, crumb_url: &str) -> Result<String> {
    let response = client
        .get(crumb_url)
        .send()
        .await
        .map_err(|e| FinanceError::InternalError(format!("Crumb request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(FinanceError::InternalError(format!(
            "Crumb request returned status {}",
            response.status()
        )));
    }

    let crumb = response.text().await.map_err(|e| {
        FinanceError::InternalError(format!("Failed to read crumb response: {}", e))
    })?;

    // Validate crumb (should not contain HTML)
    if crumb.contains("<html") || crumb.contains("<!DOCTYPE") {
        return Err(FinanceError::InternalError(
            "Crumb response contains HTML instead of token".to_string(),
        ));
    }

    debug!(
        "Successfully fetched crumb: {}",
        &crumb[..10.min(crumb.len())]
    );
    Ok(crumb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_authenticate() {
        let auth = YahooAuth::authenticate_with_config(&ClientConfig::default()).await;
        assert!(auth.is_ok());

        let auth = auth.unwrap();
        assert!(!auth.crumb.is_empty());
        assert!(!auth.crumb.contains("<html"));
    }

    #[test]
    fn test_is_expired() {
        let client = reqwest::Client::new();
        let auth = YahooAuth {
            crumb: "test".to_string(),
            last_refresh: Instant::now() - std::time::Duration::from_secs(7200),
            http_client: client,
        };

        assert!(auth.is_expired());
    }

    #[test]
    fn test_can_refresh() {
        let client = reqwest::Client::new();
        let auth = YahooAuth {
            crumb: "test".to_string(),
            last_refresh: Instant::now() - std::time::Duration::from_secs(60),
            http_client: client,
        };

        assert!(auth.can_refresh());
    }
}
