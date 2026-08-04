//! Per-IP rate limiting middleware.
//!
//! Keys a continuously-refilling token bucket on the client's TCP peer
//! address, so one abusive client can't starve everyone else's shared
//! `RATE_LIMIT_PER_MINUTE` budget. Deliberately never trusts
//! `X-Forwarded-For` or similar headers — those are caller-controlled and
//! trivially spoofed to evade or misattribute rate limiting unless a proxy
//! in front of this process is known to overwrite them, which this
//! self-hosted server can't assume. Buckets live in a bounded map: entries
//! idle past `IDLE_EVICTION` are swept once the map hits
//! `MAX_TRACKED_CLIENTS`, so long-running processes don't accumulate
//! unbounded per-IP state.
//!
//! Hand-rolled rather than a general-purpose crate: the only capability
//! this needs is a keyed, continuously-refilling counter, which a `Mutex`
//! around a small `HashMap` covers in a few dozen lines, at none of the
//! transitive dependency weight (jitter, jemalloc-style CPU timing) a
//! multi-tenant rate-limiting library carries for capabilities this
//! middleware never exercises.

use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde::Serialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::warn;

/// Upper bound on tracked per-IP buckets before idle entries are swept.
const MAX_TRACKED_CLIENTS: usize = 10_000;

/// A bucket idle longer than this is a sweep candidate once at capacity.
const IDLE_EVICTION: Duration = Duration::from_secs(600);

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per minute, applied independently per client IP.
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
    /// Create configuration from environment variables
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

/// One client's bucket plus the last time it was touched, for idle eviction.
struct ClientBucket {
    bucket: TokenBucket,
    last_seen: Instant,
}

/// Bounded map of per-IP token buckets, sharing one `requests_per_minute` quota.
struct PerIpLimiter {
    clients: HashMap<IpAddr, ClientBucket>,
    requests_per_minute: u32,
}

impl PerIpLimiter {
    fn new(requests_per_minute: u32) -> Self {
        Self {
            clients: HashMap::new(),
            requests_per_minute,
        }
    }

    fn check(&mut self, ip: IpAddr) -> Result<(), Duration> {
        let now = Instant::now();

        // Only a brand-new client can grow the map past capacity — a no-op
        // on the common path of already-seen IPs.
        if !self.clients.contains_key(&ip) && self.clients.len() >= MAX_TRACKED_CLIENTS {
            self.clients
                .retain(|_, c| now.duration_since(c.last_seen) < IDLE_EVICTION);

            // Sustained load with every tracked client still active: the idle
            // sweep freed nothing, so evict the least-recently-seen entry
            // rather than let the map grow past MAX_TRACKED_CLIENTS.
            if self.clients.len() >= MAX_TRACKED_CLIENTS
                && let Some(oldest) = self
                    .clients
                    .iter()
                    .min_by_key(|(_, c)| c.last_seen)
                    .map(|(ip, _)| *ip)
            {
                self.clients.remove(&oldest);
            }
        }

        let requests_per_minute = self.requests_per_minute;
        let client = self.clients.entry(ip).or_insert_with(|| ClientBucket {
            bucket: TokenBucket::new(requests_per_minute),
            last_seen: now,
        });
        client.last_seen = now;
        client.bucket.check()
    }
}

/// Shared rate limiter state
#[derive(Clone)]
pub struct RateLimiterState {
    limiter: Arc<Mutex<PerIpLimiter>>,
}

impl RateLimiterState {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        crate::metrics::RATE_LIMIT_QUOTA_PER_MINUTE.set(config.requests_per_minute as f64);

        Self {
            limiter: Arc::new(Mutex::new(PerIpLimiter::new(config.requests_per_minute))),
        }
    }

    fn check(&self, ip: IpAddr) -> Result<(), Duration> {
        self.limiter
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .check(ip)
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
/// Applies `RATE_LIMIT_PER_MINUTE` independently per client IP. Requires the
/// server to be served via `into_make_service_with_connect_info::<SocketAddr>()`
/// so `ConnectInfo` is available.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiterState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    match limiter.check(peer.ip()) {
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
        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Verify we can check rate limits
        assert!(state.check(ip).is_ok());
    }

    #[test]
    fn different_ips_get_independent_buckets() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
        };
        let state = RateLimiterState::new(config);
        let a: IpAddr = "10.0.0.1".parse().unwrap();
        let b: IpAddr = "10.0.0.2".parse().unwrap();

        // Exhaust `a`'s single-token budget; `b` is untouched by it.
        assert!(state.check(a).is_ok());
        assert!(state.check(a).is_err());
        assert!(state.check(b).is_ok());
    }

    #[test]
    fn same_ip_shares_one_bucket_across_calls() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
        };
        let state = RateLimiterState::new(config);
        let ip: IpAddr = "10.0.0.3".parse().unwrap();

        assert!(state.check(ip).is_ok());
        assert!(state.check(ip).is_ok());
        assert!(state.check(ip).is_err());
    }

    #[test]
    fn evicts_idle_entries_once_at_capacity() {
        let mut limiter = PerIpLimiter::new(10);
        let stale = Instant::now() - IDLE_EVICTION - Duration::from_secs(1);
        for i in 0..MAX_TRACKED_CLIENTS as u32 {
            limiter.clients.insert(
                IpAddr::from(std::net::Ipv4Addr::from(i)),
                ClientBucket {
                    bucket: TokenBucket::new(10),
                    last_seen: stale,
                },
            );
        }
        assert_eq!(limiter.clients.len(), MAX_TRACKED_CLIENTS);

        let fresh: IpAddr = "203.0.113.1".parse().unwrap();
        assert!(limiter.check(fresh).is_ok());

        // Every stale entry was swept, leaving only the new arrival.
        assert_eq!(limiter.clients.len(), 1);
    }

    #[test]
    fn evicts_lru_entry_when_all_tracked_clients_are_still_active() {
        let mut limiter = PerIpLimiter::new(10);
        let now = Instant::now();
        // Every entry is fresh (well inside IDLE_EVICTION), so the idle sweep
        // frees nothing — the least-recently-seen one must still be evicted.
        // Staggered by milliseconds (not seconds) so even the oldest entry
        // stays well under the 600s IDLE_EVICTION threshold.
        for i in 0..MAX_TRACKED_CLIENTS as u32 {
            limiter.clients.insert(
                IpAddr::from(std::net::Ipv4Addr::from(i)),
                ClientBucket {
                    bucket: TokenBucket::new(10),
                    last_seen: now - Duration::from_millis(i as u64),
                },
            );
        }
        let lru_ip = IpAddr::from(std::net::Ipv4Addr::from(MAX_TRACKED_CLIENTS as u32 - 1));
        assert_eq!(limiter.clients.len(), MAX_TRACKED_CLIENTS);

        let fresh: IpAddr = "203.0.113.1".parse().unwrap();
        assert!(limiter.check(fresh).is_ok());

        // The map never grows past capacity, even with no idle entries to sweep.
        assert_eq!(limiter.clients.len(), MAX_TRACKED_CLIENTS);
        assert!(!limiter.clients.contains_key(&lru_ip));
        assert!(limiter.clients.contains_key(&fresh));
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
