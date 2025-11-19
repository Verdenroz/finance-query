use crate::constants::{headers, urls};
use crate::error::{Result, YahooError};
use std::time::Instant;
use tracing::{debug, info, warn};

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
    /// Authenticate with Yahoo Finance and obtain cookies + crumb
    ///
    /// This performs the full authentication flow:
    /// Visit fc.yahoo.com to establish session and get cookies
    /// Request crumb token from Yahoo Finance API
    /// If primary method fails, fall back to CSRF token method
    pub async fn authenticate() -> Result<Self> {
        info!("Starting Yahoo Finance authentication");

        // Create HTTP client for authentication
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(crate::constants::timeouts::AUTH_TIMEOUT)
            .user_agent(headers::USER_AGENT)
            .build()
            .map_err(|e| {
                YahooError::InternalError(format!("Failed to create HTTP client: {}", e))
            })?;

        // Visit fc.yahoo.com to establish session
        debug!("Visiting {} to establish session", urls::YAHOO_FC);
        client.get(urls::YAHOO_FC).send().await.map_err(|e| {
            YahooError::InternalError(format!("Failed to establish session: {}", e))
        })?;

        // Try to get crumb from query1
        debug!("Attempting to fetch crumb from query1");
        let crumb = get_crumb(&client, crate::constants::endpoints::CRUMB_QUERY1)
            .await
            .map_err(|e| {
                warn!("Failed to fetch crumb: {}", e);
                YahooError::AuthenticationFailed
            })?;

        info!("Successfully authenticated with Yahoo Finance");
        Ok(Self {
            crumb,
            last_refresh: Instant::now(),
            http_client: client,
        })
    }

    /// Check if authentication is still valid
    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        self.last_refresh.elapsed() > crate::constants::auth::AUTH_MAX_AGE
    }

    /// Check if enough time has passed to allow refresh
    #[allow(dead_code)]
    pub fn can_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= crate::constants::auth::MIN_REFRESH_INTERVAL
    }
}

/// Fetch crumb token from Yahoo Finance
async fn get_crumb(client: &reqwest::Client, crumb_url: &str) -> Result<String> {
    let response = client
        .get(crumb_url)
        .send()
        .await
        .map_err(|e| YahooError::InternalError(format!("Crumb request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(YahooError::InternalError(format!(
            "Crumb request returned status {}",
            response.status()
        )));
    }

    let crumb = response
        .text()
        .await
        .map_err(|e| YahooError::InternalError(format!("Failed to read crumb response: {}", e)))?;

    // Validate crumb (should not contain HTML)
    if crumb.contains("<html") || crumb.contains("<!DOCTYPE") {
        return Err(YahooError::InternalError(
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
    #[ignore] // Ignore by default as it makes real network requests
    async fn test_authenticate() {
        let auth = YahooAuth::authenticate().await;
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
