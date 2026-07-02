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
        let symbol = chart_symbol(self.id(), vs_currency);
        fetch_chart_via!(self, symbol, interval, range)
    }

    /// Fetch historical candles over `range` at a sensible default interval
    /// ([`TimeRange::default_interval`]).
    pub async fn history(&self, vs_currency: &str, range: TimeRange) -> Result<Chart> {
        self.chart(vs_currency, range.default_interval(), range)
            .await
    }

    /// Compute all technical indicators from this coin's chart data (priced in
    /// `vs_currency`).
    #[cfg(feature = "indicators")]
    pub async fn indicators(
        &self,
        vs_currency: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorsSummary> {
        let chart = self.chart(vs_currency, interval, range).await?;
        Ok(crate::indicators::summary::calculate_indicators(
            &chart.candles,
        ))
    }

    /// Compute a single technical indicator from this coin's chart data.
    #[cfg(feature = "indicators")]
    pub async fn indicator(
        &self,
        indicator: crate::indicators::Indicator,
        vs_currency: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::indicators::IndicatorResult> {
        let chart = self.chart(vs_currency, interval, range).await?;
        Ok(crate::indicators::compute_indicator(indicator, &chart)?)
    }

    /// Compute a risk summary from this coin's chart data, annualised with the
    /// 24/7 crypto calendar (365 days/year). `beta` is always `None`.
    #[cfg(feature = "risk")]
    pub async fn risk(
        &self,
        vs_currency: &str,
        interval: Interval,
        range: TimeRange,
    ) -> Result<crate::risk::RiskSummary> {
        let chart = self.chart(vs_currency, interval, range).await?;
        Ok(crate::risk::compute_risk_summary_with_periods(
            &chart.candles,
            None,
            crate::risk::periods_per_year(interval, crate::risk::TradingCalendar::Crypto),
        ))
    }
}

/// Build the `CHART` route symbol `"{ID}-{VS}"`, uppercased (e.g. `"BTC-USD"`)
/// — the Yahoo crypto convention, valid when the handle id is the coin ticker.
fn chart_symbol(id: &str, vs_currency: &str) -> String {
    format!("{}-{}", id.to_uppercase(), vs_currency.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_symbol_uppercases_ticker_and_vs() {
        assert_eq!(chart_symbol("BTC", "USD"), "BTC-USD");
        assert_eq!(chart_symbol("eth", "eur"), "ETH-EUR");
    }
}
