use super::client::ClientConfig;
use crate::adapters::yahoo::endpoints::{api, base};
use crate::error::{FinanceError, Result};
use reqwest::Proxy;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, info, warn};

// ============================================================================
// Authentication Constants
// ============================================================================

/// User agent to use for requests (Chrome on Windows)
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Timeout for authentication requests
const AUTH_TIMEOUT: Duration = Duration::from_secs(15);

/// Yahoo Finance authentication data
pub struct YahooAuth {
    /// CSRF crumb token. Swappable so a long-lived session can re-auth in
    /// place when Yahoo expires the crumb.
    crumb: RwLock<Arc<str>>,
    refresh_guard: tokio::sync::Mutex<()>,
    /// HTTP client with cookies
    pub(crate) http_client: reqwest::Client,
}

impl std::fmt::Debug for YahooAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YahooAuth")
            .field("crumb", &&*self.crumb())
            .finish()
    }
}

impl YahooAuth {
    /// The current CSRF crumb token.
    pub(crate) fn crumb(&self) -> Arc<str> {
        Arc::clone(&self.crumb.read().unwrap())
    }

    /// Re-run the handshake on the existing HTTP client, swapping in a new
    /// crumb. Concurrent callers collapse to one refresh.
    pub(crate) async fn refresh(&self) -> Result<()> {
        let before = self.crumb();
        let _g = self.refresh_guard.lock().await;
        if !Arc::ptr_eq(&before, &self.crumb()) {
            return Ok(());
        }

        self.http_client
            .get(base::YAHOO_FC)
            .send()
            .await
            .map_err(|e| {
                FinanceError::InternalError(format!("Failed to establish session: {}", e))
            })?;
        let crumb = get_crumb(&self.http_client, api::CRUMB_QUERY1)
            .await
            .map_err(|e| FinanceError::AuthenticationFailed {
                context: format!("Failed to refresh crumb: {}", e),
            })?;

        *self.crumb.write().unwrap() = crumb.into();
        Ok(())
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
            crumb: RwLock::new(crumb.into()),
            refresh_guard: tokio::sync::Mutex::new(()),
            http_client: client,
        })
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
        let crumb = auth.crumb();
        assert!(!crumb.is_empty());
        assert!(!crumb.contains("<html"));
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn refresh_replaces_the_crumb() {
        let auth = YahooAuth::authenticate_with_config(&ClientConfig::default())
            .await
            .unwrap();
        let before = auth.crumb();
        auth.refresh().await.unwrap();
        let after = auth.crumb();
        assert!(!after.is_empty());
        assert!(!Arc::ptr_eq(&before, &after));
    }
}
