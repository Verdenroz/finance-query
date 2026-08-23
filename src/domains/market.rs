//! Market-wide calendar and performance handles.
//!
//! Created via [`Providers::calendar`](crate::Providers::calendar) and
//! [`Providers::market`](crate::Providers::market).

use std::sync::Arc;

use crate::error::Result;
use crate::models::calendar::market::{CalendarKind, MarketCalendarEntry};
use crate::models::chart::Candle;
use crate::models::market::performance::{
    IndustryPe, MoverDirection, MoverQuote, SectorPe, SectorPerformance,
};
use crate::providers::Capability;
use crate::providers::ProviderSet;

domain_handle! {
    /// Market-wide event calendars backed by configured data providers.
    ///
    /// Routes through [`Capability::CALENDAR`]. Unlike
    /// [`Ticker::calendar`](crate::Ticker::calendar), which builds a per-symbol
    /// timeline, these span the whole market over a date range.
    ///
    /// Created via [`Providers::calendar`](crate::Providers::calendar).
    pub struct MarketCalendar
    caches: { cache: Vec<MarketCalendarEntry> }
}

impl MarketCalendar {
    /// Fetch a calendar of `kind` over `[from, to]` (`YYYY-MM-DD` dates).
    ///
    /// Cached per `(kind, from, to)`.
    pub async fn fetch(
        &self,
        kind: CalendarKind,
        from: &str,
        to: &str,
    ) -> Result<Vec<MarketCalendarEntry>> {
        let key = format!("{kind:?}\u{1f}{from}\u{1f}{to}");
        let providers = Arc::clone(&self.providers);
        let (from, to) = (from.to_string(), to.to_string());
        self.cache
            .get_or_try(key, move || async move {
                providers
                    .fetch(Capability::CALENDAR, move |p| {
                        let (from, to) = (from.clone(), to.clone());
                        let p = p.clone();
                        async move {
                            p.as_calendar()
                                .ok_or_else(|| p.not_supported(kind.operation()))?
                                .fetch_market_calendar(kind, &from, &to)
                                .await
                        }
                    })
                    .await
            })
            .await
    }

    /// Earnings releases scheduled or reported over `[from, to]`.
    pub async fn earnings(&self, from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::Earnings, from, to).await
    }

    /// Initial public offerings over `[from, to]`.
    pub async fn ipos(&self, from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::Ipo, from, to).await
    }

    /// Dividend payments over `[from, to]`.
    pub async fn dividends(&self, from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::Dividend, from, to).await
    }

    /// Stock splits over `[from, to]`.
    pub async fn splits(&self, from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::Split, from, to).await
    }

    /// Macro-economic releases over `[from, to]`.
    pub async fn economic(&self, from: &str, to: &str) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::Economic, from, to).await
    }

    /// Upcoming market holidays and early closes. Providers return their
    /// upcoming set, so no date range is taken.
    pub async fn holidays(&self) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::MarketHoliday, "", "").await
    }

    /// Live open/closed status per exchange. A snapshot rather than a dated
    /// event, so no date range is taken. Currently Alpha Vantage only.
    pub async fn market_status(&self) -> Result<Vec<MarketCalendarEntry>> {
        self.fetch(CalendarKind::MarketStatus, "", "").await
    }
}

/// Market-wide performance statistics backed by configured data providers.
///
/// Routes through [`crate::Capability::MARKET`]. Unlike [`crate::finance::sector`] and
/// [`crate::finance::market_summary`] — Yahoo-only convenience shortcuts — these
/// honour the configured provider priority.
///
/// Created via [`Providers::market`](crate::Providers::market).
pub struct Market {
    providers: Arc<ProviderSet>,
}

impl Market {
    pub(crate) fn with_providers(providers: Arc<ProviderSet>) -> Self {
        Self { providers }
    }

    /// Aggregate performance for every sector.
    pub async fn sector_performance(&self) -> Result<Vec<SectorPerformance>> {
        dispatch_via!(
            self,
            MARKET,
            as_market,
            SectorPerformance,
            fetch_sector_performance,
            []
        )
    }

    /// Historical aggregate sector performance, most recent first.
    pub async fn sector_performance_history(
        &self,
        limit: u32,
    ) -> Result<Vec<crate::models::market::performance::SectorPerformanceHistory>> {
        dispatch_via!(
            self,
            MARKET,
            as_market,
            SectorPerformanceHistory,
            fetch_sector_performance_history,
            [],
            limit
        )
    }

