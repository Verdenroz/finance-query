use crate::adapters::yahoo::client::ClientConfig;
use crate::error::Result;
use crate::providers::{
    Fetch, Provider, ProviderHealth, ProviderSet, RetryPolicy, Routes, build_providers,
};
use std::sync::Arc;
use std::time::Duration;

/// Central provider configuration shared across query handles.
///
/// Build once with [`Providers::builder`], then create lightweight
/// [`Ticker`](crate::Ticker) handles that share the same underlying
/// provider connections and authentication.
///
/// # Example
///
/// ```no_run
/// use finance_query::{Providers, Provider, Fetch, Capability};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let providers = Providers::builder()
///     .route(Capability::QUOTE, [Provider::Yahoo])
///     .fetch(Fetch::Sequential)
///     .build().await?;
///
/// // All Ticker handles share the same Arc<ProviderSet>
/// let aapl = providers.ticker("AAPL").build().await?;
/// let nvda = providers.ticker("NVDA").logo().build().await?;
/// # Ok(())
/// # }
/// ```
pub struct Providers {
    pub(crate) set: Arc<ProviderSet>,
    lang: String,
}

impl std::fmt::Debug for Providers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Providers")
            .field("set", &self.set)
            .field("lang", &self.lang)
            .finish()
    }
}

impl Providers {
    /// Create a builder for configuring providers.
    pub fn builder() -> ProvidersBuilder {
        ProvidersBuilder::default()
    }

    /// Wrap a [`ProviderSet`] assembled by hand.
    ///
    /// The lower-level counterpart to [`builder`](Self::builder), for a caller
    /// that has already built its own adapters and route table. Nothing is
    /// initialised: [`ProviderAdapter::initialize`](crate::ProviderAdapter::initialize)
    /// is the builder's job, so a hand-built set must be ready to use.
    pub fn from_set(set: Arc<ProviderSet>) -> Self {
        Self::from_set_with_lang(set, ClientConfig::default().lang)
    }

    /// [`from_set`](Self::from_set) with an explicit language for the handles
    /// this set creates.
    pub fn from_set_with_lang(set: Arc<ProviderSet>, lang: impl Into<String>) -> Self {
        Self {
            set,
            lang: lang.into(),
        }
    }

    /// The underlying set, for `TickerBuilder::with_provider_set` and friends.
    pub fn provider_set(&self) -> &Arc<ProviderSet> {
        &self.set
    }

    /// Create a [`TickerBuilder`](crate::TickerBuilder) pre-wired to this provider set.
    ///
    /// The returned builder accepts the same optional configuration as
    /// [`Ticker::builder`](crate::Ticker::builder) (`.cache()`, `.logo()`,
    /// `.format()`) before calling `.build()`.
    ///
    /// The language configured via [`ProvidersBuilder::lang`] or
    /// [`ProvidersBuilder::region`] is inherited (override with `.lang()` on
    /// the returned builder). With the `translation` feature, a non-English
    /// language translates text fields automatically.
    pub fn ticker(&self, symbol: impl Into<String>) -> crate::TickerBuilder {
        crate::Ticker::builder(symbol)
            .lang(self.lang.clone())
            .with_provider_set(Arc::clone(&self.set))
    }

