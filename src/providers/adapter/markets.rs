//! Capability traits over markets rather than a single equity symbol.

use super::super::Operation;
use crate::error::Result;

use super::ProviderCore;

/// [`crate::Capability::DISCOVERY`] — symbol search, reference data, exchanges,
/// screeners.
#[async_trait::async_trait]
pub trait DiscoveryProvider: ProviderCore {
    /// Search the provider's symbol universe by free-text query.
    async fn fetch_symbol_search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>>;

    /// Fetch detailed reference data for a single symbol.
    async fn fetch_symbol_details(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::discovery::reference::SymbolDetails> {
        Err(self.not_supported(Operation::SymbolDetails))
    }

    /// Fetch the provider's tradable exchange listing.
    async fn fetch_exchanges(
        &self,
    ) -> Result<Vec<crate::models::discovery::reference::ExchangeInfo>> {
        Err(self.not_supported(Operation::Exchanges))
    }

    /// Run a screener query over the provider's universe.
    async fn fetch_screener(
        &self,
        _filters: &crate::models::discovery::reference::ScreenerFilters,
    ) -> Result<Vec<crate::models::discovery::reference::ScreenerMatch>> {
        Err(self.not_supported(Operation::Screener))
    }

    /// Fetch the provider's whole listed-security universe.
    ///
    /// `active = false` asks for delisted securities instead. Unlike
    /// [`fetch_symbol_search`](Self::fetch_symbol_search) this is an unfiltered
    /// dump, so expect thousands of rows in one response.
    async fn fetch_listing_status(
        &self,
        _active: bool,
    ) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
        Err(self.not_supported(Operation::ListingStatus))
    }
}

/// [`crate::Capability::CALENDAR`] — market-wide calendars.
#[async_trait::async_trait]
pub trait CalendarProvider: ProviderCore {
    /// Fetch a market-wide calendar over `[from, to]` (`YYYY-MM-DD` dates).
    ///
    /// One method rather than one per kind — providers serve all kinds from the
    /// same calendar family, and `kind.operation()` still reports the precise
    /// [`Operation`] in `NotSupported` errors.
    async fn fetch_market_calendar(
        &self,
        kind: crate::models::calendar::market::CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>>;
}

/// [`crate::Capability::MARKET`] — sector/industry performance and movers.
///
/// Movers is the required primary (every current implementor serves it);
/// the sector/industry statistics default to `NotSupported` since coverage
/// is ragged (FMP serves all of them; Yahoo and Alpha Vantage only movers).
#[async_trait::async_trait]
pub trait MarketProvider: ProviderCore {
    /// Fetch the market movers list for `direction`.
    async fn fetch_market_movers(
        &self,
        direction: crate::models::market::performance::MoverDirection,
    ) -> Result<Vec<crate::models::market::performance::MoverQuote>>;

    /// Fetch aggregate performance for every sector.
    async fn fetch_sector_performance(
        &self,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformance>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }

    /// Fetch historical aggregate sector performance, most recent first.
    async fn fetch_sector_performance_history(
        &self,
        _limit: u32,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
        Err(self.not_supported(Operation::SectorPerformanceHistory))
    }

    /// Fetch sector price/earnings ratios.
    async fn fetch_sector_pe(&self) -> Result<Vec<crate::models::market::performance::SectorPe>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }

    /// Fetch industry price/earnings ratios.
    ///
    /// NOTE: FMP is the only route among the providers integrated here.
    /// Yahoo's screener fan-out backs `fetch_sector_pe` across 11 sectors,
    /// but industries run to roughly 160 and the thin ones carry too few
    /// sampled P/Es to aggregate into a publishable number.
    async fn fetch_industry_pe(
        &self,
    ) -> Result<Vec<crate::models::market::performance::IndustryPe>> {
        Err(self.not_supported(Operation::SectorPerformance))
    }
}

/// [`crate::Capability::CRYPTO`] — cryptocurrency quotes.
#[async_trait::async_trait]
pub trait CryptoProvider: ProviderCore {
    /// Fetch a quote for one coin, priced in `vs_currency`.
    async fn fetch_crypto_quote(
        &self,
        id: &str,
        vs_currency: &str,
    ) -> Result<crate::models::crypto::CryptoQuote>;

    /// Fetch total value locked in a DeFi protocol.
    #[cfg(feature = "defi")]
    async fn fetch_protocol_tvl(
        &self,
        _protocol: &str,
    ) -> Result<crate::models::crypto::defi::ProtocolTvl> {
        Err(self.not_supported(Operation::ProtocolTvl))
    }

    /// Fetch a DeFi protocol's TVL history, oldest first.
    #[cfg(feature = "defi")]
    async fn fetch_protocol_tvl_history(
        &self,
        _protocol: &str,
    ) -> Result<Vec<crate::models::crypto::defi::TvlPoint>> {
        Err(self.not_supported(Operation::ProtocolTvlHistory))
    }

    /// Fetch coins/nfts/categories trending in the last 24h (CoinGecko only).
    #[cfg(feature = "crypto")]
    async fn fetch_crypto_trending(&self) -> Result<Vec<crate::models::crypto::TrendingCoin>> {
        Err(self.not_supported(Operation::CryptoTrending))
    }

