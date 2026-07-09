//! Forex currency-pair query handle.
//!
//! Created via [`Providers::forex`](crate::Providers::forex).

use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;

domain_handle! {
    /// A foreign-exchange currency pair backed by configured data providers.
    ///
    /// Created via [`Providers::forex`](crate::Providers::forex).
    pub struct ForexPair {
        /// The base (from) currency code (e.g., `"USD"`).
        from, from,
        /// The quote (to) currency code (e.g., `"EUR"`).
        to, to,
    }
    cache: crate::models::forex::ForexQuote, chart
}

impl ForexPair {
    /// Fetch the current exchange rate for this currency pair.
    pub async fn quote(&self) -> Result<crate::models::forex::ForexQuote> {
        fetch_via_two!(
            self,
            from,
            to,
            FOREX,
            fetch_forex_quote,
            crate::models::forex::ForexQuote
        )
    }

    /// Fetch historical OHLCV candles for this currency pair.
    ///
    /// The pair is mapped to the `CHART` route's symbol form `"{FROM}{TO}=X"`
    /// (e.g. `"USDEUR=X"`, the Yahoo FX convention).
    pub async fn chart(&self, interval: Interval, range: TimeRange) -> Result<Chart> {
        fetch_chart_via!(self, chart_symbol(&self.from, &self.to), interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, range: TimeRange) -> Result<Chart> {
        self.chart(range.default_interval(), range).await
    }
}

impl_chartable_analytics!(ForexPair, crate::risk::TradingCalendar::Forex);

/// Map a currency pair to the `CHART` route's symbol form `"{FROM}{TO}=X"`
/// (the Yahoo FX convention, e.g. `"USDEUR=X"`).
fn chart_symbol(from: &str, to: &str) -> String {
    format!("{from}{to}=X")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_symbol_uses_yahoo_fx_convention() {
        assert_eq!(chart_symbol("USD", "EUR"), "USDEUR=X");
        assert_eq!(chart_symbol("GBP", "JPY"), "GBPJPY=X");
    }
}
