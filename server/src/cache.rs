//! Redis caching module for finance-query-server.
//!
//! Provides time-based caching with market-hours awareness.
//! Cache keys are prefixed with "v2:" to avoid conflicts with v1 Python server.

use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

#[cfg(feature = "redis-cache")]
use redis::aio::ConnectionManager;

/// Cache key prefix to avoid conflicts with v1 Python server
const CACHE_PREFIX: &str = "v2:";

/// Cache TTL configuration based on data type
#[derive(Debug, Clone, Copy)]
pub struct CacheTtl {
    /// TTL when market is open (seconds)
    pub market_open: u64,
    /// TTL when market is closed (seconds) - data changes less frequently
    pub market_closed: u64,
}

impl CacheTtl {
    pub const fn new(market_open: u64, market_closed: u64) -> Self {
        Self {
            market_open,
            market_closed,
        }
    }

    /// Get the appropriate TTL based on market status
    pub fn get(&self, market_open: bool) -> u64 {
        match market_open {
            true => self.market_open,
            false => self.market_closed,
        }
    }
}

/// Predefined TTL configurations
pub mod ttl {
    use super::CacheTtl;

    /// Real-time price data: 10s open, 60s closed
    pub const QUOTES: CacheTtl = CacheTtl::new(10, 60);

    /// Market indices: 15s open, 180s closed
    pub const INDICES: CacheTtl = CacheTtl::new(15, 180);

    /// Market movers (actives/gainers/losers): 15s open, 1h closed
    pub const MOVERS: CacheTtl = CacheTtl::new(15, 3600);

    /// Historical data: 60s open, 10m closed
    pub const HISTORICAL: CacheTtl = CacheTtl::new(60, 600);

    /// Chart data: 60s open, 10m closed
    pub const CHART: CacheTtl = CacheTtl::new(60, 600);

    /// Spark data (batch sparklines): 30s open, 5m closed
    pub const SPARK: CacheTtl = CacheTtl::new(30, 300);

    /// Sector data: 5m open, 1h closed
    pub const SECTORS: CacheTtl = CacheTtl::new(300, 3600);

    /// News for symbol: 5m (no market dependency)
    pub const NEWS: CacheTtl = CacheTtl::new(300, 300);

    /// General news: 15m (no market dependency)
    pub const GENERAL_NEWS: CacheTtl = CacheTtl::new(900, 900);

    /// Holders data: 1h (no market dependency)
    pub const HOLDERS: CacheTtl = CacheTtl::new(3600, 3600);

    /// Analysis data: 1h (no market dependency)
    pub const ANALYSIS: CacheTtl = CacheTtl::new(3600, 3600);

    /// Financials: 24h (rarely changes)
    pub const FINANCIALS: CacheTtl = CacheTtl::new(86400, 86400);

    /// Earnings calls list: 24h
    pub const EARNINGS_LIST: CacheTtl = CacheTtl::new(86400, 86400);

    /// Earnings transcript: 7 days (immutable once published)
    pub const TRANSCRIPT: CacheTtl = CacheTtl::new(604800, 604800);

    /// Search results: 1h
    pub const SEARCH: CacheTtl = CacheTtl::new(3600, 3600);

    /// Options chain: 60s open, 10m closed
    pub const OPTIONS: CacheTtl = CacheTtl::new(60, 600);

    /// Indicators: 60s open, 10m closed
    pub const INDICATORS: CacheTtl = CacheTtl::new(60, 600);

    /// Market hours: 5m (changes throughout the day)
    pub const MARKET_HOURS: CacheTtl = CacheTtl::new(300, 300);

    /// Static metadata (quote type, currencies, exchanges): 24h
    pub const METADATA: CacheTtl = CacheTtl::new(86400, 86400);
}

/// Cache client wrapper
#[derive(Clone)]
pub struct Cache {
    #[cfg(feature = "redis-cache")]
    conn: Option<Arc<ConnectionManager>>,
}

impl Cache {
    /// Create a new cache instance
    #[cfg(feature = "redis-cache")]
    pub async fn new(redis_url: Option<&str>) -> Self {
        let conn = if let Some(url) = redis_url {
            match redis::Client::open(url) {
                Ok(client) => match ConnectionManager::new(client).await {
                    Ok(manager) => {
                        tracing::info!("Redis cache connected: {}", url);
                        Some(Arc::new(manager))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to Redis: {}. Caching disabled.", e);
                        None
                    }
                },
                Err(e) => {
                    tracing::warn!("Invalid Redis URL: {}. Caching disabled.", e);
                    None
                }
            }
        } else {
            tracing::info!("No REDIS_URL configured. Caching disabled.");
            None
        };

        Self { conn }
    }

