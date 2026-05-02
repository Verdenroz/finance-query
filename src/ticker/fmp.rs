//! Financial Modeling Prep (FMP) adapter integration for [`Ticker`].
//!
//! Exposes a curated subset of the [`crate::adapters::fmp`] free
//! functions as methods on a typed handle obtained via
//! [`crate::Ticker::fmp`].
//!
//! Requires the `fmp` feature flag and a one-time call to
//! [`crate::adapters::fmp::init`] before any method is invoked.
//!
//! All methods return adapter-native types unchanged (no translation,
//! no fallback). If the adapter has not been initialized, the
//! underlying free function returns
//! [`crate::error::FinanceError::InvalidParameter`].

use std::sync::Arc;

use crate::adapters::fmp;
use crate::error::Result;

/// Typed accessor for the FMP adapter, scoped to a single ticker.
#[derive(Clone, Debug)]
pub struct FmpHandle {
    symbol: Arc<str>,
}

impl FmpHandle {
    pub(crate) fn new(symbol: Arc<str>) -> Self {
        Self { symbol }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Real-time quote (price, change, volume, market cap, etc.).
    ///
    /// Wraps [`fmp::quote`].
    pub async fn quote(&self) -> Result<Vec<fmp::FmpQuote>> {
        fmp::quote(&self.symbol).await
    }

    /// Historical daily OHLCV between two optional dates (`YYYY-MM-DD`).
    ///
    /// Wraps [`fmp::historical_price_daily`].
    pub async fn historical(
        &self,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<fmp::HistoricalPriceResponse> {
        let params = fmp::HistoricalPriceParams {
            from: from.map(str::to_owned),
            to: to.map(str::to_owned),
        };
        fmp::historical_price_daily(&self.symbol, Some(params)).await
    }

    /// Intraday OHLCV bars at the given interval ("1min" | "5min" |
    /// "15min" | "30min" | "1hour" | "4hour").
    ///
    /// Wraps [`fmp::historical_price_intraday`].
    pub async fn intraday(
        &self,
        interval: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<fmp::IntradayPrice>> {
        let params = if from.is_some() || to.is_some() {
            Some(fmp::HistoricalPriceParams {
                from: from.map(str::to_owned),
                to: to.map(str::to_owned),
            })
        } else {
            None
        };
        fmp::historical_price_intraday(&self.symbol, interval, params).await
    }

    /// Income statement for the configured period.
    ///
    /// Wraps [`fmp::income_statement`].
    pub async fn income_statement(
        &self,
        period: fmp::Period,
        limit: Option<u32>,
    ) -> Result<Vec<fmp::IncomeStatement>> {
        fmp::income_statement(&self.symbol, period, limit).await
    }

    /// Balance sheet for the configured period.
    ///
    /// Wraps [`fmp::balance_sheet`].
    pub async fn balance_sheet(
        &self,
        period: fmp::Period,
        limit: Option<u32>,
    ) -> Result<Vec<fmp::BalanceSheet>> {
        fmp::balance_sheet(&self.symbol, period, limit).await
    }

    /// Cash flow statement for the configured period.
    ///
    /// Wraps [`fmp::cash_flow`].
    pub async fn cash_flow(
        &self,
        period: fmp::Period,
        limit: Option<u32>,
    ) -> Result<Vec<fmp::CashFlow>> {
        fmp::cash_flow(&self.symbol, period, limit).await
    }

    /// Key valuation, profitability, and efficiency metrics.
    ///
    /// Wraps [`fmp::key_metrics`].
    pub async fn key_metrics(
        &self,
        period: fmp::Period,
        limit: Option<u32>,
    ) -> Result<Vec<fmp::KeyMetrics>> {
        fmp::key_metrics(&self.symbol, period, limit).await
    }

    /// Financial ratios (PE, PB, debt/equity, etc.).
    ///
    /// Wraps [`fmp::financial_ratios`].
    pub async fn ratios(
        &self,
        period: fmp::Period,
        limit: Option<u32>,
    ) -> Result<Vec<fmp::FinancialRatios>> {
        fmp::financial_ratios(&self.symbol, period, limit).await
    }

    /// Company profile (description, sector, industry, executives).
    ///
    /// Wraps [`fmp::company_profile`].
    pub async fn profile(&self) -> Result<Vec<fmp::CompanyProfile>> {
        fmp::company_profile(&self.symbol).await
    }

    /// News articles for this ticker.
    ///
    /// Wraps [`fmp::stock_news`].
    pub async fn news(&self, limit: u32) -> Result<Vec<fmp::StockNews>> {
        fmp::stock_news(&self.symbol, limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time existence test: each method is awaited on a real handle.
    // Body never runs (function is unused), but it MUST type-check, so a
    // signature drift in any wrapped adapter function fails the build.
    #[allow(dead_code)]
    async fn _fmp_method_signatures_compile(h: &FmpHandle) {
        let _ = h.quote().await;
        let _ = h.historical(None, None).await;
        let _ = h.intraday("1min", None, None).await;
        let _ = h.income_statement(fmp::Period::Quarter, Some(4)).await;
        let _ = h.balance_sheet(fmp::Period::Quarter, Some(4)).await;
        let _ = h.cash_flow(fmp::Period::Quarter, Some(4)).await;
        let _ = h.key_metrics(fmp::Period::Quarter, Some(4)).await;
        let _ = h.ratios(fmp::Period::Quarter, Some(4)).await;
        let _ = h.profile().await;
        let _ = h.news(10).await;
    }

    #[test]
    fn handle_holds_symbol() {
        let h = FmpHandle::new(Arc::from("AAPL"));
        assert_eq!(h.symbol(), "AAPL");
    }

    #[test]
    fn handle_clone_is_cheap_arc_clone() {
        let h1 = FmpHandle::new(Arc::from("AAPL"));
        let h2 = h1.clone();
        assert_eq!(h1.symbol(), h2.symbol());
    }
}
