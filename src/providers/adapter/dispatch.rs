//! [`ProviderAdapter`]: the accessors dispatch walks to reach a capability.

use super::super::Capability;
use crate::error::Result;

use super::ProviderCore;
use super::equity::*;
use super::markets::*;

/// A configured provider as seen by [`super::ProviderSet`] dispatch: lifecycle
/// plus one `as_*` accessor per capability. Override an accessor to
/// `Some(self)` for each capability trait the provider implements.
///
/// [`capabilities`](Self::capabilities) is derived from which accessors are
/// overridden, so a provider cannot advertise a capability it does not serve.
#[async_trait::async_trait]
pub trait ProviderAdapter: ProviderCore {
    /// Initialize this provider. Called once during construction.
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Best-effort estimate of remaining rate-limit budget — tokens
    /// currently available in this provider's own token bucket, for
    /// [`super::Providers::health`]. Peeking never consumes a
    /// token.
    ///
    /// `None` by default: providers with no local rate limiter to peek (e.g.
    /// Yahoo) or whose singleton hasn't been initialized yet return `None`
    /// rather than a misleading number.
    fn rate_limit_remaining(&self) -> Option<f64> {
        None
    }

    /// Override to `Some(self)` to serve [`Capability::QUOTE`].
    fn as_quote(&self) -> Option<&dyn QuoteProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::CHART`].
    fn as_chart(&self) -> Option<&dyn ChartProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::FUNDAMENTALS`].
    fn as_fundamentals(&self) -> Option<&dyn FundamentalsProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::CORPORATE`].
    fn as_corporate(&self) -> Option<&dyn CorporateProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::OPTIONS`].
    fn as_options(&self) -> Option<&dyn OptionsProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::FILINGS`].
    fn as_filings(&self) -> Option<&dyn FilingsProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::DISCOVERY`].
    fn as_discovery(&self) -> Option<&dyn DiscoveryProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::CALENDAR`].
    fn as_calendar(&self) -> Option<&dyn CalendarProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::MARKET`].
    fn as_market(&self) -> Option<&dyn MarketProvider> {
        None
    }
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
    /// Override to `Some(self)` to serve [`Capability::CRYPTO`].
    fn as_crypto(&self) -> Option<&dyn CryptoProvider> {
        None
    }
    #[cfg(any(
        feature = "alphavantage",
        feature = "bls",
        feature = "fiscaldata",
        feature = "fred",
        feature = "polygon",
        feature = "worldbank"
    ))]
    /// Override to `Some(self)` to serve [`Capability::ECONOMIC`].
    fn as_economic(&self) -> Option<&dyn EconomicProvider> {
        None
    }
    #[cfg(any(
        feature = "alphavantage",
        feature = "fmp",
        feature = "frankfurter",
        feature = "gdelt",
        feature = "polygon"
    ))]
    /// Override to `Some(self)` to serve [`Capability::FOREX`].
    fn as_forex(&self) -> Option<&dyn ForexProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::INDICES`].
    fn as_indices(&self) -> Option<&dyn IndicesProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::FUTURES`].
    fn as_futures(&self) -> Option<&dyn FuturesProvider> {
        None
    }
    /// Override to `Some(self)` to serve [`Capability::COMMODITIES`].
    fn as_commodities(&self) -> Option<&dyn CommoditiesProvider> {
        None
    }

    /// Derived from the accessors — a provider supports a capability iff the
    /// matching `as_*` accessor returns `Some`. Never override.
    fn capabilities(&self) -> Capability {
        let mut caps = Capability::NONE;
        if self.as_quote().is_some() {
            caps = caps | Capability::QUOTE;
        }
        if self.as_chart().is_some() {
            caps = caps | Capability::CHART;
        }
        if self.as_fundamentals().is_some() {
            caps = caps | Capability::FUNDAMENTALS;
        }
        if self.as_corporate().is_some() {
            caps = caps | Capability::CORPORATE;
        }
        if self.as_options().is_some() {
            caps = caps | Capability::OPTIONS;
        }
        if self.as_filings().is_some() {
            caps = caps | Capability::FILINGS;
        }
        if self.as_market().is_some() {
            caps = caps | Capability::MARKET;
        }
        if self.as_discovery().is_some() {
            caps = caps | Capability::DISCOVERY;
        }
        if self.as_calendar().is_some() {
            caps = caps | Capability::CALENDAR;
        }
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
        if self.as_crypto().is_some() {
            caps = caps | Capability::CRYPTO;
        }
        #[cfg(any(
            feature = "alphavantage",
            feature = "bls",
            feature = "fiscaldata",
            feature = "fred",
            feature = "polygon",
            feature = "worldbank"
        ))]
        if self.as_economic().is_some() {
            caps = caps | Capability::ECONOMIC;
        }
        #[cfg(any(
            feature = "alphavantage",
            feature = "fmp",
            feature = "frankfurter",
            feature = "gdelt",
            feature = "polygon"
        ))]
        if self.as_forex().is_some() {
            caps = caps | Capability::FOREX;
        }
        if self.as_indices().is_some() {
            caps = caps | Capability::INDICES;
        }
        if self.as_futures().is_some() {
            caps = caps | Capability::FUTURES;
        }
        if self.as_commodities().is_some() {
            caps = caps | Capability::COMMODITIES;
        }
        caps
    }
}
