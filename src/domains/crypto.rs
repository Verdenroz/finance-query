//! Cryptocurrency coin query handle.
//!
//! Created via [`Providers::crypto`](crate::Providers::crypto).

use crate::constants::{Interval, TimeRange};
use crate::error::Result;
use crate::models::chart::Chart;

domain_handle! {
    /// A cryptocurrency coin backed by configured data providers.
    ///
    /// Created via [`Providers::crypto`](crate::Providers::crypto).
    pub struct CryptoCoin { id, id }
    cache: crate::models::crypto::CryptoQuote, chart
}

impl CryptoCoin {
    /// Fetch the current quote for this coin priced in `vs_currency` (e.g., `"usd"`).
    pub async fn quote(&self, vs_currency: &str) -> Result<crate::models::crypto::CryptoQuote> {
        fetch_via_with!(
            self,
            id,
            CRYPTO,
            fetch_crypto_quote,
            vs_currency,
            crate::models::crypto::CryptoQuote
        )
    }

    /// Fetch historical OHLCV candles for this coin priced in `vs_currency`.
    ///
    /// Unlike [`quote`](Self::quote) (which uses the coin *id*, e.g.
    /// `"bitcoin"`), the `CHART` route is symbol-based, so the chart symbol is
    /// built as `"{ID}-{VS}"` uppercased (e.g. `"BTC-USD"`). This resolves on
    /// the default Yahoo route only when the handle's id is the coin's *ticker*
    /// (`providers.crypto("BTC")`); coins identified by a CoinGecko id should
    /// route `Capability::CHART` to a crypto-aware provider.
    pub async fn chart(
        &self,
        vs_currency: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<Chart> {
        let symbol = format!(
            "{}-{}",
            self.id().to_uppercase(),
            vs_currency.to_uppercase()
        );
        fetch_chart_via!(self, symbol, interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, vs_currency: &str, range: TimeRange) -> Result<Chart> {
        self.chart(vs_currency, range.default_interval(), range)
            .await
    }
}
