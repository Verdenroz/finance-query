//! Commodity price quote handle.
//!
//! Created via [`Providers::commodity`](crate::Providers::commodity).

use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;

domain_handle! {
    /// A commodity backed by configured data providers.
    ///
    /// Created via [`Providers::commodity`](crate::Providers::commodity).
    pub struct Commodity { symbol, symbol }
    cache: crate::models::commodities::CommodityQuote, chart
}

impl Commodity {
    /// Fetch the current quote for this commodity.
    pub async fn quote(&self) -> Result<crate::models::commodities::CommodityQuote> {
        fetch_via!(
            self,
            symbol,
            COMMODITIES,
            fetch_commodities_quote,
            crate::models::commodities::CommodityQuote
        )
    }

    /// Fetch historical OHLCV candles for this commodity.
    ///
    /// The symbol is passed to the `CHART` route as-is. Note this expects the
    /// route's chart-symbol form (e.g. the Yahoo futures symbol `GC=F` for
    /// gold), which differs from the commodity *names* (`WHEAT`) some providers
    /// use for [`quote`](Self::quote).
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        fetch_chart_via!(self, self.symbol.to_string(), interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, range: TimeRange) -> Result<Chart> {
        self.chart(range.default_interval(), range).await
    }
}

impl_chartable_analytics!(Commodity, crate::risk::TradingCalendar::Exchange);
