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

        //: Try to get crumb from query1
        debug!("Attempting to fetch crumb from query1");
        match get_crumb(&client, crate::constants::endpoints::CRUMB_QUERY1).await {
            Ok(crumb) => {
                info!("Successfully authenticated with Yahoo Finance");
                return Ok(Self {
                    crumb,
                    last_refresh: Instant::now(),
                    http_client: client,
                });
            }
            Err(e) => {
                warn!("Primary crumb fetch failed: {}, trying fallback method", e);
            }
        }

        // Fallback - Try CSRF token method with GUCE consent
        debug!("Attempting CSRF token fallback method");
        match csrf_token_fallback(&client).await {
            Ok(crumb) => {
                info!("Successfully authenticated with Yahoo Finance (CSRF fallback)");
                Ok(Self {
                    crumb,
                    last_refresh: Instant::now(),
                    http_client: client,
                })
            }
            Err(e) => {
                warn!("CSRF fallback also failed: {}", e);
                Err(YahooError::AuthenticationFailed)
            }
        }
    }

    /// Check if authentication is still valid
    pub fn is_expired(&self) -> bool {
        self.last_refresh.elapsed() > crate::constants::auth::AUTH_MAX_AGE
    }

    /// Check if enough time has passed to allow refresh
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

/// CSRF token fallback method using GUCE consent flow
async fn csrf_token_fallback(client: &reqwest::Client) -> Result<String> {
    use regex::Regex;
    use scraper::{Html, Selector};

    // Get consent page to extract CSRF token and session ID
    debug!("Fetching GUCE consent page");
    let consent_response = client
        .get(urls::YAHOO_GUCE_CONSENT)
        .send()
        .await
        .map_err(|e| YahooError::InternalError(format!("Consent page request failed: {}", e)))?;

    let consent_html = consent_response
        .text()
        .await
        .map_err(|e| YahooError::InternalError(format!("Failed to read consent page: {}", e)))?;

    // Extract session ID from URL or form
    let session_id_re = Regex::new(r#"sessionId[="]([^"&]+)"#).unwrap();
    let session_id = session_id_re
        .captures(&consent_html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| YahooError::InternalError("Failed to extract session ID".to_string()))?;

    // Extract CSRF token
    let document = Html::parse_document(&consent_html);
    let csrf_selector = Selector::parse(r#"input[name="csrfToken"]"#).unwrap();
    let csrf_token = document
        .select(&csrf_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .ok_or_else(|| YahooError::InternalError("Failed to extract CSRF token".to_string()))?;

    debug!(
        "Extracted session_id: {}, csrf_token length: {}",
        session_id,
        csrf_token.len()
    );

    // Submit consent
    let consent_url = format!("{}?sessionId={}", urls::YAHOO_CONSENT_SUBMIT, session_id);
    let consent_data = [
        ("csrfToken", csrf_token),
        ("sessionId", session_id),
        (
            "originalDoneUrl",
            "https://guce.yahoo.com/copyConsent?sessionId=",
        ),
        ("namespace", "yahoo"),
        ("agree", "agree"),
    ];

    client
        .post(&consent_url)
        .form(&consent_data)
        .send()
        .await
        .map_err(|e| YahooError::InternalError(format!("Consent submission failed: {}", e)))?;

    // Copy consent
    let copy_consent_url = format!("{}?sessionId={}", urls::YAHOO_COPY_CONSENT, session_id);
    client
        .get(&copy_consent_url)
        .send()
        .await
        .map_err(|e| YahooError::InternalError(format!("Copy consent failed: {}", e)))?;

    // Try to get crumb again from query2
    debug!("Fetching crumb after consent flow");
    let crumb = get_crumb(client, crate::constants::endpoints::CRUMB_QUERY2).await?;

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
