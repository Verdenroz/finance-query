//! API keys scoped to one [`Providers`](crate::Providers) instance.
//!
//! The keyed adapters read their credentials from process-global
//! [`OnceLock`](std::sync::OnceLock) singletons, which allows exactly one key
//! per provider per process. A key configured on a builder is instead published
//! into a task-local for the duration of a dispatch call, so two `Providers`
//! can hold different keys for the same provider, and a key can be rotated
//! without restarting.
//!
//! `build_client` resolves in order: scoped key, singleton, environment
//! variable. Anything dispatched outside a scope behaves exactly as before.

// Everything below is reached from the keyed adapters' `build_client`, which
// only exists when one of their features is enabled.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, OnceLock, RwLock, Weak};
use std::time::Duration;

use crate::rate_limiter::RateLimiter;

#[derive(Clone)]
pub(crate) struct ScopedKey {
    pub(crate) api_key: String,
    pub(crate) timeout: Duration,
    limiter: Arc<OnceLock<Arc<RateLimiter>>>,
}

impl ScopedKey {
    pub(crate) fn new(api_key: String, timeout: Duration) -> Self {
        Self {
            api_key,
            timeout,
            limiter: Arc::new(OnceLock::new()),
        }
    }

    /// Rate limits are enforced per API key upstream, so each distinct key gets
    /// its own bucket rather than sharing the singleton's.
    pub(crate) fn limiter(&self, provider_key: &'static str, rate: f64) -> Arc<RateLimiter> {
        Arc::clone(
            self.limiter
                .get_or_init(|| shared_limiter(provider_key, &self.api_key, rate)),
        )
    }
}

pub(crate) type KeyMap = HashMap<&'static str, ScopedKey>;

tokio::task_local! {
    static SCOPED: Arc<KeyMap>;
}

type LimiterMap = HashMap<(&'static str, String), Weak<RateLimiter>>;

/// Weak so a rotated-out key's bucket is dropped with the `Providers` holding
/// it. The strong reference lives in that instance's [`ScopedKey`].
static LIMITERS: LazyLock<RwLock<LimiterMap>> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn shared_limiter(provider_key: &'static str, api_key: &str, rate: f64) -> Arc<RateLimiter> {
    let id = (provider_key, api_key.to_string());
    if let Some(limiter) = LIMITERS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(&id)
        .and_then(Weak::upgrade)
    {
        return limiter;
    }
    let mut limiters = LIMITERS.write().unwrap_or_else(|e| e.into_inner());
    if let Some(limiter) = limiters.get(&id).and_then(Weak::upgrade) {
        return limiter;
    }
    limiters.retain(|_, held| held.strong_count() > 0);
    let limiter = Arc::new(RateLimiter::new(rate));
    limiters.insert(id, Arc::downgrade(&limiter));
    limiter
}

fn tracked_limiters() -> usize {
    LIMITERS.read().unwrap_or_else(|e| e.into_inner()).len()
}

pub(crate) fn scoped_key(provider_key: &str) -> Option<ScopedKey> {
    SCOPED
        .try_with(|keys| keys.get(provider_key).cloned())
        .ok()
        .flatten()
}

pub(crate) async fn scope<F>(keys: Arc<KeyMap>, f: F) -> F::Output
where
    F: std::future::Future,
{
    if keys.is_empty() {
        return f.await;
    }
    SCOPED.scope(keys, f).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&'static str, &str)]) -> Arc<KeyMap> {
        Arc::new(
            pairs
                .iter()
                .map(|(p, k)| {
                    (
                        *p,
                        ScopedKey::new((*k).to_string(), Duration::from_secs(30)),
                    )
                })
                .collect(),
        )
    }

    #[tokio::test]
    async fn no_scope_yields_no_key() {
        assert!(scoped_key("fmp").is_none());
    }

    #[tokio::test]
    async fn a_scope_publishes_its_own_keys() {
        scope(map(&[("fmp", "abc")]), async {
            assert_eq!(scoped_key("fmp").map(|k| k.api_key), Some("abc".into()));
            assert!(scoped_key("polygon").is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn an_empty_map_leaves_the_scope_unset() {
        scope(Arc::new(KeyMap::new()), async {
            assert!(scoped_key("fmp").is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn concurrent_scopes_do_not_see_each_other() {
        let one = tokio::spawn(scope(map(&[("fmp", "first")]), async {
            tokio::task::yield_now().await;
            scoped_key("fmp").map(|k| k.api_key)
        }));
        let two = tokio::spawn(scope(map(&[("fmp", "second")]), async {
            tokio::task::yield_now().await;
            scoped_key("fmp").map(|k| k.api_key)
        }));
        assert_eq!(one.await.unwrap(), Some("first".into()));
        assert_eq!(two.await.unwrap(), Some("second".into()));
    }

    #[test]
    fn each_key_gets_its_own_limiter() {
        let a = ScopedKey::new("key-a".into(), Duration::from_secs(30));
        let b = ScopedKey::new("key-b".into(), Duration::from_secs(30));
        assert!(Arc::ptr_eq(&a.limiter("fmp", 5.0), &a.limiter("fmp", 5.0)));
        assert!(!Arc::ptr_eq(&a.limiter("fmp", 5.0), &b.limiter("fmp", 5.0)));
    }

    #[test]
    fn two_instances_of_one_key_share_a_bucket() {
        let first = ScopedKey::new("shared".into(), Duration::from_secs(30));
        let second = ScopedKey::new("shared".into(), Duration::from_secs(30));
        assert!(Arc::ptr_eq(
            &first.limiter("polygon", 5.0),
            &second.limiter("polygon", 5.0)
        ));
    }

    #[test]
    fn a_dropped_key_stops_being_tracked() {
        let before = tracked_limiters();
        {
            let key = ScopedKey::new("transient-xyz".into(), Duration::from_secs(30));
            let _live = key.limiter("fred", 5.0);
            assert!(tracked_limiters() > before);
        }
        let _prune =
            ScopedKey::new("prune-trigger".into(), Duration::from_secs(30)).limiter("fred", 5.0);
        assert!(
            !LIMITERS
                .read()
                .unwrap()
                .contains_key(&("fred", "transient-xyz".to_string()))
        );
    }
}
