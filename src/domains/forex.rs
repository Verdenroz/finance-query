//! Forex currency-pair query handle.
//!
//! Created via [`Providers::forex`](crate::Providers::forex).

use crate::constants::{Interval, TimeRange};
use crate::domains::DomainCache;
use crate::error::Result;
use crate::models::chart::Chart;
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
    chart_cache: DomainCache<Chart>,
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
            chart_cache: DomainCache::new(None),
        }
    }

    /// Cache responses for `ttl`, deduplicating concurrent identical requests.
    /// Off by default (each call fetches fresh).
    pub fn cache(mut self, ttl: Duration) -> Self {
        self.cache = DomainCache::new(Some(ttl));
        self.chart_cache = DomainCache::new(Some(ttl));
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

    /// Fetch historical OHLCV candles for this currency pair.
    ///
    /// The pair is mapped to the `CHART` route's symbol form `"{FROM}{TO}=X"`
    /// (e.g. `"USDEUR=X"`, the Yahoo FX convention).
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        let symbol = format!("{}{}=X", self.from, self.to);
        let providers = Arc::clone(&self.providers);
        let key = format!("{symbol}:{interval}:{range}");
        self.chart_cache
            .get_or_try(key, move || async move {
                providers
                    .fetch(Capability::CHART, move |p| {
                        let s = symbol.clone();
                        let p = p.clone();
                        async move { p.fetch_chart(&s, interval, range).await }
                    })
                    .await
            })
            .await
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, range: TimeRange) -> Result<Chart> {
        self.chart(range.default_interval(), range).await
    }
}

impl_chartable_analytics!(ForexPair, crate::risk::TradingCalendar::Forex);