    /// Fetch aggregate global cryptocurrency market statistics (CoinGecko only).
    #[cfg(feature = "crypto")]
    async fn fetch_crypto_global(&self) -> Result<crate::models::crypto::GlobalCryptoStats> {
        Err(self.not_supported(Operation::CryptoGlobal))
    }

    /// Fetch market-wide crypto news, newest first.
    async fn fetch_crypto_news(
        &self,
        _limit: u32,
    ) -> Result<Vec<crate::models::corporate::news::News>> {
        Err(self.not_supported(Operation::CryptoNews))
    }
}

/// [`crate::Capability::ECONOMIC`] — macro-economic data series.
#[async_trait::async_trait]
pub trait EconomicProvider: ProviderCore {
    /// Fetch observations for one macro-economic series.
    async fn fetch_economic_series(
        &self,
        series_id: &str,
    ) -> Result<crate::models::economic::EconomicSeries>;

    /// Fetch a series as it stood on `date` (`YYYY-MM-DD`) rather than as
    /// currently revised — the point-in-time view backtests need.
    ///
    /// NOTE: this vintage/realtime-window concept is unique to FRED/ALFRED
    /// among the providers integrated here — WorldBank, FiscalData, and BLS
    /// all serve only the latest value per period, with no revision
    /// history. Stays FRED-only.
    async fn fetch_economic_series_as_of(
        &self,
        _series_id: &str,
        _date: &str,
    ) -> Result<crate::models::economic::EconomicSeries> {
        Err(self.not_supported(Operation::EconomicSeriesAsOf))
    }

    /// Search the provider's series catalog by free text.
    ///
    /// NOTE: WorldBank and BLS treat series ids as opaque, live-validated
    /// strings with no local catalog; FiscalData carries only 7 curated
    /// series. None is a meaningful substitute for FRED's live full-text
    /// search over its ~800k series, so this stays FRED-only.
    async fn fetch_economic_search(
        &self,
        _query: &str,
        _limit: u32,
    ) -> Result<Vec<crate::models::economic::EconomicSeriesMatch>> {
        Err(self.not_supported(Operation::EconomicSearch))
    }

    /// List the child categories of `parent_id` in the series category tree.
    ///
    /// NOTE: none of WorldBank/FiscalData/BLS models a category/topic tree
    /// in this crate. Stays FRED-only.
    async fn fetch_economic_categories(
        &self,
        _parent_id: i64,
    ) -> Result<Vec<crate::models::economic::EconomicCategory>> {
        Err(self.not_supported(Operation::EconomicCategories))
    }

    /// List the provider's scheduled data releases.
    ///
    /// NOTE: no keyless provider integrated here models a scheduled-release
    /// entity. Stays FRED-only.
    async fn fetch_economic_releases(
        &self,
    ) -> Result<Vec<crate::models::economic::EconomicRelease>> {
        Err(self.not_supported(Operation::EconomicReleases))
    }
}

/// [`crate::Capability::FOREX`] — currency-pair quotes.
#[async_trait::async_trait]
pub trait ForexProvider: ProviderCore {
    /// Fetch the exchange rate for one currency pair.
    async fn fetch_forex_quote(
        &self,
        from: &str,
        to: &str,
    ) -> Result<crate::models::forex::ForexQuote>;

    /// Fetch market-wide forex news, newest first.
    async fn fetch_forex_news(
        &self,
        _limit: u32,
    ) -> Result<Vec<crate::models::corporate::news::News>> {
        Err(self.not_supported(Operation::ForexNews))
    }
}

/// [`crate::Capability::INDICES`] — stock market index quotes.
#[async_trait::async_trait]
pub trait IndicesProvider: ProviderCore {
    /// Fetch a quote for one market index.
    async fn fetch_indices_quote(&self, symbol: &str)
    -> Result<crate::models::indices::IndexQuote>;

    /// Fetch the current constituents of a major index.
    async fn fetch_index_constituents(
        &self,
        _index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituent>> {
        Err(self.not_supported(Operation::IndexConstituents))
    }

    /// Fetch historical constituent changes of a major index.
    async fn fetch_index_constituent_changes(
        &self,
        _index: crate::models::indices::MajorIndex,
    ) -> Result<Vec<crate::models::indices::IndexConstituentChange>> {
        Err(self.not_supported(Operation::IndexConstituentChanges))
    }
}

/// [`crate::Capability::FUTURES`] — futures contract quotes.
#[async_trait::async_trait]
pub trait FuturesProvider: ProviderCore {
    /// Fetch a quote for one futures contract.
    async fn fetch_futures_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::futures::FuturesQuote>;

    /// Fetch weekly CFTC Commitments of Traders positioning for a futures
    /// symbol, broken down by trader category.
    #[cfg(feature = "cftc")]
    async fn fetch_commitments_of_traders(
        &self,
        _symbol: &str,
    ) -> Result<crate::models::futures::cot::CommitmentsOfTraders> {
        Err(self.not_supported(Operation::CommitmentsOfTraders))
    }
}

/// [`crate::Capability::COMMODITIES`] — commodity price quotes.
#[async_trait::async_trait]
pub trait CommoditiesProvider: ProviderCore {
    /// Fetch a quote for one commodity.
    async fn fetch_commodities_quote(
        &self,
        symbol: &str,
    ) -> Result<crate::models::commodities::CommodityQuote>;
}
