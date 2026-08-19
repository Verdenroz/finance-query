#[cfg(feature = "indicators")]
use super::BatchIndicatorsResponse;
use super::Tickers;
#[cfg(feature = "backtesting")]
use crate::backtesting;
use crate::constants::{Interval, TimeRange};
#[cfg(feature = "indicators")]
use crate::error::Result;
#[cfg(any(feature = "backtesting", feature = "indicators"))]
use crate::indicators;
#[cfg(feature = "indicators")]
use std::sync::Arc;

impl Tickers {
    /// Batch calculate all technical indicators for all symbols
    ///
    /// Calculates complete indicator summaries for all symbols from their chart data.
    /// Indicators are cached per (symbol, interval, range) tuple.
    ///
    /// # Arguments
    ///
    /// * `interval` - The time interval for each candle
    /// * `range` - The time range to fetch data for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{Tickers, Interval, TimeRange};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT"]).await?;
    /// let indicators = tickers.indicators(Interval::OneDay, TimeRange::ThreeMonths).await?;
    ///
    /// for (symbol, ind) in &indicators.indicators {
    ///     println!("{}: RSI(14) = {:?}, SMA(20) = {:?}", symbol, ind.rsi_14, ind.sma_20);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "indicators")]
    pub async fn indicators(
        &self,
        interval: Interval,
        range: TimeRange,
    ) -> Result<BatchIndicatorsResponse> {
        let cache_key_for = |symbol: &Arc<str>| (symbol.clone(), interval, range);

        // Fast path: check if all symbols are cached
        {
            let cache = self.indicators_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().map(&cache_key_for)) {
                let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&cache_key_for(symbol)) {
                        response
                            .indicators
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Slow path: acquire fetch guard to prevent duplicate concurrent calculations
        let fetch_guard = Self::get_fetch_guard(&self.indicators_fetch, (interval, range)).await;
        let _guard = fetch_guard.lock().await;

        // Double-check: another task may have computed while we waited
        {
            let cache = self.indicators_cache.read().await;
            if self.all_cached(&cache, self.symbols.iter().map(&cache_key_for)) {
                let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());
                for symbol in &self.symbols {
                    if let Some(entry) = cache.get(&cache_key_for(symbol)) {
                        response
                            .indicators
                            .insert(symbol.to_string(), entry.value.clone());
                    }
                }
                return Ok(response);
            }
        }

        // Fetch charts first (which may already be cached, has its own deduplication)
        let charts_response = self.charts(interval, range).await?;

        let mut response = BatchIndicatorsResponse::with_capacity(self.symbols.len());

        // Calculate all indicators first (no lock held)
        let mut calculated_indicators: Vec<(String, indicators::IndicatorsSummary)> = Vec::new();

        for (symbol, chart) in &charts_response.charts {
            let indicators = indicators::summary::calculate_indicators(&chart.candles);
            calculated_indicators.push((symbol.to_string(), indicators));
        }

        // Now acquire write lock briefly for batch cache insertion
        if self.cache_mode.enabled() {
            let mut cache = self.indicators_cache.write().await;
            for (symbol, indicators) in &calculated_indicators {
                let key: Arc<str> = symbol.as_str().into();
                self.cache_insert(&mut cache, cache_key_for(&key), indicators.clone());
            }
        }

        // Populate response (no lock needed)
        for (symbol, indicators) in calculated_indicators {
            response.indicators.insert(symbol, indicators);
        }

        // Add errors from chart fetch
        for (symbol, error) in &charts_response.errors {
            response.errors.insert(symbol.to_string(), error.clone());
        }

        Ok(response)
    }

    // ========================================================================
    // Portfolio Backtesting
    // ========================================================================

    /// Run a multi-symbol portfolio backtest across all tracked symbols.
    ///
    /// Fetches charts and dividends for each symbol concurrently, then runs
    /// the portfolio engine with the given strategy factory. Capital is shared
    /// across all symbols according to the [`PortfolioConfig`] allocation rules.
    ///
    /// `factory` is called once per symbol to produce an independent strategy
    /// instance:
    ///
    /// ```no_run
    /// use finance_query::{Tickers, Interval, TimeRange};
    /// use finance_query::backtesting::{SmaCrossover, BacktestConfig};
    /// use finance_query::backtesting::portfolio::{PortfolioConfig, RebalanceMode};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tickers = Tickers::new(["AAPL", "MSFT", "NVDA"]).await?;
    ///
    /// let config = PortfolioConfig::new(BacktestConfig::default())
    ///     .max_total_positions(2)
    ///     .rebalance(RebalanceMode::EqualWeight);
    ///
    /// let result = tickers.backtest(
    ///     Interval::OneDay,
    ///     TimeRange::TwoYears,
    ///     Some(config),
    ///     |_sym| SmaCrossover::new(10, 50),
    /// ).await?;
    ///
    /// println!("Portfolio return: {:.2}%", result.portfolio_metrics.total_return_pct);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`PortfolioConfig`]: backtesting::portfolio::PortfolioConfig
    #[cfg(feature = "backtesting")]
    pub async fn backtest<S, F>(
        &self,
        interval: Interval,
        range: TimeRange,
        config: Option<backtesting::portfolio::PortfolioConfig>,
        factory: F,
    ) -> backtesting::Result<backtesting::portfolio::PortfolioResult>
    where
        S: backtesting::Strategy,
        F: Fn(&str) -> S,
    {
        use crate::backtesting::portfolio::{PortfolioEngine, SymbolData};

        let config = config.unwrap_or_default();
        config.validate(self.symbols.len())?;

        // Charts and dividends hit disjoint caches and disjoint capabilities
        // (CHART vs CORPORATE), so neither warms the other.
        let (charts, dividends_map) =
            tokio::join!(self.charts(interval, range), self.dividends(range));
        let charts = charts.map_err(|e| backtesting::BacktestError::ChartError(e.to_string()))?;
        // Treat errors as "no dividends" — dividend processing is best-effort
        let dividends_map = dividends_map.map(|b| b.dividends).unwrap_or_default();

        // Assemble SymbolData slices — skip symbols with no chart data
        let symbol_data: Vec<SymbolData> = self
            .symbols
            .iter()
            .filter_map(|sym| {
                charts.charts.get(sym.as_ref()).map(|chart| {
                    let divs = dividends_map.get(sym.as_ref()).cloned().unwrap_or_default();
                    SymbolData::new(sym.as_ref(), chart.candles.clone()).with_dividends(divs)
                })
            })
            .collect();

        let engine = PortfolioEngine::new(config);
        engine.run(&symbol_data, factory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires network access"]
    #[cfg(feature = "indicators")]
    async fn test_tickers_indicators() {
        let tickers = Tickers::new(["AAPL", "MSFT"]).await.unwrap();
        let result = tickers
            .indicators(Interval::OneDay, TimeRange::ThreeMonths)
            .await
            .unwrap();

        assert!(result.success_count() > 0);

        // Verify indicators structure
        for ind in result.indicators.values() {
            // Check that at least some indicators are present
            assert!(ind.rsi_14.is_some() || ind.sma_20.is_some());
        }
    }
}
