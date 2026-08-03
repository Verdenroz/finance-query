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
            as_indices,
            IndicesQuote,
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

    /// The [`MajorIndex`](crate::models::indices::MajorIndex) this handle's
    /// symbol denotes, or an error for indices without constituent support.
    fn major_index(&self) -> Result<crate::models::indices::MajorIndex> {
        crate::models::indices::MajorIndex::from_symbol(self.symbol()).ok_or_else(|| {
            crate::error::FinanceError::InvalidParameter {
                param: "symbol".into(),
                reason: format!(
                    "constituents are available for major indices only \
                     (S&P 500, Nasdaq 100, Dow Jones), not {}",
                    self.symbol()
                ),
            }
        })
    }

    /// Fetch the index's current constituents (major indices only). Not
    /// cached — constituent lists change rarely but the call is uncommon.
    pub async fn constituents(&self) -> Result<Vec<crate::models::indices::IndexConstituent>> {
        let index = self.major_index()?;
        self.providers
            .fetch(crate::providers::Capability::INDICES, move |p| {
                let p = p.clone();
                async move {
                    p.as_indices()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::IndexConstituents)
                        })?
                        .fetch_index_constituents(index)
                        .await
                }
            })
            .await
    }

    /// Fetch historical changes to the index's constituency (currently
    /// S&P 500 only on FMP).
    pub async fn constituent_changes(
        &self,
    ) -> Result<Vec<crate::models::indices::IndexConstituentChange>> {
        let index = self.major_index()?;
        self.providers
            .fetch(crate::providers::Capability::INDICES, move |p| {
                let p = p.clone();
                async move {
                    p.as_indices()
                        .ok_or_else(|| {
                            p.not_supported(crate::providers::Operation::IndexConstituentChanges)
                        })?
                        .fetch_index_constituent_changes(index)
                        .await
                }
            })
            .await
    }
}

impl_chartable_analytics!(Index, crate::risk::TradingCalendar::Exchange);
