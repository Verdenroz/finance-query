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
    /// Fetch one independently addressable delisted page (zero-based).
    ///
    /// `page` must be below 10,000 and `limit` in 1–100.
    ///
    /// Callers fetching pages in parallel must check contiguous coverage,
    /// repeated records and the empty terminal page. Unlike
    /// [`delisted_stocks_page`](Self::delisted_stocks_page), this does not carry
    /// scan-progress validation.
    pub async fn delisted_stocks_page_at(&self, page: u32, limit: u32) -> Result<Vec<SymbolMatch>> {
        dispatch_via!(
            self,
            DISCOVERY,
            as_discovery,
            DelistedStocksPageAt,
            fetch_delisted_stocks_page_at,
            [],
            page,
            limit
        )
    }

    /// Fetch one delisted-stock page of 1–100 rows. Persist the returned cursor
    /// with the records and pass it back with the same `limit`.
    pub async fn delisted_stocks_page(
        &self,
        limit: u32,
        cursor: Option<&crate::PageCursor>,
    ) -> Result<crate::ProviderPage<SymbolMatch>> {
        self.providers
            .fetch_paged(
                Capability::DISCOVERY,
                crate::Operation::DelistedStocksPage,
                &limit.to_string(),
                cursor,
                |p| {
                    let p = Arc::clone(p);
                    async move {
                        p.as_discovery()
                            .ok_or_else(|| p.not_supported(crate::Operation::DelistedStocksPage))?
                            .fetch_delisted_stocks_page(limit, cursor)
                            .await
                    }
                },
            )
            .await
    }

    /// Fetch the current provider-wide symbol directory. Active status can be unknown.
    pub async fn stock_list(&self) -> Result<Vec<SymbolMatch>> {
        dispatch_via!(
            self,
            DISCOVERY,
            as_discovery,
            StockList,
            fetch_stock_list,
            []
        )
    }

    /// Fetch one numbered bulk-profile part. No additional parts are downloaded or cached.
    /// An empty part is returned as empty; access and transport failures remain errors.
    pub async fn company_profiles_bulk(&self, part: u32) -> Result<Vec<crate::CompanyProfile>> {
        dispatch_via!(
            self,
            DISCOVERY,
            as_discovery,
            CompanyProfilesBulk,
            fetch_company_profiles_bulk,
            [],
            part
        )
    }

    /// Download one dated stock-directory page. Continuations never change providers.
    pub async fn stock_listings_page(
        &self,
        request: &crate::StockListingRequest,
        cursor: Option<&crate::PageCursor>,
    ) -> Result<crate::ProviderPage<crate::StockListing>> {
        self.providers
            .fetch_paged(
                Capability::DISCOVERY,
                crate::Operation::StockListingsPage,
                &request.cursor_identity(),
                cursor,
                |p| {
                    let p = Arc::clone(p);
                    async move {
                        p.as_discovery()
                            .ok_or_else(|| p.not_supported(crate::Operation::StockListingsPage))?
                            .fetch_stock_listings_page(request, cursor)
                            .await
                    }
                },
            )
            .await
    }

    /// Fetch details as of YYYY-MM-DD, independently of undated detail cache entries.
    pub async fn details_at(&self, symbol: &str, date: &str) -> Result<SymbolDetails> {
        crate::models::discovery::listings::date(date)?;
        let key = serde_json::to_string(&("details_at", symbol, date))?;
        self.details_cache
            .get_or_try(key, || async {
                self.providers
                    .fetch(Capability::DISCOVERY, |p| {
                        let p = Arc::clone(p);
                        async move {
                            p.as_discovery()
                                .ok_or_else(|| p.not_supported(crate::Operation::SymbolDetailsAt))?
                                .fetch_symbol_details_at(symbol, date)
                                .await
                        }
                    })
                    .await
            })
            .await
    }

    /// Fetch open-ended stock-type codes for a market locale, such as us.
    pub async fn stock_types(&self, locale: &str) -> Result<Vec<crate::StockType>> {
        self.providers
            .fetch(Capability::DISCOVERY, |p| {
                let p = Arc::clone(p);
                async move {
                    p.as_discovery()
                        .ok_or_else(|| p.not_supported(crate::Operation::StockTypes))?
                        .fetch_stock_types(locale)
                        .await
                }
            })
            .await
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
    /// there). Alpha Vantage and FMP serve both. FMP's active list includes
    /// global instruments; callers must filter their intended stock universe.
    /// FMP delisted listings include all pages and preserve IPO/delisting dates.
    /// A failed page fails the call; partial results are not returned or cached.
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
