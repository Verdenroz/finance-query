//! Token bucket rate limiter for external API calls.
//!
//! Shared across all modules that need request throttling (EDGAR, FRED, CoinGecko).
//! Implements a simple token bucket: tokens refill at a steady rate and one token
//! is consumed per request. When the bucket is empty, [`RateLimiter::acquire`]
//! sleeps until a token becomes available.

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
/// When no tokens are available, [`acquire`](Self::acquire) sleeps until one is ready.
pub(crate) struct RateLimiter {
    state: Mutex<TokenState>,
}

impl RateLimiter {
    /// Create a rate limiter that allows `max_per_second` requests per second.
    ///
    /// The bucket capacity is at least 1 token so that [`acquire`](Self::acquire)
    /// always makes progress, even for sub-1/sec rates (e.g. 0.5 req/sec).
    pub fn new(max_per_second: f64) -> Self {
        let max_tokens = max_per_second.max(1.0);
        Self {
            state: Mutex::new(TokenState {
                available: max_tokens,
                last_refill: Instant::now(),
                max_tokens,
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

    #[tokio::test]
    async fn test_sub_one_per_second_rate() {
        tokio::time::pause();

        // 0.5 req/sec — bucket holds 1 token (clamped), refills at 0.5/sec
        let limiter = RateLimiter::new(0.5);
        limiter.acquire().await; // immediate (1 token available)

        let start = Instant::now();
        limiter.acquire().await; // must wait ~2s (1 token / 0.5 rate)
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(1900));
        assert!(elapsed <= Duration::from_millis(2100));
    }
}
