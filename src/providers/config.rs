use crate::adapters::yahoo::client::ClientConfig;
use crate::error::Result;
use crate::providers::{Fetch, Provider, ProviderSet, Routes, build_providers};
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
///     .route(Capability::QUOTE, &[Provider::Yahoo])
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
}

impl Providers {
    /// Create a builder for configuring providers.
    pub fn builder() -> ProvidersBuilder {
        ProvidersBuilder::default()
    }

    /// Create a [`TickerBuilder`](crate::TickerBuilder) pre-wired to this provider set.
    ///
    /// The returned builder accepts the same optional configuration as
    /// [`Ticker::builder`](crate::Ticker::builder) (`.cache()`, `.logo()`,
    /// `.format()`) before calling `.build()`.
    pub fn ticker(&self, symbol: impl Into<String>) -> crate::TickerBuilder {
        crate::Ticker::builder(symbol).with_provider_set(Arc::clone(&self.set))
    }

    /// Create a [`TickersBuilder`](crate::TickersBuilder) pre-wired to this provider set.
    ///
    /// The returned builder accepts the same optional configuration as
    /// [`Tickers::builder`](crate::Tickers::builder) (`.cache()`,
    /// `.max_concurrency()`, `.logo()`, `.format()`) before calling `.build()`.
    pub fn tickers<S, I>(&self, symbols: I) -> crate::TickersBuilder
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        crate::Tickers::builder(symbols).with_provider_set(Arc::clone(&self.set))
    }

    /// Create a [`CryptoCoin`](crate::CryptoCoin) handle backed by this provider set.
    #[cfg(any(
        feature = "alphavantage",
        feature = "crypto",
        feature = "fmp",
        feature = "polygon"
    ))]
    pub fn crypto(&self, id: impl Into<String>) -> crate::domains::CryptoCoin {
        crate::domains::CryptoCoin::with_providers(id.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`ForexPair`](crate::ForexPair) handle backed by this provider set.
    #[cfg(any(feature = "alphavantage", feature = "fmp", feature = "polygon"))]
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
    #[cfg(any(feature = "alphavantage", feature = "polygon", feature = "fred"))]
    pub fn economic(&self, series_id: impl Into<String>) -> crate::domains::EconomicIndicator {
        crate::domains::EconomicIndicator::with_providers(
            series_id.into().into(),
            Arc::clone(&self.set),
        )
    }

    /// Create an [`Index`](crate::Index) handle backed by this provider set.
    #[cfg(any(feature = "polygon", feature = "fmp"))]
    pub fn index(&self, symbol: impl Into<String>) -> crate::domains::Index {
        crate::domains::Index::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`FuturesContract`](crate::FuturesContract) handle backed by this provider set.
    #[cfg(feature = "polygon")]
    pub fn futures(&self, symbol: impl Into<String>) -> crate::domains::FuturesContract {
        crate::domains::FuturesContract::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`Commodity`](crate::Commodity) handle backed by this provider set.
    #[cfg(any(feature = "fmp", feature = "alphavantage"))]
    pub fn commodity(&self, symbol: impl Into<String>) -> crate::domains::Commodity {
        crate::domains::Commodity::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }

    /// Create a [`Filings`](crate::Filings) handle backed by this provider set.
    ///
    /// Always available — EDGAR is auto-injected when no other FILINGS provider
    /// is configured.
    pub fn filings(&self, symbol: impl Into<String>) -> crate::domains::Filings {
        crate::domains::Filings::with_providers(symbol.into().into(), Arc::clone(&self.set))
    }
}

/// Builder for [`Providers`].
pub struct ProvidersBuilder {
    provider_ids: Vec<Provider>,
    config: ClientConfig,
    routes: Routes,
}

impl Default for ProvidersBuilder {
    fn default() -> Self {
        Self {
            provider_ids: vec![Provider::Yahoo],
            config: ClientConfig::default(),
            routes: Routes::new(Fetch::Sequential),
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
    pub fn route(mut self, cap: crate::providers::Capability, providers: &[Provider]) -> Self {
        self.routes.map.insert(cap, providers.to_vec());
        for provider in providers {
            if !self.provider_ids.contains(provider) {
                self.provider_ids.push(*provider);
            }
        }
        self
    }

    /// Set the region (automatically sets lang and region code).
    pub fn region(mut self, region: crate::constants::Region) -> Self {
        self.config.lang = region.lang().to_string();
        self.config.region = region.region().to_string();
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

    /// Build the [`Providers`] instance, initialising all configured providers.
    pub async fn build(self) -> Result<Providers> {
        let set = build_providers(&self.provider_ids, &self.config, self.routes).await?;
        Ok(Providers { set: Arc::new(set) })
    }
}
