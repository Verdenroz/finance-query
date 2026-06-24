//! Stock market index quote handle.
//!
//! Created via [`Providers::index`](crate::Providers::index).

use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;

domain_handle! {
    /// A stock market index backed by configured data providers.
    ///
    /// Created via [`Providers::index`](crate::Providers::index).
    pub struct Index { symbol, symbol }
    cache: crate::models::indices::IndexQuote, chart
}

impl Index {
    /// Fetch the current quote for this index.
    pub async fn quote(&self) -> Result<crate::models::indices::IndexQuote> {
        fetch_via!(
            self,
            symbol,
            INDICES,
            fetch_indices_quote,
            crate::models::indices::IndexQuote
        )
    }

    /// Fetch historical OHLCV candles for this index.
    ///
    /// The symbol is passed to the `CHART` route as-is, so it should be in the
    /// form the route expects (e.g. Yahoo index symbols like `^GSPC`).
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        fetch_chart_via!(self, self.symbol.to_string(), interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, range: TimeRange) -> Result<Chart> {
        self.chart(range.default_interval(), range).await
    }
}

impl_chartable_analytics!(Index, crate::risk::TradingCalendar::Exchange);