    #[cfg(not(feature = "redis-cache"))]
    pub async fn new(_redis_url: Option<&str>) -> Self {
        tracing::info!("Redis cache feature not enabled. Caching disabled.");
        Self {}
    }

    /// Check if caching is enabled
    #[allow(dead_code)]
    #[cfg(feature = "redis-cache")]
    pub fn is_enabled(&self) -> bool {
        self.conn.is_some()
    }

    #[allow(dead_code)]
    #[cfg(not(feature = "redis-cache"))]
    pub fn is_enabled(&self) -> bool {
        false
    }

    /// Build a cache key with the v2 prefix
    pub fn key(endpoint: &str, params: &[&str]) -> String {
        if params.is_empty() {
            format!("{}{}", CACHE_PREFIX, endpoint)
        } else {
            format!("{}{}:{}", CACHE_PREFIX, endpoint, params.join(":"))
        }
    }

    /// Get a value from cache
    #[cfg(feature = "redis-cache")]
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let conn = self.conn.as_ref()?;
        let mut conn = conn.as_ref().clone();

        let timer = crate::metrics::CacheTimer::new("get");

        match redis::cmd("GET")
            .arg(key)
            .query_async::<Option<String>>(&mut conn)
            .await
        {
            Ok(Some(data)) => match serde_json::from_str(&data) {
                Ok(value) => {
                    tracing::debug!(key = %key, "Cache HIT");
                    crate::metrics::CACHE_HITS.inc();
                    timer.observe();
                    Some(value)
                }
                Err(e) => {
                    tracing::warn!(key = %key, error = %e, "Cache deserialize error");
                    timer.observe();
                    None
                }
            },
            Ok(None) => {
                tracing::debug!(key = %key, "Cache MISS");
                crate::metrics::CACHE_MISSES.inc();
                timer.observe();
                None
            }
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "Cache GET error");
                timer.observe();
                None
            }
        }
    }

    #[cfg(not(feature = "redis-cache"))]
    pub async fn get<T: DeserializeOwned>(&self, _key: &str) -> Option<T> {
        None
    }

    /// Set a value in cache with TTL
    #[cfg(feature = "redis-cache")]
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) {
        let Some(conn) = self.conn.as_ref() else {
            return;
        };

        let mut conn = conn.as_ref().clone();
        let timer = crate::metrics::CacheTimer::new("set");

        let data = match serde_json::to_string(value) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!(key = %key, error = %e, "Cache serialize error");
                timer.observe();
                return;
            }
        };

        if let Err(e) = redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(&data)
            .query_async::<()>(&mut conn)
            .await
        {
            tracing::warn!(key = %key, error = %e, "Cache SET error");
        } else {
            tracing::debug!(key = %key, ttl = ttl_seconds, "Cache SET");
        }

        timer.observe();
    }

    #[cfg(not(feature = "redis-cache"))]
    pub async fn set<T: Serialize>(&self, _key: &str, _value: &T, _ttl_seconds: u64) {}

    /// Get or fetch: returns cached value or fetches and caches it
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: CacheTtl,
        market_open: bool,
        fetch: F,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        // Try cache first
        if let Some(cached) = self.get::<T>(key).await {
            return Ok(cached);
        }

        // Fetch fresh data
        let value = fetch().await?;

        // Cache the result
        let ttl_seconds = ttl.get(market_open);
        self.set(key, &value, ttl_seconds).await;

        Ok(value)
    }
}

/// Check if US stock market is currently open (9:30 AM - 4:00 PM ET, Mon-Fri)
pub fn is_market_open() -> bool {
    use chrono::{Datelike, Timelike, Utc};
    use chrono_tz::America::New_York;

    // Get current time in US Eastern timezone
    let now_et = Utc::now().with_timezone(&New_York);
    let weekday = now_et.weekday();

    // Weekend check
    if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
        return false;
    }

    let hour = now_et.hour();
    let minute = now_et.minute();

    // Market hours: 9:30 AM - 4:00 PM ET
    let after_open = hour > 9 || (hour == 9 && minute >= 30);
    let before_close = hour < 16;

    after_open && before_close
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        assert_eq!(Cache::key("quotes", &["AAPL"]), "v2:quotes:AAPL");
        assert_eq!(
            Cache::key("quotes", &["AAPL", "NVDA"]),
            "v2:quotes:AAPL:NVDA"
        );
        assert_eq!(Cache::key("indices", &[]), "v2:indices");
        assert_eq!(
            Cache::key("chart", &["AAPL", "1d", "1mo"]),
            "v2:chart:AAPL:1d:1mo"
        );
    }

    #[test]
    fn test_ttl_market_hours() {
        let ttl = CacheTtl::new(10, 60);
        assert_eq!(ttl.get(true), 10);
        assert_eq!(ttl.get(false), 60);
    }
}
