//! The [`Provider`] identifier.

use serde::de::{Error as DeError, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::*;

/// Typed identifier for a financial data provider.
///
/// Variants are feature-gated: unavailable providers are excluded at compile time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Provider {
    #[default]
    /// Yahoo Finance (always available).
    Yahoo,
    /// Polygon.io (requires `polygon` feature).
    #[cfg(feature = "polygon")]
    Polygon,
    /// Financial Modeling Prep (requires `fmp` feature).
    #[cfg(feature = "fmp")]
    Fmp,
    /// Alpha Vantage (requires `alphavantage` feature).
    #[cfg(feature = "alphavantage")]
    AlphaVantage,
    /// CoinGecko cryptocurrency data (requires `crypto` feature).
    #[cfg(feature = "crypto")]
    CoinGecko,
    /// FRED economic data (requires `fred` feature).
    #[cfg(feature = "fred")]
    Fred,
    /// World Bank Open Data global macro indicators (requires `worldbank` feature, keyless).
    #[cfg(feature = "worldbank")]
    WorldBank,
    /// US Treasury FiscalData (requires `fiscaldata` feature, keyless).
    #[cfg(feature = "fiscaldata")]
    FiscalData,
    /// US Bureau of Labor Statistics (requires `bls` feature; keyless v1,
    /// keyed v2 when `BLS_API_KEY` is set).
    #[cfg(feature = "bls")]
    Bls,
    /// Frankfurter ECB reference exchange rates (requires `frankfurter` feature, keyless).
    #[cfg(feature = "frankfurter")]
    Frankfurter,
    /// Binance public market data (requires `binance` feature, keyless).
    #[cfg(feature = "binance")]
    Binance,
    /// Kraken public market data (requires `kraken` feature, keyless).
    #[cfg(feature = "kraken")]
    Kraken,
    /// FINRA daily short-sale volume (requires `finra` feature, keyless).
    #[cfg(feature = "finra")]
    Finra,
    /// DefiLlama DeFi TVL data (requires `defi` feature, keyless).
    #[cfg(feature = "defi")]
    DefiLlama,
    /// GDELT DOC 2.0 global news search (requires `gdelt` feature, keyless).
    #[cfg(feature = "gdelt")]
    Gdelt,
    /// CFTC Commitments of Traders futures positioning (requires `cftc`
    /// feature, keyless).
    #[cfg(feature = "cftc")]
    Cftc,
    /// Nasdaq market-wide earnings/IPO/dividend/split calendars (requires
    /// `nasdaq` feature, keyless).
    #[cfg(feature = "nasdaq")]
    Nasdaq,
    /// Wikipedia S&P 500 index-constituent table (requires `wikipedia`
    /// feature, keyless).
    #[cfg(feature = "wikipedia")]
    Wikipedia,
    /// Combined House + Senate PTR stock-trade disclosures (requires the
    /// `housetrades` and/or `senatetrades` feature, keyless).
    #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
    CongressTrades,
    /// SEC EDGAR filings (always available, keyless).
    Edgar,
    /// NYSE/NASDAQ market holidays, computed locally from federal-holiday
    /// rules rather than fetched (always available, no network call).
    LocalMarketCalendar,
    /// A static table of major global exchanges (always available, no
    /// network call).
    LocalExchange,
    /// A provider supplied by a downstream crate and registered with
    /// `ProvidersBuilder::with_adapter`. Build one with
    /// [`Provider::custom`], which is the only way to obtain a [`CustomId`].
    ///
    /// Interning is per-process, so a custom id deserializes only after
    /// something in this process has constructed it.
    ///
    /// The id is held as an index rather than a string so that `Provider`
    /// stays small: it is stored in every model's `provider_id`, and a fat
    /// pointer here grows `Candle` by 22%.
    Custom(CustomId),
}

impl Provider {
    /// Parse a provider id string back to the typed variant.
    /// Returns `None` if the string doesn't match any known provider.
    /// Prefer this over string conversion to avoid panics.
    pub fn from_id_str(s: &str) -> Option<Self> {
        match s {
            "yahoo" => Some(Self::Yahoo),
            #[cfg(feature = "polygon")]
            "polygon" => Some(Self::Polygon),
            #[cfg(feature = "fmp")]
            "fmp" => Some(Self::Fmp),
            #[cfg(feature = "alphavantage")]
            "alphavantage" => Some(Self::AlphaVantage),
            #[cfg(feature = "crypto")]
            "coingecko" => Some(Self::CoinGecko),
            #[cfg(feature = "fred")]
            "fred" => Some(Self::Fred),
            #[cfg(feature = "worldbank")]
            "worldbank" => Some(Self::WorldBank),
            #[cfg(feature = "fiscaldata")]
            "fiscaldata" => Some(Self::FiscalData),
            #[cfg(feature = "bls")]
            "bls" => Some(Self::Bls),
            #[cfg(feature = "frankfurter")]
            "frankfurter" => Some(Self::Frankfurter),
            #[cfg(feature = "binance")]
            "binance" => Some(Self::Binance),
            #[cfg(feature = "kraken")]
            "kraken" => Some(Self::Kraken),
            #[cfg(feature = "finra")]
            "finra" => Some(Self::Finra),
            #[cfg(feature = "defi")]
            "defillama" => Some(Self::DefiLlama),
            #[cfg(feature = "gdelt")]
            "gdelt" => Some(Self::Gdelt),
            #[cfg(feature = "cftc")]
            "cftc" => Some(Self::Cftc),
            #[cfg(feature = "nasdaq")]
            "nasdaq" => Some(Self::Nasdaq),
            #[cfg(feature = "wikipedia")]
            "wikipedia" => Some(Self::Wikipedia),
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            "congresstrades" => Some(Self::CongressTrades),
            "edgar" => Some(Self::Edgar),
            "local_market_calendar" => Some(Self::LocalMarketCalendar),
            "local_exchange" => Some(Self::LocalExchange),
            other => lookup(other).map(Self::Custom),
        }
    }

