//! Symbol discovery handle.
//!
//! Created via [`Providers::discovery`](crate::Providers::discovery).

use std::sync::Arc;

use crate::error::Result;
use crate::models::discovery::reference::{
    ExchangeInfo, ScreenerFilters, ScreenerMatch, SymbolDetails, SymbolMatch,
};
use crate::providers::{Capability, ProviderSet};

/// Symbol discovery backed by configured data providers.
///
/// Unlike [`crate::finance::search`] — a Yahoo-only convenience shortcut —
/// this routes through [`Capability::DISCOVERY`], so it honours the provider
/// priority configured on [`Providers::builder`](crate::Providers::builder)
/// and falls back across providers.
///
/// Created via [`Providers::discovery`](crate::Providers::discovery).
pub struct Discovery {
    providers: Arc<ProviderSet>,
    cache: crate::domains::DomainCache<Vec<SymbolMatch>>,
    details_cache: crate::domains::DomainCache<SymbolDetails>,
}

impl Discovery {
    pub(crate) fn with_providers(providers: Arc<ProviderSet>) -> Self {
        Self {
            providers,
            cache: crate::domains::DomainCache::new(crate::utils::CacheMode::default()),
            details_cache: crate::domains::DomainCache::new(crate::utils::CacheMode::default()),
        }
    }

    /// Cache responses for `ttl` instead of for the handle's lifetime,
    /// deduplicating concurrent identical requests.
    pub fn cache(mut self, ttl: std::time::Duration) -> Self {
        let mode = crate::utils::CacheMode::Ttl(ttl);
        self.cache = crate::domains::DomainCache::new(mode);
        self.details_cache = crate::domains::DomainCache::new(mode);
        self
    }

    /// Disable caching — every call fetches fresh data.
    pub fn no_cache(mut self) -> Self {
        let mode = crate::utils::CacheMode::Off;
        self.cache = crate::domains::DomainCache::new(mode);
        self.details_cache = crate::domains::DomainCache::new(mode);
        self
    }

    /// Search the configured providers' symbol universe.
    ///
    /// Results are cached per `(query, limit)` pair.
    pub async fn search(&self, query: &str, limit: u32) -> Result<Vec<SymbolMatch>> {
        let key = format!("{query}\u{1f}{limit}");
        let providers = Arc::clone(&self.providers);
        let query = query.to_string();
        self.cache
            .get_or_try(key, move || async move {
                providers
                    .fetch(Capability::DISCOVERY, move |p| {
                        let query = query.clone();
                        let p = p.clone();
                        async move {
                            p.as_discovery()
                                .ok_or_else(|| {
                                    p.not_supported(crate::providers::Operation::SymbolSearch)
                                })?
                                .fetch_symbol_search(&query, limit)
                                .await
                        }
                    })
                    .await
            })
            .await
    }

    /// Fetch detailed reference data for one symbol.
    pub async fn details(&self, symbol: &str) -> Result<SymbolDetails> {
        let providers = Arc::clone(&self.providers);
        let symbol = symbol.to_string();
        let key = symbol.clone();
        self.details_cache
            .get_or_try(key, move || async move {
                providers
                    .fetch(Capability::DISCOVERY, move |p| {
                        let symbol = symbol.clone();
                        let p = p.clone();
                        async move {
                            p.as_discovery()
                                .ok_or_else(|| {
                                    p.not_supported(crate::providers::Operation::SymbolDetails)
                                })?
                                .fetch_symbol_details(&symbol)
                                .await
                        }
                    })
                    .await
            })
            .await
    }

    /// Fetch the tradable exchange listing.
    pub async fn exchanges(&self) -> Result<Vec<ExchangeInfo>> {
        self.providers
            .fetch(Capability::DISCOVERY, |p| {
                let p = p.clone();
                async move {
                    p.as_discovery()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Exchanges))?
                        .fetch_exchanges()
                        .await
                }
            })
            .await
    }

    /// Fetch the providers' whole listed-security universe.
    ///
    /// `active = false` asks for delisted securities instead. This is an
    /// unfiltered dump — expect thousands of rows in one response — so prefer
    /// [`search`](Self::search) when you have a query. Cached per `active`.
    /// Currently Alpha Vantage only.
    pub async fn listing_status(&self, active: bool) -> Result<Vec<SymbolMatch>> {
        let providers = Arc::clone(&self.providers);
        self.cache
            .get_or_try(
                format!("listing_status\u{1f}{active}"),
                move || async move {
                    providers
                        .fetch(Capability::DISCOVERY, move |p| {
                            let p = p.clone();
                            async move {
                                p.as_discovery()
                                    .ok_or_else(|| {
                                        p.not_supported(crate::providers::Operation::ListingStatus)
                                    })?
                                    .fetch_listing_status(active)
                                    .await
                            }
                        })
                        .await
                },
            )
            .await
    }

    /// Run a screener query over the providers' universe.
    ///
    /// Not cached — screener filters are open-ended and results are
    /// price-sensitive, so every call fetches fresh.
    pub async fn screener(&self, filters: &ScreenerFilters) -> Result<Vec<ScreenerMatch>> {
        self.providers
            .fetch(Capability::DISCOVERY, |p| {
                let p = p.clone();
                let filters = filters.clone();
                async move {
                    p.as_discovery()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::Screener))?
                        .fetch_screener(&filters)
                        .await
                }
            })
            .await
    }
}
