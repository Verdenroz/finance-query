//! Token bucket rate limiter for SEC EDGAR API.
//!
//! SEC EDGAR enforces a global limit of 10 requests per second.
//! This module provides a simple token bucket implementation using
//! only `tokio` primitives (no external rate-limiting crates).

use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

struct TokenState {
    available: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
}

/// A token bucket rate limiter.
///
/// Tokens are consumed on each request and refilled at a steady rate.
/// When no tokens are available, `acquire()` sleeps until one is ready.
pub(crate) struct RateLimiter {
    state: Mutex<TokenState>,
}

impl RateLimiter {
    /// Create a rate limiter that allows `max_per_second` requests per second.
    pub fn new(max_per_second: f64) -> Self {
        Self {
            state: Mutex::new(TokenState {
                available: max_per_second,
                last_refill: Instant::now(),
                max_tokens: max_per_second,
                refill_rate: max_per_second,
            }),
        }
    }

    /// Acquire a token, sleeping if necessary to respect the rate limit.
    pub async fn acquire(&self) {
        loop {
            let sleep_duration = {
                let mut state = self.state.lock().await;
                let now = Instant::now();
                let elapsed = now.duration_since(state.last_refill).as_secs_f64();
                state.available =
                    (state.available + elapsed * state.refill_rate).min(state.max_tokens);
                state.last_refill = now;

                if state.available >= 1.0 {
                    state.available -= 1.0;
                    return;
                }

                let deficit = 1.0 - state.available;
                Duration::from_secs_f64(deficit / state.refill_rate)
            };
            tokio::time::sleep(sleep_duration).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_immediate_acquire() {
        let limiter = RateLimiter::new(10.0);
        // Should acquire immediately (10 tokens available)
        for _ in 0..10 {
            limiter.acquire().await;
        }
    }

    #[tokio::test]
    async fn test_rate_limiting_blocks() {
        tokio::time::pause();

        let limiter = RateLimiter::new(2.0);
        // Consume both tokens
        limiter.acquire().await;
        limiter.acquire().await;

        // Next acquire should require waiting
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // Should have waited ~500ms (1 token at 2/sec = 0.5s)
        assert!(elapsed >= Duration::from_millis(400));
        assert!(elapsed <= Duration::from_millis(600));
    }
}