    /// Create a [`TickersBuilder`](crate::TickersBuilder) pre-wired to this provider set.
    ///
    /// The returned builder accepts the same optional configuration as
    /// [`Tickers::builder`](crate::Tickers::builder) (`.cache()`,
    /// `.max_concurrency()`, `.logo()`, `.format()`) before calling `.build()`.
    ///
    /// The language configured via [`ProvidersBuilder::lang`] or
    /// [`ProvidersBuilder::region`] is inherited (override with `.lang()` on
    /// the returned builder). With the `translation` feature, a non-English
    /// language translates text fields automatically.
    pub fn tickers<S, I>(&self, symbols: I) -> crate::TickersBuilder
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        crate::Tickers::builder(symbols)
            .lang(self.lang.clone())
            .with_provider_set(Arc::clone(&self.set))
    }

    /// Create a [`CryptoCoin`](crate::CryptoCoin) handle backed by this provider set.
    #[cfg(any(
        feature = "alphavantage",
        feature = "binance",
        feature = "crypto",
        feature = "defi",
        feature = "fmp",
        feature = "gdelt",
        feature = "kraken",
        feature = "polygon"
    ))]
    pub fn crypto(&self, id: impl Into<String>) -> crate::domains::CryptoCoin {
        crate::domains::CryptoCoin::with_providers(id.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`ForexPair`](crate::ForexPair) handle backed by this provider set.
    #[cfg(any(
        feature = "alphavantage",
        feature = "fmp",
        feature = "frankfurter",
        feature = "gdelt",
        feature = "polygon"
    ))]
    pub fn forex(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> crate::domains::ForexPair {
        crate::domains::ForexPair::with_providers(
            from.into().into(),
            to.into().into(),
            Arc::clone(&self.set),
        )
    }

    /// Create an [`EconomicIndicator`](crate::EconomicIndicator) handle backed by this provider set.
    #[cfg(any(
        feature = "alphavantage",
        feature = "bls",
        feature = "fiscaldata",
        feature = "fred",
        feature = "polygon",
        feature = "worldbank"
    ))]
    pub fn economic(&self, series_id: impl Into<String>) -> crate::domains::EconomicIndicator {
        crate::domains::EconomicIndicator::with_providers(
            series_id.into().into(),
            Arc::clone(&self.set),
        )
    }

    /// Create an [`Index`](crate::Index) handle backed by this provider set.
    pub fn index(&self, symbol: impl Into<String>) -> crate::domains::Index {
        crate::domains::Index::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`FuturesContract`](crate::FuturesContract) handle backed by this provider set.
    pub fn futures(&self, symbol: impl Into<String>) -> crate::domains::FuturesContract {
        crate::domains::FuturesContract::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`Commodity`](crate::Commodity) handle backed by this provider set.
    pub fn commodity(&self, symbol: impl Into<String>) -> crate::domains::Commodity {
        crate::domains::Commodity::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`Discovery`](crate::Discovery) handle backed by this provider set.
    ///
    /// Routes symbol search, reference data, and screening through
    /// [`Capability::DISCOVERY`](crate::Capability::DISCOVERY). Distinct from
    /// [`crate::finance::search`], which is a Yahoo-only shortcut.
    pub fn discovery(&self) -> crate::domains::Discovery {
        crate::domains::Discovery::with_providers(Arc::clone(&self.set))
    }

    /// Create a [`MarketCalendar`](crate::MarketCalendar) handle backed by this provider set.
    ///
    /// Routes market-wide earnings/IPO/dividend/split/economic calendars
    /// through [`Capability::CALENDAR`](crate::Capability::CALENDAR).
    pub fn calendar(&self) -> crate::domains::MarketCalendar {
        crate::domains::MarketCalendar::with_providers(Arc::clone(&self.set))
    }

    /// Create a [`Market`](crate::Market) handle backed by this provider set.
    ///
    /// Routes sector/industry performance and movers through
    /// [`Capability::MARKET`](crate::Capability::MARKET). Movers work on the
    /// default keyless route (Yahoo screeners); the sector/industry
    /// statistics need a keyed provider (FMP).
    pub fn market(&self) -> crate::domains::Market {
        crate::domains::Market::with_providers(Arc::clone(&self.set))
    }

    /// Create an [`EconomicCatalog`](crate::EconomicCatalog) handle backed by
    /// this provider set.
    ///
    /// Routes series search and category/release browsing through
    /// [`Capability::ECONOMIC`](crate::Capability::ECONOMIC). Unlike
    /// [`economic`](Self::economic) it takes no series id — it is how you find
    /// one.
    #[cfg(any(feature = "fred", feature = "alphavantage", feature = "polygon"))]
    pub fn economic_catalog(&self) -> crate::domains::EconomicCatalog {
        crate::domains::EconomicCatalog::with_providers(Arc::clone(&self.set))
    }

    /// Create a [`Snapshot`](crate::Snapshot) handle backed by this provider set.
    ///
    /// Routes cross-market snapshots through
    /// [`Capability::QUOTE`](crate::Capability::QUOTE). Needs a provider whose
    /// snapshot endpoint spans asset classes — currently Polygon only, which is
    /// why the handle is gated on that feature.
    #[cfg(feature = "polygon")]
    pub fn snapshot(&self) -> crate::domains::Snapshot {
        crate::domains::Snapshot::with_providers(Arc::clone(&self.set))
    }

    /// Create a [`Filings`](crate::Filings) handle backed by this provider set.
    ///
    /// Always available — EDGAR is auto-injected when no other FILINGS provider
    /// is configured.
    pub fn filings(&self, symbol: impl Into<String>) -> crate::domains::Filings {
        crate::domains::Filings::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Snapshot recent health for every configured provider.
    ///
    /// Each [`ProviderHealth`] entry reflects up to the last 20 dispatch
    /// outcomes recorded in-process for that provider (recency window is
    /// internal and unspecified beyond "recent"), plus a best-effort
    /// rate-limit budget estimate where the provider exposes one. Purely
    /// observational — it does not affect routing or retries.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::Providers;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let providers = Providers::builder().build().await?;
    /// for health in providers.health() {
    ///     println!("{:?}: healthy={}", health.provider, health.is_healthy);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn health(&self) -> Vec<ProviderHealth> {
        self.set.health()
    }
}

/// Builder for [`Providers`].
pub struct ProvidersBuilder {
    provider_ids: Vec<Provider>,
    adapters: Vec<Arc<dyn crate::ProviderAdapter>>,
    config: ClientConfig,
    routes: Routes,
    retry: Option<RetryPolicy>,
}

impl std::fmt::Debug for ProvidersBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProvidersBuilder")
            .field("provider_ids", &self.provider_ids)
            .field(
                "adapters",
                &self.adapters.iter().map(|a| a.id()).collect::<Vec<_>>(),
            )
            .field("routes", &self.routes)
            .field("retry", &self.retry)
            .finish()
    }
}

