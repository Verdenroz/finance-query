//! Polygon.io adapter integration for [`Ticker`].
//!
//! Exposes a curated subset of the [`crate::adapters::polygon`] free
//! functions as methods on a typed handle obtained via
//! [`crate::Ticker::polygon`].
//!
//! Requires the `polygon` feature flag and a one-time call to
//! [`crate::adapters::polygon::init`] before any method is invoked.
//!
//! All methods return adapter-native types unchanged (no translation, no
//! fallback). If the adapter has not been initialized, the underlying
//! free function returns
//! [`crate::error::FinanceError::InvalidParameter`].

use std::sync::Arc;

use crate::adapters::polygon;
use crate::error::Result;

/// Typed accessor for the Polygon.io adapter, scoped to a single ticker.
///
/// Construct via [`crate::Ticker::polygon`]. See the module docs for
/// usage and feature/init requirements.
#[derive(Clone, Debug)]
pub struct PolygonHandle {
    symbol: Arc<str>,
}

impl PolygonHandle {
    /// Construct a handle. Crate-internal — public callers go through
    /// [`crate::Ticker::polygon`].
    pub(crate) fn new(symbol: Arc<str>) -> Self {
        Self { symbol }
    }

    /// The ticker symbol this handle is scoped to.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Most-recent stock snapshot (price, volume, last trade, last quote).
    ///
    /// Wraps [`polygon::stock_snapshot`].
    pub async fn snapshot(&self) -> Result<polygon::SingleSnapshotResponse> {
        polygon::stock_snapshot(&self.symbol).await
    }

    /// Aggregate (OHLCV) bars between two dates.
    ///
    /// Wraps [`polygon::stock_aggregates`].
    pub async fn aggregates(
        &self,
        multiplier: u32,
        timespan: polygon::Timespan,
        from: &str,
        to: &str,
        params: Option<polygon::AggregateParams>,
    ) -> Result<polygon::AggregateResponse> {
        polygon::stock_aggregates(&self.symbol, multiplier, timespan, from, to, params).await
    }

    /// Previous trading day's open/close/high/low/volume bar.
    ///
    /// Wraps [`polygon::stock_previous_close`].
    pub async fn previous_close(
        &self,
        adjusted: Option<bool>,
    ) -> Result<polygon::AggregateResponse> {
        polygon::stock_previous_close(&self.symbol, adjusted).await
    }

    /// Most-recent tick-level trade.
    ///
    /// Wraps [`polygon::stock_last_trade`].
    pub async fn last_trade(&self) -> Result<polygon::LastTradeResponse> {
        polygon::stock_last_trade(&self.symbol).await
    }

    /// News articles for this ticker, most recent first. `limit` is capped
    /// by Polygon at 1000.
    ///
    /// Wraps [`polygon::stock_news`].
    pub async fn news(
        &self,
        limit: u32,
    ) -> Result<polygon::PaginatedResponse<polygon::NewsArticle>> {
        let limit_str = limit.to_string();
        polygon::stock_news(&[("ticker", &self.symbol), ("limit", &limit_str)]).await
    }

    /// Historical dividends for this ticker.
    ///
    /// Wraps [`polygon::stock_dividends`].
    pub async fn dividends(&self) -> Result<polygon::PaginatedResponse<polygon::Dividend>> {
        polygon::stock_dividends(&[("ticker", &self.symbol)]).await
    }

    /// Historical stock splits for this ticker.
    ///
    /// Wraps [`polygon::stock_splits`].
    pub async fn splits(&self) -> Result<polygon::PaginatedResponse<polygon::Split>> {
        polygon::stock_splits(&[("ticker", &self.symbol)]).await
    }

    /// Quarterly or annual financial statements.
    ///
    /// `period` is one of `"annual"`, `"quarterly"`, or `"ttm"`.
    ///
    /// Wraps [`polygon::stock_financials`].
    pub async fn financials(
        &self,
        period: &str,
        limit: Option<u32>,
    ) -> Result<polygon::PaginatedResponse<polygon::FinancialResult>> {
        let limit_str = limit.map(|n| n.to_string()).unwrap_or_default();
        let mut params: Vec<(&str, &str)> = vec![("period_of_report_type", period)];
        if !limit_str.is_empty() {
            params.push(("limit", &limit_str));
        }
        polygon::stock_financials(&self.symbol, &params).await
    }

    /// Ticker reference data (profile, shares outstanding, market cap).
    ///
    /// Wraps [`polygon::ticker_details`].
    pub async fn details(&self) -> Result<polygon::TickerDetailsResponse> {
        polygon::ticker_details(&self.symbol).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_holds_symbol() {
        let h = PolygonHandle::new(Arc::from("AAPL"));
        assert_eq!(h.symbol(), "AAPL");
    }

    #[test]
    fn handle_clone_is_cheap_arc_clone() {
        let h1 = PolygonHandle::new(Arc::from("AAPL"));
        let h2 = h1.clone();
        assert_eq!(h1.symbol(), h2.symbol());
    }

    // Compile-time existence tests: verify each method's signature compiles.
    // These do not run at runtime — if they compile, they pass. If a wrapped
    // adapter function changes shape and the wrapper isn't updated, this fails
    // to compile.
    #[allow(dead_code, clippy::manual_async_fn)]
    async fn _polygon_method_signatures_compile(h: &super::PolygonHandle) {
        use crate::adapters::polygon;
        let _ = h.snapshot().await;
        let _ = h
            .aggregates(1, polygon::Timespan::Day, "2024-01-01", "2024-01-31", None)
            .await;
        let _ = h.previous_close(None).await;
        let _ = h.last_trade().await;
        let _ = h.news(10).await;
        let _ = h.dividends().await;
        let _ = h.splits().await;
        let _ = h.financials("annual", None).await;
        let _ = h.details().await;
    }
}
