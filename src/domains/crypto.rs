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
    cache: crate::models::crypto::CryptoQuote, chart,
    extra: {
        #[cfg(feature = "defi")]
        tvl_cache: crate::models::crypto::defi::ProtocolTvl,
        #[cfg(feature = "defi")]
        tvl_history_cache: Vec<crate::models::crypto::defi::TvlPoint>,
    }
}

impl CryptoCoin {
    /// Fetch the current quote for this coin priced in `vs_currency` (e.g., `"usd"`).
    pub async fn quote(&self, vs_currency: &str) -> Result<crate::models::crypto::CryptoQuote> {
        fetch_via_with!(
            self,
            id,
            CRYPTO,
            as_crypto,
            CryptoQuote,
            fetch_crypto_quote,
            vs_currency,
            crate::models::crypto::CryptoQuote
        )
    }

    /// Fetch total value locked for this handle read as a **DeFi protocol
    /// slug** (e.g. `providers.crypto("aave")`).
    ///
    /// Routed through `Capability::CRYPTO`; only DefiLlama serves it, so route
    /// `CRYPTO` to include [`Provider::DefiLlama`](crate::Provider::DefiLlama).
    /// The id is a protocol slug here, not a coin id — most DefiLlama slugs
    /// happen to match their CoinGecko id, but not all do. The response is
    /// cached on the handle, so a repeat call costs nothing.
    #[cfg(feature = "defi")]
    pub async fn tvl(&self) -> Result<crate::models::crypto::defi::ProtocolTvl> {
        fetch_via!(
            cache: tvl_cache,
            self,
            id,
            CRYPTO,
            as_crypto,
            ProtocolTvl,
            fetch_protocol_tvl,
            crate::models::crypto::defi::ProtocolTvl
        )
    }

    /// Fetch this protocol's full TVL history, oldest first.
    ///
    /// Same routing and slug semantics as [`tvl`](Self::tvl).
    #[cfg(feature = "defi")]
    pub async fn tvl_history(&self) -> Result<Vec<crate::models::crypto::defi::TvlPoint>> {
        fetch_via!(
            cache: tvl_history_cache,
            self,
            id,
            CRYPTO,
            as_crypto,
            ProtocolTvlHistory,
            fetch_protocol_tvl_history,
            Vec<crate::models::crypto::defi::TvlPoint>
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
    ///
    /// With the `crypto` feature, `.route(Capability::CHART,
    /// [Provider::CoinGecko])` serves history keylessly by coin id. CoinGecko
    /// picks its own bar granularity from the requested range, so `interval` is
    /// advisory there, and its OHLC endpoint carries no volume — those candles
    /// report `volume: 0`.
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

    /// Fetch market-wide crypto news via `Capability::CRYPTO` (currently FMP
    /// only). Not scoped to this handle's coin id.
    pub async fn news(&self, limit: u32) -> Result<Vec<crate::models::corporate::news::News>> {
        dispatch_via!(
            self,
            CRYPTO,
            as_crypto,
            CryptoNews,
            fetch_crypto_news,
            [],
            limit
        )
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
