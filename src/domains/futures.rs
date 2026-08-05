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
    cache: crate::models::futures::FuturesQuote, chart,
    extra: {
        #[cfg(feature = "cftc")]
        cot_cache: crate::models::futures::cot::CommitmentsOfTraders,
    }
}

impl FuturesContract {
    /// Fetch the current quote for this futures contract.
    pub async fn quote(&self) -> Result<crate::models::futures::FuturesQuote> {
        fetch_via!(
            self,
            symbol,
            FUTURES,
            as_futures,
            FuturesQuote,
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

    /// Fetch weekly CFTC Commitments of Traders positioning for this futures
    /// contract — long/short/spread broken down by trader category
    /// (commercial hedgers, swap dealers, managed money, other reportables,
    /// small traders).
    ///
    /// Routed through `Capability::FUTURES`; only [`Provider::Cftc`](crate::Provider::Cftc)
    /// serves it, so route `FUTURES` to include it. CFTC covers physical
    /// commodities only (agriculture, energy, metals) via the disaggregated
    /// futures-only report — the symbol is either a recognised Yahoo-style
    /// continuous futures root (`"GC=F"`, `"CL=F"`, …) or a raw CFTC
    /// `cftc_contract_market_code` passed straight through.
    #[cfg(feature = "cftc")]
    pub async fn commitments_of_traders(
        &self,
    ) -> Result<crate::models::futures::cot::CommitmentsOfTraders> {
        fetch_via!(
            cache: cot_cache,
            self,
            symbol,
            FUTURES,
            as_futures,
            CommitmentsOfTraders,
            fetch_commitments_of_traders,
            crate::models::futures::cot::CommitmentsOfTraders
        )
    }
}

impl_chartable_analytics!(FuturesContract, crate::risk::TradingCalendar::Exchange);