    /// A provider this crate does not build, identified by `id`.
    ///
    /// Interning is process-wide and append-only, so the same id always maps
    /// to the same value and `Provider::custom("x") == Provider::custom("x")`.
    pub fn custom(id: &'static str) -> Self {
        Self::Custom(intern(id))
    }

    /// String identifier matching [`ProviderCore::id`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yahoo => "yahoo",
            #[cfg(feature = "polygon")]
            Self::Polygon => "polygon",
            #[cfg(feature = "fmp")]
            Self::Fmp => "fmp",
            #[cfg(feature = "alphavantage")]
            Self::AlphaVantage => "alphavantage",
            #[cfg(feature = "crypto")]
            Self::CoinGecko => "coingecko",
            #[cfg(feature = "fred")]
            Self::Fred => "fred",
            #[cfg(feature = "worldbank")]
            Self::WorldBank => "worldbank",
            #[cfg(feature = "fiscaldata")]
            Self::FiscalData => "fiscaldata",
            #[cfg(feature = "bls")]
            Self::Bls => "bls",
            #[cfg(feature = "frankfurter")]
            Self::Frankfurter => "frankfurter",
            #[cfg(feature = "binance")]
            Self::Binance => "binance",
            #[cfg(feature = "kraken")]
            Self::Kraken => "kraken",
            #[cfg(feature = "finra")]
            Self::Finra => "finra",
            #[cfg(feature = "defi")]
            Self::DefiLlama => "defillama",
            #[cfg(feature = "gdelt")]
            Self::Gdelt => "gdelt",
            #[cfg(feature = "cftc")]
            Self::Cftc => "cftc",
            #[cfg(feature = "nasdaq")]
            Self::Nasdaq => "nasdaq",
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia => "wikipedia",
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            Self::CongressTrades => "congresstrades",
            Self::Edgar => "edgar",
            Self::LocalMarketCalendar => "local_market_calendar",
            Self::LocalExchange => "local_exchange",
            Self::Custom(id) => id.as_str(),
        }
    }

    /// Every provider variant compiled into this build, regardless of whether
    /// it's actually configured/initialized. Used to compute error-message
    /// hints (see [`Capability::candidate_providers`]) without needing a live
    /// `ProviderSet`.
    pub(crate) fn all() -> Vec<Self> {
        let mut v = vec![Self::Yahoo];
        #[cfg(feature = "polygon")]
        v.push(Self::Polygon);
        #[cfg(feature = "fmp")]
        v.push(Self::Fmp);
        #[cfg(feature = "alphavantage")]
        v.push(Self::AlphaVantage);
        #[cfg(feature = "crypto")]
        v.push(Self::CoinGecko);
        #[cfg(feature = "fred")]
        v.push(Self::Fred);
        #[cfg(feature = "worldbank")]
        v.push(Self::WorldBank);
        #[cfg(feature = "fiscaldata")]
        v.push(Self::FiscalData);
        #[cfg(feature = "bls")]
        v.push(Self::Bls);
        #[cfg(feature = "frankfurter")]
        v.push(Self::Frankfurter);
        #[cfg(feature = "binance")]
        v.push(Self::Binance);
        #[cfg(feature = "kraken")]
        v.push(Self::Kraken);
        #[cfg(feature = "finra")]
        v.push(Self::Finra);
        #[cfg(feature = "defi")]
        v.push(Self::DefiLlama);
        #[cfg(feature = "gdelt")]
        v.push(Self::Gdelt);
        #[cfg(feature = "cftc")]
        v.push(Self::Cftc);
        #[cfg(feature = "nasdaq")]
        v.push(Self::Nasdaq);
        #[cfg(feature = "wikipedia")]
        v.push(Self::Wikipedia);
        #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
        v.push(Self::CongressTrades);
        v.push(Self::Edgar);
        v.push(Self::LocalMarketCalendar);
        v.push(Self::LocalExchange);
        v
    }

    /// Capability bitflags for this provider variant, derived from each
    /// adapter's `as_*` accessor overrides — implementing a capability trait
    /// and declaring it can no longer drift apart. Yahoo is the one exception:
    /// constructing `YahooProvider` needs a live auth handshake, so its set is
    /// a const declared beside its accessor overrides (`yahoo::CAPS`).
    pub(crate) fn capabilities(self) -> Capability {
        match self {
            Self::Yahoo => yahoo::CAPS,
            #[cfg(feature = "polygon")]
            Self::Polygon => ProviderAdapter::capabilities(&polygon::PolygonProvider),
            #[cfg(feature = "fmp")]
            Self::Fmp => ProviderAdapter::capabilities(&fmp::FmpProvider),
            #[cfg(feature = "alphavantage")]
            Self::AlphaVantage => {
                ProviderAdapter::capabilities(&alphavantage::AlphaVantageProvider)
            }
            #[cfg(feature = "crypto")]
            Self::CoinGecko => ProviderAdapter::capabilities(&coingecko::CoinGeckoProvider),
            #[cfg(feature = "fred")]
            Self::Fred => ProviderAdapter::capabilities(&fred::FredProvider),
            #[cfg(feature = "worldbank")]
            Self::WorldBank => ProviderAdapter::capabilities(&worldbank::WorldBankProvider),
            #[cfg(feature = "fiscaldata")]
            Self::FiscalData => ProviderAdapter::capabilities(&fiscaldata::FiscalDataProvider),
            #[cfg(feature = "bls")]
            Self::Bls => ProviderAdapter::capabilities(&bls::BlsProvider),
            #[cfg(feature = "frankfurter")]
            Self::Frankfurter => ProviderAdapter::capabilities(&frankfurter::FrankfurterProvider),
            #[cfg(feature = "binance")]
            Self::Binance => ProviderAdapter::capabilities(&binance::BinanceProvider),
            #[cfg(feature = "kraken")]
            Self::Kraken => ProviderAdapter::capabilities(&kraken::KrakenProvider),
            #[cfg(feature = "finra")]
            Self::Finra => ProviderAdapter::capabilities(&finra::FinraProvider),
            #[cfg(feature = "defi")]
            Self::DefiLlama => ProviderAdapter::capabilities(&defillama::DefiLlamaProvider),
            #[cfg(feature = "gdelt")]
            Self::Gdelt => ProviderAdapter::capabilities(&gdelt::GdeltProvider),
            #[cfg(feature = "cftc")]
            Self::Cftc => ProviderAdapter::capabilities(&cftc::CftcProvider),
            #[cfg(feature = "nasdaq")]
            Self::Nasdaq => ProviderAdapter::capabilities(&nasdaq::NasdaqProvider),
            #[cfg(feature = "wikipedia")]
            Self::Wikipedia => ProviderAdapter::capabilities(&wikipedia::WikipediaProvider),
            #[cfg(any(feature = "housetrades", feature = "senatetrades"))]
            Self::CongressTrades => {
                ProviderAdapter::capabilities(&congresstrades::CongressTradesProvider)
            }
            Self::Edgar => ProviderAdapter::capabilities(&edgar::EdgarProvider),
            Self::LocalMarketCalendar => {
                ProviderAdapter::capabilities(&market_calendar::LocalMarketCalendarProvider)
            }
            Self::LocalExchange => {
                ProviderAdapter::capabilities(&local_exchanges::LocalExchangeProvider)
            }
            Self::Custom(_) => Capability::NONE,
        }
    }
}