    /// Price/earnings ratios by sector.
    pub async fn sector_pe(&self) -> Result<Vec<SectorPe>> {
        dispatch_via!(
            self,
            MARKET,
            as_market,
            SectorPerformance,
            fetch_sector_pe,
            []
        )
    }

    /// Price/earnings ratios by industry.
    pub async fn industry_pe(&self) -> Result<Vec<IndustryPe>> {
        dispatch_via!(
            self,
            MARKET,
            as_market,
            SectorPerformance,
            fetch_industry_pe,
            []
        )
    }

    /// Market movers for `direction`.
    pub async fn movers(&self, direction: MoverDirection) -> Result<Vec<MoverQuote>> {
        dispatch_via!(
            self,
            MARKET,
            as_market,
            MarketMovers,
            fetch_market_movers,
            [],
            direction
        )
    }

    /// Largest percentage gainers.
    pub async fn gainers(&self) -> Result<Vec<MoverQuote>> {
        self.movers(MoverDirection::Gainers).await
    }

    /// Largest percentage losers.
    pub async fn losers(&self) -> Result<Vec<MoverQuote>> {
        self.movers(MoverDirection::Losers).await
    }

    /// Highest traded volume.
    pub async fn most_active(&self) -> Result<Vec<MoverQuote>> {
        self.movers(MoverDirection::MostActive).await
    }

    /// Fetch grouped daily OHLCV bars for every stock ticker on `date`
    /// (`YYYY-MM-DD`) in one call — "give me every ticker's OHLC for this
    /// date" rather than one symbol at a time.
    ///
    /// Routes through [`Capability::CHART`](crate::providers::Capability::CHART)
    /// (the same capability backing per-symbol chart methods) rather than
    /// `MARKET`, since it's OHLCV data rather than a performance statistic.
    /// Currently Polygon only. Not cached — one date is one request either way.
    pub async fn grouped_daily(&self, date: &str) -> Result<Vec<(String, Candle)>> {
        let date = date.to_string();
        dispatch_via!(
            self,
            CHART,
            as_chart,
            GroupedDaily,
            fetch_grouped_daily,
            [date],
            &date
        )
    }

    /// Fetch grouped daily OHLCV bars for every crypto ticker on `date`
    /// (`YYYY-MM-DD`) in one call. See [`grouped_daily`](Self::grouped_daily).
    pub async fn crypto_grouped_daily(&self, date: &str) -> Result<Vec<(String, Candle)>> {
        let date = date.to_string();
        dispatch_via!(
            self,
            CHART,
            as_chart,
            CryptoGroupedDaily,
            fetch_crypto_grouped_daily,
            [date],
            &date
        )
    }

    /// Fetch grouped daily OHLCV bars for every forex ticker on `date`
    /// (`YYYY-MM-DD`) in one call. See [`grouped_daily`](Self::grouped_daily).
    pub async fn forex_grouped_daily(&self, date: &str) -> Result<Vec<(String, Candle)>> {
        let date = date.to_string();
        dispatch_via!(
            self,
            CHART,
            as_chart,
            ForexGroupedDaily,
            fetch_forex_grouped_daily,
            [date],
            &date
        )
    }

    /// Fetch coins/nfts/categories trending in the last 24h.
    ///
    /// Routes through [`Capability::CRYPTO`](crate::providers::Capability::CRYPTO).
    /// Currently CoinGecko only.
    #[cfg(feature = "crypto")]
    pub async fn crypto_trending(&self) -> Result<Vec<crate::models::crypto::TrendingCoin>> {
        dispatch_via!(
            self,
            CRYPTO,
            as_crypto,
            CryptoTrending,
            fetch_crypto_trending,
            []
        )
    }

    /// Fetch aggregate global cryptocurrency market statistics.
    ///
    /// Routes through [`Capability::CRYPTO`](crate::providers::Capability::CRYPTO).
    /// Currently CoinGecko only.
    #[cfg(feature = "crypto")]
    pub async fn crypto_global(&self) -> Result<crate::models::crypto::GlobalCryptoStats> {
        dispatch_via!(
            self,
            CRYPTO,
            as_crypto,
            CryptoGlobal,
            fetch_crypto_global,
            []
        )
    }
}
