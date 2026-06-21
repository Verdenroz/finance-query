//! Futures contract quote handle.
//!
//! Created via [`Providers::futures`](crate::Providers::futures).

use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;

domain_handle! {
    /// A futures contract backed by configured data providers.
    ///
    /// Created via [`Providers::futures`](crate::Providers::futures).
    pub struct FuturesContract { symbol, symbol }
    cache: crate::models::futures::FuturesQuote, chart
}

impl FuturesContract {
    /// Fetch the current quote for this futures contract.
    pub async fn quote(&self) -> Result<crate::models::futures::FuturesQuote> {
        fetch_via!(
            self,
            symbol,
            FUTURES,
            fetch_futures_quote,
            crate::models::futures::FuturesQuote
        )
    }

    /// Fetch historical OHLCV candles for this futures contract.
    ///
    /// The symbol is passed to the `CHART` route as-is (e.g. Yahoo futures
    /// symbols like `NQ=F`).
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        fetch_chart_via!(self, self.symbol.to_string(), interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, range: TimeRange) -> Result<Chart> {
        self.chart(range.default_interval(), range).await
    }
}
