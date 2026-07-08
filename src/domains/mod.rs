//! Domain-specific query handles — non-equity asset classes.
//!
//! These types are constructable only through [`Providers`](crate::Providers)
//! factory methods — they share the same provider connections and configuration.
//!
//! Each handle maps to a [`Capability`](crate::Capability) and routes through
//! the multi-provider dispatch system, making multi-source aggregation
//! a first-class concept rather than an opt-in on Ticker/Tickers.

// ── Caching ─────────────────────────────────────────────────────────

use crate::error::Result;
use crate::utils::CacheEntry;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

/// Optional per-handle response cache with request deduplication.
///
/// Keyed by a `String` so a handle can cache multiple variants (e.g. a
/// crypto coin priced in different `vs_currency` values); single-result
/// handles use the empty string. When no TTL is configured (the default),
/// every call fetches fresh — preserving the stateless behavior.
pub(crate) struct DomainCache<V> {
    ttl: Option<Duration>,
    entries: RwLock<HashMap<String, CacheEntry<V>>>,
    guard: Mutex<()>,
}

impl<V: Clone> DomainCache<V> {
    pub(crate) fn new(ttl: Option<Duration>) -> Self {
        Self {
            ttl,
            entries: RwLock::new(HashMap::new()),
            guard: Mutex::new(()),
        }
    }

    /// Return a fresh cached value for `key`, or run `f` to fetch it.
    /// Concurrent identical misses collapse to a single upstream call.
    pub(crate) async fn get_or_try<F, Fut>(&self, key: String, f: F) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
    {
        let Some(ttl) = self.ttl else {
            return f().await;
        };

        if let Some(entry) = self.entries.read().await.get(&key)
            && entry.is_fresh(ttl)
        {
            return Ok(entry.value.clone());
        }

        // Dedup: hold the guard so concurrent identical misses don't all fetch.
        let _g = self.guard.lock().await;
        if let Some(entry) = self.entries.read().await.get(&key)
            && entry.is_fresh(ttl)
        {
            return Ok(entry.value.clone());
        }

        let value = f().await?;
        self.entries
            .write()
            .await
            .insert(key, CacheEntry::new(value.clone()));
        Ok(value)
    }
}

// ── Macros ──────────────────────────────────────────────────────────

