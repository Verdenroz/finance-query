//! Forex currency-pair query handle.
//!
//! Created via [`Providers::forex`](crate::Providers::forex).

use crate::domains::DomainCache;
use crate::error::Result;
use crate::providers::{Capability, ProviderSet};
use std::sync::Arc;
use std::time::Duration;

/// A foreign-exchange currency pair backed by configured data providers.
///
/// Created via [`Providers::forex`](crate::Providers::forex).
pub struct ForexPair {
    from: Arc<str>,
    to: Arc<str>,
    providers: Arc<ProviderSet>,
    cache: DomainCache<crate::models::forex::ForexQuote>,
}

impl ForexPair {
    pub(crate) fn with_providers(
        from: Arc<str>,
        to: Arc<str>,
        providers: Arc<ProviderSet>,
    ) -> Self {
        Self {
            from,
            to,
            providers,
            cache: DomainCache::new(None),
        }
    }

    /// Cache responses for `ttl`, deduplicating concurrent identical requests.
    /// Off by default (each call fetches fresh).
    pub fn cache(mut self, ttl: Duration) -> Self {
        self.cache = DomainCache::new(Some(ttl));
        self
    }

    /// The base (from) currency code (e.g., `"USD"`).
    pub fn from(&self) -> &str {
        &self.from
    }

    /// The quote (to) currency code (e.g., `"EUR"`).
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Fetch the current exchange rate for this currency pair.
    pub async fn quote(&self) -> Result<crate::models::forex::ForexQuote> {
        let from = self.from.clone();
        let to = self.to.clone();
        let providers = Arc::clone(&self.providers);
        self.cache
            .get_or_try(String::new(), move || async move {
                providers
                    .fetch(Capability::FOREX, move |p| {
                        let f = from.clone();
                        let t = to.clone();
                        let p = p.clone();
                        async move { p.fetch_forex_quote(&f, &t).await }
                    })
                    .await
            })
            .await
    }
}
