//! Symbol discovery handle.
//!
//! Created via [`Providers::discovery`](crate::Providers::discovery).

use std::sync::Arc;

use crate::error::Result;
use crate::models::discovery::reference::{
    ExchangeInfo, ScreenerFilters, ScreenerMatch, SymbolDetails, SymbolMatch,
};
use crate::providers::Capability;

domain_handle! {
    /// Symbol discovery backed by configured data providers.
    ///
    /// Unlike [`crate::finance::search`] — a Yahoo-only convenience shortcut —
    /// this routes through [`Capability::DISCOVERY`], so it honours the provider
    /// priority configured on [`Providers::builder`](crate::Providers::builder)
    /// and falls back across providers.
    ///
    /// Created via [`Providers::discovery`](crate::Providers::discovery).
    pub struct Discovery
    caches: {
        cache: Vec<SymbolMatch>,
        details_cache: SymbolDetails,
    }
}

impl Discovery {
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
        dispatch_via!(
            self,
            DISCOVERY,
            as_discovery,
            Exchanges,
            fetch_exchanges,
            []
        )
    }

    /// Fetch the providers' whole listed-security universe.
    ///
    /// `active = false` asks for delisted securities instead. This is an
    /// unfiltered dump — expect thousands of rows in one response — so prefer
    /// [`search`](Self::search) when you have a query. Cached per `active`.
    /// EDGAR serves `active = true` keylessly from SEC's bulk ticker files
    /// (no exchange-listing history, so `active = false` isn't supported
    /// there); Alpha Vantage and FMP serve both.
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
        let filters = filters.clone();
        dispatch_via!(
            self,
            DISCOVERY,
            as_discovery,
            Screener,
            fetch_screener,
            [filters],
            &filters
        )
    }
}