/// Index of a custom provider id, obtained from [`Provider::custom`].
///
/// Opaque so that only an interned id can be named: the index is meaningless
/// outside the process that registered it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CustomId(u16);

impl CustomId {
    /// The id this index was interned from.
    pub fn as_str(self) -> &'static str {
        // Indices only come from `intern`, which never removes an entry.
        registry()
            .read()
            .ok()
            .and_then(|ids| ids.get(self.0 as usize).copied())
            .unwrap_or("custom")
    }
}

fn registry() -> &'static std::sync::RwLock<Vec<&'static str>> {
    static IDS: std::sync::OnceLock<std::sync::RwLock<Vec<&'static str>>> =
        std::sync::OnceLock::new();
    IDS.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

fn intern(id: &'static str) -> CustomId {
    let mut ids = match registry().write() {
        Ok(ids) => ids,
        Err(poisoned) => poisoned.into_inner(),
    };
    let index = match ids.iter().position(|existing| *existing == id) {
        Some(index) => index,
        None => {
            ids.push(id);
            ids.len() - 1
        }
    };
    CustomId(index as u16)
}

fn lookup(id: &str) -> Option<CustomId> {
    registry()
        .read()
        .ok()?
        .iter()
        .position(|existing| *existing == id)
        .map(|index| CustomId(index as u16))
}

impl Serialize for Provider {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// Borrowed, owned, and escaped inputs all land in `visit_str`, so no shape
/// allocates. `Cow`'s own `Deserialize` always yields `Owned`.
struct IdVisitor;

impl serde::de::Visitor<'_> for IdVisitor {
    type Value = Provider;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a provider id")
    }

    fn visit_str<E: DeError>(self, v: &str) -> std::result::Result<Provider, E> {
        Provider::from_id_str(v).ok_or_else(|| E::invalid_value(Unexpected::Str(v), &self))
    }
}

impl<'de> Deserialize<'de> for Provider {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(IdVisitor)
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
