//! Rate limiting middleware using governor.
//!
//! Provides per-IP rate limiting to prevent abuse and protect upstream Yahoo Finance API.

use axum::{
    Json,
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use governor::{
    Quota, RateLimiter,
    clock::{Clock, DefaultClock},
    state::{InMemoryState, NotKeyed},
};
use serde::Serialize;
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::warn;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute per IP
    pub requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // Default: 60 requests per minute per IP (1 req/sec)
            requests_per_minute: 60,
        }
    }
}

impl RateLimitConfig {
    /// Create configuration from environment variable
    pub fn from_env() -> Self {
        let requests_per_minute = std::env::var("RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            requests_per_minute,
        }
    }
}

/// Shared rate limiter state
#[derive(Clone)]
pub struct RateLimiterState {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiterState {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(config.requests_per_minute).unwrap_or(NonZeroU32::new(60).unwrap()),
        );

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }
}

/// Rate limit error response
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitError {
    error: String,
    message: String,
    retry_after_seconds: u64,
}

/// Rate limiting middleware
///
/// Applies global rate limiting to all requests.
/// Note: Per-IP tracking would require additional state management.
pub async fn rate_limit_middleware(
    State(limiter): axum::extract::State<RateLimiterState>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    // Check rate limit
    match limiter.limiter.check() {
        Ok(_) => {
            // Request allowed
            next.run(request).await.into_response()
        }
        Err(not_until) => {
            // Rate limit exceeded
            let retry_after = not_until
                .wait_time_from(DefaultClock::default().now())
                .as_secs();

            warn!("Rate limit exceeded (retry after {} seconds)", retry_after);

            // Track rate limit rejection
            crate::metrics::RATE_LIMIT_REJECTIONS.inc();

            let error_response = RateLimitError {
                error: "Rate limit exceeded".to_string(),
                message: format!(
                    "Too many requests. Please retry after {} seconds.",
                    retry_after
                ),
                retry_after_seconds: retry_after,
            };

            (
                StatusCode::TOO_MANY_REQUESTS,
                [
                    ("Retry-After", retry_after.to_string()),
                    ("Content-Type", "application/json".to_string()),
                ],
                Json(error_response),
            )
                .into_response()
        }
    }
}

// Import State extractor for middleware
use axum::extract::State;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig {
            requests_per_minute: 100,
        };
        let state = RateLimiterState::new(config);

        // Verify we can check rate limits
        assert!(state.limiter.check().is_ok());
    }
}