/// Generate a single-field domain handle struct with internal constructor,
/// a string accessor, and an optional response cache (`.cache(ttl)`).
/// Callers add fetch methods manually.
macro_rules! domain_handle {
    // Chartable variant — adds a `Chart` response cache read by `chart()`.
    ($(#[$meta:meta])* pub struct $name:ident { $field:ident, $accessor:ident } cache: $val:ty, chart) => {
        $(#[$meta])*
        pub struct $name {
            $field: std::sync::Arc<str>,
            providers: std::sync::Arc<crate::providers::ProviderSet>,
            cache: crate::domains::DomainCache<$val>,
            chart_cache: crate::domains::DomainCache<crate::models::chart::Chart>,
        }

        impl $name {
            pub(crate) fn with_providers(
                $field: std::sync::Arc<str>,
                providers: std::sync::Arc<crate::providers::ProviderSet>,
            ) -> Self {
                Self {
                    $field,
                    providers,
                    cache: crate::domains::DomainCache::new(None),
                    chart_cache: crate::domains::DomainCache::new(None),
                }
            }

            /// Cache responses for `ttl`, deduplicating concurrent identical
            /// requests. Off by default (each call fetches fresh).
            pub fn cache(mut self, ttl: std::time::Duration) -> Self {
                self.cache = crate::domains::DomainCache::new(Some(ttl));
                self.chart_cache = crate::domains::DomainCache::new(Some(ttl));
                self
            }

            /// The handle's identifier string.
            pub fn $accessor(&self) -> &str {
                &self.$field
            }
        }
    };

    // Two-key chartable variant (e.g. forex pairs) — two identifier fields
    // with their own accessors instead of one shared field/accessor.
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $(#[$meta1:meta])* $field1:ident, $accessor1:ident,
            $(#[$meta2:meta])* $field2:ident, $accessor2:ident $(,)?
        } cache: $val:ty, chart
    ) => {
        $(#[$meta])*
        pub struct $name {
            $field1: std::sync::Arc<str>,
            $field2: std::sync::Arc<str>,
            providers: std::sync::Arc<crate::providers::ProviderSet>,
            cache: crate::domains::DomainCache<$val>,
            chart_cache: crate::domains::DomainCache<crate::models::chart::Chart>,
        }

        impl $name {
            pub(crate) fn with_providers(
                $field1: std::sync::Arc<str>,
                $field2: std::sync::Arc<str>,
                providers: std::sync::Arc<crate::providers::ProviderSet>,
            ) -> Self {
                Self {
                    $field1,
                    $field2,
                    providers,
                    cache: crate::domains::DomainCache::new(None),
                    chart_cache: crate::domains::DomainCache::new(None),
                }
            }

            /// Cache responses for `ttl`, deduplicating concurrent identical
            /// requests. Off by default (each call fetches fresh).
            pub fn cache(mut self, ttl: std::time::Duration) -> Self {
                self.cache = crate::domains::DomainCache::new(Some(ttl));
                self.chart_cache = crate::domains::DomainCache::new(Some(ttl));
                self
            }

            $(#[$meta1])*
            pub fn $accessor1(&self) -> &str {
                &self.$field1
            }

            $(#[$meta2])*
            pub fn $accessor2(&self) -> &str {
                &self.$field2
            }
        }
    };

    // Non-chartable variant.
    ($(#[$meta:meta])* pub struct $name:ident { $field:ident, $accessor:ident } cache: $val:ty) => {
        $(#[$meta])*
        pub struct $name {
            $field: std::sync::Arc<str>,
            providers: std::sync::Arc<crate::providers::ProviderSet>,
            cache: crate::domains::DomainCache<$val>,
        }

        impl $name {
            pub(crate) fn with_providers(
                $field: std::sync::Arc<str>,
                providers: std::sync::Arc<crate::providers::ProviderSet>,
            ) -> Self {
                Self {
                    $field,
                    providers,
                    cache: crate::domains::DomainCache::new(None),
                }
            }

            /// Cache responses for `ttl`, deduplicating concurrent identical
            /// requests. Off by default (each call fetches fresh).
            pub fn cache(mut self, ttl: std::time::Duration) -> Self {
                self.cache = crate::domains::DomainCache::new(Some(ttl));
                self
            }

            /// The handle's identifier string.
            pub fn $accessor(&self) -> &str {
                &self.$field
            }
        }
    };
}

/// Fetch via the provider dispatch — single symbol field, no extra args.
/// Routes through the handle's cache. Use inside a method body.
macro_rules! fetch_via {
    ($self:expr, $field:ident, $cap:ident, $fetch:ident, $ret:ty) => {{
        let __sym = $self.$field.clone();
        let __providers = std::sync::Arc::clone(&$self.providers);
        $self
            .cache
            .get_or_try(String::new(), move || async move {
                __providers
                    .fetch(crate::providers::Capability::$cap, move |p| {
                        let __s = __sym.clone();
                        let p = p.clone();
                        async move { p.$fetch(&__s).await }
                    })
                    .await
            })
            .await
    }};
}

/// Fetch via the provider dispatch — two identifier fields (e.g. forex's
/// `from`/`to`), cached under the empty-string key. Use inside a method body.
#[allow(unused_macros)]
macro_rules! fetch_via_two {
    ($self:expr, $field1:ident, $field2:ident, $cap:ident, $fetch:ident, $ret:ty) => {{
        let __a = $self.$field1.clone();
        let __b = $self.$field2.clone();
        let __providers = std::sync::Arc::clone(&$self.providers);
        $self
            .cache
            .get_or_try(String::new(), move || async move {
                __providers
                    .fetch(crate::providers::Capability::$cap, move |p| {
                        let __x = __a.clone();
                        let __y = __b.clone();
                        let p = p.clone();
                        async move { p.$fetch(&__x, &__y).await }
                    })
                    .await
            })
            .await
    }};
}

/// Fetch with one extra string argument (e.g. `vs_currency` for crypto),
/// keyed in the cache by that argument.
#[allow(unused_macros)]
macro_rules! fetch_via_with {
    ($self:expr, $field:ident, $cap:ident, $fetch:ident, $arg:expr, $ret:ty) => {{
        let __sym = $self.$field.clone();
        let __arg = ($arg).to_string();
        let __providers = std::sync::Arc::clone(&$self.providers);
        $self
            .cache
            .get_or_try(__arg.clone(), move || async move {
                __providers
                    .fetch(crate::providers::Capability::$cap, move |p| {
                        let __s = __sym.clone();
                        let __a = __arg.clone();
                        let p = p.clone();
                        async move { p.$fetch(&__s, &__a).await }
                    })
                    .await
            })
            .await
    }};
}

/// Fetch chart candles via the `CHART` capability, keyed by `(interval, range)`.
/// `$sym` is the chart-ready symbol expression for this asset class.
#[allow(unused_macros)]
macro_rules! fetch_chart_via {
    ($self:expr, $sym:expr, $interval:expr, $range:expr) => {{
        let __sym: String = $sym;
        let __interval = $interval;
        let __range = $range;
        let __providers = std::sync::Arc::clone(&$self.providers);
        let __key = format!("{}:{}:{}", __sym, __interval, __range);
        $self
            .chart_cache
            .get_or_try(__key, move || async move {
                __providers
                    .fetch(crate::providers::Capability::CHART, move |p| {
                        let __s = __sym.clone();
                        let p = p.clone();
                        async move { p.fetch_chart(&__s, __interval, __range).await }
                    })
                    .await
            })
            .await
    }};
}

/// Generate `indicators()`, `indicator()`, and `risk()` for a chartable handle
/// whose `chart(interval, range)` takes no extra arguments. All three reuse the
/// cached `chart()`; `risk()` annualises with the handle's `$cal`
/// ([`TradingCalendar`](crate::risk::TradingCalendar)). Crypto is hand-written
/// (its `chart()` also takes a `vs_currency`).
#[allow(unused_macros)]
macro_rules! impl_chartable_analytics {
    ($name:ident, $cal:expr) => {
        impl $name {
            /// Compute all technical indicators from this handle's chart data.
            #[cfg(feature = "indicators")]
            pub async fn indicators(
                &self,
                interval: crate::Interval,
                range: crate::TimeRange,
            ) -> crate::error::Result<crate::indicators::IndicatorsSummary> {
                let chart = self.chart(interval, range).await?;
                Ok(crate::indicators::summary::calculate_indicators(
                    &chart.candles,
                ))
            }

            /// Compute a single technical indicator from this handle's chart data.
            #[cfg(feature = "indicators")]
            pub async fn indicator(
                &self,
                indicator: crate::indicators::Indicator,
                interval: crate::Interval,
                range: crate::TimeRange,
            ) -> crate::error::Result<crate::indicators::IndicatorResult> {
                let chart = self.chart(interval, range).await?;
                Ok(crate::indicators::compute_indicator(indicator, &chart)?)
            }

            /// Compute a risk summary (VaR, Sharpe/Sortino/Calmar, max drawdown)
            /// from this handle's chart data. Annualised with this asset class's
            /// trading calendar, so non-daily intervals scale correctly. `beta`
            /// is always `None` (no benchmark for non-equity handles).
            #[cfg(feature = "risk")]
            pub async fn risk(
                &self,
                interval: crate::Interval,
                range: crate::TimeRange,
            ) -> crate::error::Result<crate::risk::RiskSummary> {
                let chart = self.chart(interval, range).await?;
                Ok(crate::risk::compute_risk_summary_with_periods(
                    &chart.candles,
                    None,
                    crate::risk::periods_per_year(interval, $cal),
                ))
            }
        }
    };
}

// ── Modules ─────────────────────────────────────────────────────────

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub(crate) mod commodities;
#[cfg(any(
    feature = "alphavantage",
    feature = "crypto",
    feature = "fmp",
    feature = "polygon"
))]
pub(crate) mod crypto;
#[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
pub(crate) mod economic;
pub(crate) mod filings;
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub(crate) mod forex;
#[cfg(feature = "polygon")]
pub(crate) mod futures;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub(crate) mod indices;

// ── Re-exports ──────────────────────────────────────────────────────

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub use commodities::Commodity;
#[cfg(any(
    feature = "alphavantage",
    feature = "crypto",
    feature = "fmp",
    feature = "polygon"
))]
pub use crypto::CryptoCoin;
#[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
pub use economic::EconomicIndicator;
pub use filings::Filings;
#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub use forex::ForexPair;
#[cfg(feature = "polygon")]
pub use futures::FuturesContract;
#[cfg(any(feature = "polygon", feature = "fmp"))]
pub use indices::Index;

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn err() -> FinanceError {
        FinanceError::NoProviderAvailable {
            operation: crate::providers::Capability::QUOTE,
            candidates: Vec::new(),
        }
    }

    #[tokio::test]
    async fn no_ttl_fetches_every_call() {
        let cache: DomainCache<u32> = DomainCache::new(None);
        let calls = Arc::new(AtomicUsize::new(0));
        for _ in 0..3 {
            let c = calls.clone();
            let v = cache
                .get_or_try(String::new(), move || async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<u32, FinanceError>(42)
                })
                .await
                .unwrap();
            assert_eq!(v, 42);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn ttl_caches_within_window() {
        let cache: DomainCache<u32> = DomainCache::new(Some(Duration::from_secs(60)));
        let calls = Arc::new(AtomicUsize::new(0));
        for _ in 0..3 {
            let c = calls.clone();
            let v = cache
                .get_or_try(String::new(), move || async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<u32, FinanceError>(7)
                })
                .await
                .unwrap();
            assert_eq!(v, 7);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distinct_keys_cached_separately() {
        let cache: DomainCache<String> = DomainCache::new(Some(Duration::from_secs(60)));
        let calls = Arc::new(AtomicUsize::new(0));
        for key in ["usd", "eur", "usd", "eur"] {
            let c = calls.clone();
            let owned = key.to_string();
            let v = cache
                .get_or_try(key.to_string(), move || async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<String, FinanceError>(owned)
                })
                .await
                .unwrap();
            assert_eq!(v, key);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn errors_are_not_cached() {
        let cache: DomainCache<u32> = DomainCache::new(Some(Duration::from_secs(60)));
        let calls = Arc::new(AtomicUsize::new(0));

        let c = calls.clone();
        let first = cache
            .get_or_try(String::new(), move || async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<u32, FinanceError>(err())
            })
            .await;
        assert!(first.is_err());

        let c = calls.clone();
        let second = cache
            .get_or_try(String::new(), move || async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<u32, FinanceError>(5)
            })
            .await
            .unwrap();
        assert_eq!(second, 5);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_misses_dedup_to_one_fetch() {
        let cache: Arc<DomainCache<u32>> =
            Arc::new(DomainCache::new(Some(Duration::from_secs(60))));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let cache = Arc::clone(&cache);
            let c = calls.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_try(String::new(), move || async move {
                        c.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        Ok::<u32, FinanceError>(1)
                    })
                    .await
                    .unwrap()
            }));
        }
        for h in handles {
            assert_eq!(h.await.unwrap(), 1);
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
