//! Alpha Vantage adapter integration for [`Ticker`].
//!
//! Exposes a curated subset of the [`crate::adapters::alphavantage`]
//! free functions as methods on a typed handle obtained via
//! [`crate::Ticker::alphavantage`].
//!
//! Requires the `alphavantage` feature flag and a one-time call to
//! [`crate::adapters::alphavantage::init`] before any method is invoked.

use std::sync::Arc;

use crate::adapters::alphavantage as av;
use crate::error::Result;

/// Typed accessor for the Alpha Vantage adapter, scoped to a single ticker.
#[derive(Clone, Debug)]
pub struct AlphaVantageHandle {
    symbol: Arc<str>,
}

impl AlphaVantageHandle {
    pub(crate) fn new(symbol: Arc<str>) -> Self {
        Self { symbol }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Real-time global quote (price, volume, change, latest trading day).
    ///
    /// Wraps [`av::global_quote`].
    pub async fn quote(&self) -> Result<av::GlobalQuote> {
        av::global_quote(&self.symbol).await
    }

    /// Intraday time series at the given interval.
    ///
    /// Wraps [`av::time_series_intraday`]. The optional `output_size`
    /// parameter is not exposed here — call [`av::time_series_intraday`]
    /// directly if you need it.
    pub async fn intraday(&self, interval: av::AvInterval) -> Result<av::TimeSeries> {
        av::time_series_intraday(&self.symbol, interval, None).await
    }

    /// Daily OHLCV time series.
    ///
    /// Wraps [`av::time_series_daily`].
    pub async fn daily(&self) -> Result<av::TimeSeries> {
        av::time_series_daily(&self.symbol, None).await
    }

    /// Daily OHLCV time series, adjusted for splits and dividends.
    ///
    /// Wraps [`av::time_series_daily_adjusted`].
    pub async fn daily_adjusted(&self) -> Result<av::AdjustedTimeSeries> {
        av::time_series_daily_adjusted(&self.symbol, None).await
    }

    /// Weekly OHLCV time series.
    ///
    /// Wraps [`av::time_series_weekly`].
    pub async fn weekly(&self) -> Result<av::TimeSeries> {
        av::time_series_weekly(&self.symbol).await
    }

    /// Weekly OHLCV time series, adjusted.
    ///
    /// Wraps [`av::time_series_weekly_adjusted`].
    pub async fn weekly_adjusted(&self) -> Result<av::AdjustedTimeSeries> {
        av::time_series_weekly_adjusted(&self.symbol).await
    }

    /// Monthly OHLCV time series.
    ///
    /// Wraps [`av::time_series_monthly`].
    pub async fn monthly(&self) -> Result<av::TimeSeries> {
        av::time_series_monthly(&self.symbol).await
    }

    /// Monthly OHLCV time series, adjusted.
    ///
    /// Wraps [`av::time_series_monthly_adjusted`].
    pub async fn monthly_adjusted(&self) -> Result<av::AdjustedTimeSeries> {
        av::time_series_monthly_adjusted(&self.symbol).await
    }

    /// Company fundamentals overview (description, sector, P/E, EPS, etc.).
    ///
    /// Wraps [`av::company_overview`].
    pub async fn overview(&self) -> Result<av::CompanyOverview> {
        av::company_overview(&self.symbol).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_holds_symbol() {
        let h = AlphaVantageHandle::new(Arc::from("AAPL"));
        assert_eq!(h.symbol(), "AAPL");
    }

    #[test]
    fn handle_clone_is_cheap_arc_clone() {
        let h1 = AlphaVantageHandle::new(Arc::from("AAPL"));
        let h2 = h1.clone();
        assert_eq!(h1.symbol(), h2.symbol());
    }

    // Compile-time existence test. Body never runs but must type-check, so
    // signature drift in any wrapped adapter function fails the build.
    #[allow(dead_code)]
    async fn _av_method_signatures_compile(h: &AlphaVantageHandle) {
        let _ = h.quote().await;
        let _ = h.intraday(av::AvInterval::OneMin).await;
        let _ = h.daily().await;
        let _ = h.daily_adjusted().await;
        let _ = h.weekly().await;
        let _ = h.weekly_adjusted().await;
        let _ = h.monthly().await;
        let _ = h.monthly_adjusted().await;
        let _ = h.overview().await;
    }
}