impl Default for ProvidersBuilder {
    fn default() -> Self {
        Self {
            provider_ids: vec![Provider::Yahoo],
            adapters: Vec::new(),
            config: ClientConfig::default(),
            routes: Routes::new(Fetch::Sequential),
            retry: None,
        }
    }
}

impl ProvidersBuilder {
    /// Configure how providers are queried. Default: `Sequential`.
    ///
    /// Use [`Fetch::Sequential`] or [`Fetch::Parallel`].
    pub fn fetch(mut self, mode: Fetch) -> Self {
        self.routes.fetch = mode;
        self
    }

    /// Route a capability to a specific provider priority list.
    ///
    /// Providers referenced in the route are automatically added to the
    /// initialisation list if not already present. If omitted for a capability,
    /// Yahoo is used as default.
    pub fn route(
        mut self,
        cap: crate::providers::Capability,
        providers: impl IntoIterator<Item = Provider>,
    ) -> Self {
        let providers: Vec<Provider> = providers.into_iter().collect();
        for provider in &providers {
            // A Custom id has no arm in `build_providers`; it arrives only
            // through `with_adapter`.
            if matches!(provider, Provider::Custom(_)) {
                continue;
            }
            if !self.provider_ids.contains(provider) {
                self.provider_ids.push(*provider);
            }
        }
        self.routes.map.insert(cap, providers);
        self
    }

    /// Set the region (automatically sets lang and region code).
    pub fn region(mut self, region: crate::constants::Region) -> Self {
        self.config.lang = region.lang().to_string();
        self.config.region = region.region().to_string();
        self
    }

    /// Set the language code (e.g., "en-US", "ja-JP").
    ///
    /// Inherited by every `Ticker`/`Tickers` handle created from the built
    /// [`Providers`]. With the `translation` feature, a non-English language
    /// translates text fields on those handles automatically.
    pub fn lang(mut self, lang: impl Into<String>) -> Self {
        self.config.lang = lang.into();
        self
    }

    /// Set the region code (e.g., "US", "JP", "DE").
    ///
    /// For standard countries, prefer `.region()` instead to ensure correct
    /// lang/region pairing.
    pub fn region_code(mut self, region: impl Into<String>) -> Self {
        self.config.region = region.into();
        self
    }

    /// Set the HTTP request timeout.
    pub fn timeout(mut self, t: Duration) -> Self {
        self.config.timeout = t;
        self
    }

    /// Set the proxy URL.
    pub fn proxy(mut self, p: impl Into<String>) -> Self {
        self.config.proxy = Some(p.into());
        self
    }

    /// Opt into retrying `FinanceError::RateLimited` errors during dispatch
    /// See [`RetryPolicy`] for the exact semantics.
    ///
    /// **Default is no retry** — omitting this call preserves the exact
    /// prior behavior: a `RateLimited` error is treated like any other
    /// failure and dispatch moves straight to the next routed provider.
    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = Some(policy);
        self
    }

    /// Register an adapter this crate does not build itself.
    ///
    /// Route to it by the id its [`ProviderCore::id`](crate::ProviderCore::id)
    /// returns, usually [`Provider::Custom`]. Registering alone does not route
    /// anything: a capability with no explicit route still falls back to its
    /// default provider.
    ///
    /// ```no_run
    /// # use std::sync::Arc;
    /// # use finance_query::{Capability, Provider, Providers, ProviderAdapter};
    /// # async fn f(my_adapter: Arc<dyn ProviderAdapter>) -> finance_query::Result<()> {
    /// let providers = Providers::builder()
    ///     .with_adapter(my_adapter)
    ///     .route(Capability::ECONOMIC, [Provider::Custom("my-source")])
    ///     .build()
    ///     .await?;
    /// # let _ = providers;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn with_adapter(mut self, adapter: Arc<dyn crate::ProviderAdapter>) -> Self {
        self.adapters.push(adapter);
        self
    }

    /// Build the [`Providers`] instance, initialising all configured providers.
    pub async fn build(self) -> Result<Providers> {
        #[cfg(feature = "translation")]
        crate::translation::Lang::parse(&self.config.lang)?;
        for adapter in &self.adapters {
            let id = adapter.id();
            if let Provider::Custom(name) = id
                && Provider::from_id_str(name).is_some()
            {
                return Err(crate::FinanceError::InvalidParameter {
                    param: "adapter".to_string(),
                    reason: format!("custom provider id `{name}` collides with a built-in"),
                });
            }
            let duplicate = self.provider_ids.contains(&id)
                || self.adapters.iter().filter(|a| a.id() == id).count() > 1;
            if duplicate {
                return Err(crate::FinanceError::InvalidParameter {
                    param: "adapter".to_string(),
                    reason: format!("provider id `{id}` is already configured"),
                });
            }
        }
        let lang = self.config.lang.clone();
        let set = build_providers(&self.provider_ids, self.adapters, &self.config, self.routes)
            .await?
            .with_retry_policy(self.retry);
        Ok(Providers {
            set: Arc::new(set),
            lang,
        })
    }
}
