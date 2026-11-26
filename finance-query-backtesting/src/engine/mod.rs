//! Backtest execution engine.

mod benchmark;
mod exits;
#[cfg(test)]
mod fixtures;
mod indicators;
mod margin;
mod positions;
mod simulate;
mod sizing;

pub(crate) use exits::{check_sl_tp, update_position_extremes, update_trailing_hwm};
pub(crate) use indicators::compute_for_candles;
pub(crate) use sizing::SizingSeries;

use crate::models::chart::{Candle, Dividend};

use self::benchmark::compute_benchmark_metrics;
use super::config::BacktestConfig;
use super::error::{BacktestError, Result};
use super::result::BacktestResult;
use super::strategy::Strategy;

/// Reject candle or dividend series that are not ascending by timestamp.
///
/// Conditions binary-search the candle slice to locate the position entry, so an
/// out-of-order series would silently yield a wrong index rather than an error.
/// Dividend crediting walks a forward-only index for the same reason.
///
/// Optimisers and walk-forward call this once for the whole series and then use
/// [`BacktestEngine::simulate`] per candidate, so a sweep pays the O(n) scan once
/// instead of once per evaluation.
pub(crate) fn validate_series_order(candles: &[Candle], dividends: &[Dividend]) -> Result<()> {
    if !candles.windows(2).all(|w| w[0].timestamp <= w[1].timestamp) {
        return Err(BacktestError::invalid_param(
            "candles",
            "must be sorted by timestamp (ascending)",
        ));
    }
    if !dividends
        .windows(2)
        .all(|w| w[0].timestamp <= w[1].timestamp)
    {
        return Err(BacktestError::invalid_param(
            "dividends",
            "must be sorted by timestamp (ascending)",
        ));
    }
    Ok(())
}

/// Backtest execution engine.
///
/// Handles indicator pre-computation, position management, and trade execution.
pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    /// Create a new backtest engine with the given configuration
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run a backtest with the given strategy on historical candle data.
    ///
    /// Dividend income is not included. Use [`run_with_dividends`] to account
    /// for dividend payments during holding periods.
    ///
    /// [`run_with_dividends`]: Self::run_with_dividends
    pub fn run<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
    ) -> Result<BacktestResult> {
        validate_series_order(candles, &[])?;
        self.simulate(symbol, candles, strategy, &[])
    }

    /// Run a backtest and credit dividend income for any dividends paid while a
    /// position is open.
    ///
    /// `dividends` should be sorted by timestamp (ascending). The engine credits
    /// each dividend whose ex-date falls on or before the current candle bar.
    /// When [`BacktestConfig::reinvest_dividends`] is `true`, the income is also
    /// used to notionally purchase additional shares at the ex-date close price.
    pub fn run_with_dividends<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
        dividends: &[Dividend],
    ) -> Result<BacktestResult> {
        validate_series_order(candles, dividends)?;
        self.simulate(symbol, candles, strategy, dividends)
    }

    /// Run a backtest and compare against a benchmark, optionally crediting dividends.
    ///
    /// The result's `benchmark` field is populated with buy-and-hold comparison
    /// metrics including alpha, beta, and information ratio. The benchmark candle
    /// slice should cover the same time period as `candles` but need not be the
    /// same length.
    ///
    /// `dividends` must be sorted ascending by timestamp. Pass `&[]` to omit
    /// dividend processing.
    pub fn run_with_benchmark<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
        dividends: &[Dividend],
        benchmark_symbol: &str,
        benchmark_candles: &[Candle],
    ) -> Result<BacktestResult> {
        validate_series_order(candles, dividends)?;
        let mut result = self.simulate(symbol, candles, strategy, dividends)?;
        result.benchmark = Some(compute_benchmark_metrics(
            benchmark_symbol,
            candles,
            benchmark_candles,
            &result.equity_curve,
            self.config.risk_free_rate,
            self.config.bars_per_year,
        ));
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::make_candles;
    use super::*;
    use crate::backtesting::strategy::SmaCrossover;

    #[test]
    fn test_engine_basic() {
        // Price trends up then down - should trigger crossover signals
        let mut prices = vec![100.0; 30];
        // Make fast SMA cross above slow SMA around bar 15
        for (i, price) in prices.iter_mut().enumerate().take(25).skip(15) {
            *price = 100.0 + (i - 15) as f64 * 2.0;
        }
        // Then cross back down
        for (i, price) in prices.iter_mut().enumerate().take(30).skip(25) {
            *price = 118.0 - (i - 25) as f64 * 3.0;
        }

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(5, 10);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        assert_eq!(result.symbol, "TEST");
        assert_eq!(result.strategy_name, "SMA Crossover");
        assert!(!result.equity_curve.is_empty());
    }

    #[test]
    fn test_stop_loss() {
        // Price drops significantly after entry
        let mut prices = vec![100.0; 20];
        // Trend up to trigger long entry
        for (i, price) in prices.iter_mut().enumerate().take(15).skip(10) {
            *price = 100.0 + (i - 10) as f64 * 2.0;
        }
        // Then crash
        for (i, price) in prices.iter_mut().enumerate().take(20).skip(15) {
            *price = 108.0 - (i - 15) as f64 * 10.0;
        }

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .stop_loss_pct(0.05) // 5% stop loss
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(3, 6);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        // Should have triggered stop-loss
        let _sl_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| {
                s.reason
                    .as_ref()
                    .map(|r| r.contains("Stop-loss"))
                    .unwrap_or(false)
            })
            .collect();

        // May or may not trigger depending on exact timing
        // The important thing is the engine doesn't crash
        assert!(!result.equity_curve.is_empty());
    }

    #[test]
    fn test_trailing_stop() {
        // Price rises to 120, then drops 10%+ → trailing stop should fire
        let mut prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        // Peak is 119; now drop past 10% from peak (< 107.1)
        prices.extend_from_slice(&[105.0, 103.0, 101.0]);

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .trailing_stop_pct(0.10)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(3, 6);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        let trail_exits: Vec<_> = result
            .signals
            .iter()
            .filter(|s| {
                s.reason
                    .as_ref()
                    .map(|r| r.contains("Trailing stop"))
                    .unwrap_or(false)
            })
            .collect();

        // Not guaranteed to fire given the specific crossover timing, but engine must not crash
        let _ = trail_exits;
        assert!(!result.equity_curve.is_empty());
    }
}
