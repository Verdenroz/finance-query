//! Token bucket rate limiter for CoinGecko public API (30 req/min free tier).

use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

struct TokenState {
    available: f64,
    last_refill: Instant,
    max_tokens: f64,
    refill_rate: f64,
}

pub(crate) struct RateLimiter {
    state: Mutex<TokenState>,
}

impl RateLimiter {
    pub fn new(max_per_second: f64) -> Self {
        // Bucket must hold at least 1 token so acquire() can always make progress.
        // For rates < 1/sec (e.g. 0.5), max_tokens is capped at 1.0 while the
        // refill_rate stays at the true rate, enforcing the correct long-run average.
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
