//! Global (non-keyed) rate limiting middleware.
//!
//! Applies one shared token bucket across all requests to protect the
//! upstream Yahoo Finance API. Per-IP tracking would require additional
//! state management and isn't implemented.
//!
//! Hand-rolled rather than a general-purpose crate: the only capability
//! this needs is a single continuously-refilling counter, which a `Mutex`
//! around a token count + timestamp covers in a few dozen lines, at none of
//! the transitive dependency weight (keyed limiters, jitter, jemalloc-style
//! CPU timing) a multi-tenant rate-limiting library carries for capabilities
//! this middleware never exercises.

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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

/// Continuously-refilling token bucket: capacity and refill rate are both
/// derived from `requests_per_minute`, so a bucket allows an initial burst
/// up to that count and then admits one request every `60s / count`.
struct TokenBucket {
    capacity: f64,
    refill_per_sec: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(requests_per_minute: u32) -> Self {
        let capacity = requests_per_minute.max(1) as f64;
        Self {
            capacity,
            refill_per_sec: capacity / 60.0,
            tokens: capacity,
            last_refill: Instant::now(),
        }
    }

    /// Attempt to consume one token. `Ok` on success; `Err(retry_after)` on
    /// failure, with the wait time until the next token would be available.
    fn check(&mut self) -> Result<(), Duration> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            let deficit = 1.0 - self.tokens;
            Err(Duration::from_secs_f64(deficit / self.refill_per_sec))
        }
    }
}

/// Shared rate limiter state
#[derive(Clone)]
pub struct RateLimiterState {
    bucket: Arc<Mutex<TokenBucket>>,
}

impl RateLimiterState {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        crate::metrics::RATE_LIMIT_QUOTA_PER_MINUTE.set(config.requests_per_minute as f64);

        Self {
            bucket: Arc::new(Mutex::new(TokenBucket::new(config.requests_per_minute))),
        }
    }

    fn check(&self) -> Result<(), Duration> {
        self.bucket
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .check()
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
    State(limiter): State<RateLimiterState>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    match limiter.check() {
        Ok(()) => {
            crate::metrics::RATE_LIMIT_ALLOWED.inc();
            next.run(request).await.into_response()
        }
        Err(retry_after) => {
            let retry_after = retry_after.as_secs().max(1);

            warn!("Rate limit exceeded (retry after {} seconds)", retry_after);

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
        assert!(state.check().is_ok());
    }

    #[test]
    fn allows_burst_up_to_capacity() {
        let mut bucket = TokenBucket::new(5);
        for _ in 0..5 {
            assert!(bucket.check().is_ok());
        }
        assert!(bucket.check().is_err());
    }

    #[test]
    fn denies_beyond_capacity_with_retry_after() {
        let mut bucket = TokenBucket::new(1);
        assert!(bucket.check().is_ok());
        let err = bucket.check().unwrap_err();
        // At 1 request/minute, the next token is ~60s away.
        assert!(err.as_secs_f64() > 50.0 && err.as_secs_f64() <= 60.0);
    }

    #[test]
    fn refills_over_time() {
        let mut bucket = TokenBucket::new(60); // 1 token/sec
        for _ in 0..60 {
            assert!(bucket.check().is_ok());
        }
        assert!(bucket.check().is_err());

        // Simulate elapsed time by rewinding last_refill instead of sleeping.
        bucket.last_refill = Instant::now() - Duration::from_secs(1);
        assert!(bucket.check().is_ok());
    }
}
