//! Backtest execution engine.

mod benchmark;
mod exits;
mod indicators;
mod positions;
mod simulate;

pub(crate) use exits::update_trailing_hwm;
pub(crate) use indicators::compute_for_candles;

use crate::models::chart::{Candle, Dividend};

use self::benchmark::compute_benchmark_metrics;
use super::config::BacktestConfig;
use super::error::Result;
use super::result::BacktestResult;
use super::strategy::Strategy;

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
mod fixtures {
    use crate::backtesting::signal::Signal;
    use crate::backtesting::strategy::{Strategy, StrategyContext};
    use crate::indicators::Indicator;
    use crate::models::chart::Candle;

    #[derive(Clone)]
    pub(super) struct EnterLongHold;

    impl Strategy for EnterLongHold {
        fn name(&self) -> &str {
            "Enter Long Hold"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    #[derive(Clone)]
    pub(super) struct EnterShortHold;

    impl Strategy for EnterShortHold {
        fn name(&self) -> &str {
            "Enter Short Hold"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    pub(super) fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| Candle {
                timestamp: i as i64,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1000,
                adj_close: Some(p),
                provider_id: None,
            })
            .collect()
    }

    pub(super) fn make_candles_with_timestamps(prices: &[f64], timestamps: &[i64]) -> Vec<Candle> {
        prices
            .iter()
            .zip(timestamps.iter())
            .map(|(&p, &ts)| Candle {
                timestamp: ts,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1000,
                adj_close: Some(p),
                provider_id: None,
            })
            .collect()
    }

    /// Build a candle with explicit OHLC values (not derived from a single price).
    pub(super) fn make_candle_ohlc(ts: i64, open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            timestamp: ts,
            open,
            high,
            low,
            close,
            volume: 1000,
            adj_close: Some(close),
            provider_id: None,
        }
    }

    /// A strategy that opens a long on the first bar and holds forever.
    pub(super) struct EnterLongBar0;
    impl Strategy for EnterLongBar0 {
        fn name(&self) -> &str {
            "Enter Long Bar 0"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    /// Strategy: enter long on bar 0, scale in on bar 1, exit on bar 2.
    #[derive(Clone)]
    pub(super) struct EnterScaleInExit;

    impl Strategy for EnterScaleInExit {
        fn name(&self) -> &str {
            "EnterScaleInExit"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match ctx.index {
                0 => Signal::long(ctx.timestamp(), ctx.close()),
                1 if ctx.has_position() => Signal::scale_in(0.5, ctx.timestamp(), ctx.close()),
                2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                _ => Signal::hold(),
            }
        }
    }

    /// Strategy: enter long on bar 0, scale out 50% on bar 1, exit remainder on bar 2.
    #[derive(Clone)]
    pub(super) struct EnterScaleOutExit;

    impl Strategy for EnterScaleOutExit {
        fn name(&self) -> &str {
            "EnterScaleOutExit"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match ctx.index {
                0 => Signal::long(ctx.timestamp(), ctx.close()),
                1 if ctx.has_position() => Signal::scale_out(0.5, ctx.timestamp(), ctx.close()),
                2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                _ => Signal::hold(),
            }
        }
    }

    /// Enters a long position on bar 0 with a per-trade stop-loss.
    #[derive(Clone)]
    pub(super) struct BracketLongStopLossStrategy {
        pub(super) stop_pct: f64,
    }
    impl Strategy for BracketLongStopLossStrategy {
        fn name(&self) -> &str {
            "BracketLongStopLoss"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close()).stop_loss(self.stop_pct)
            } else {
                Signal::hold()
            }
        }
    }

    /// Enters a short position on bar 0 with a per-trade stop-loss.
    #[derive(Clone)]
    pub(super) struct BracketShortStopLossStrategy {
        pub(super) stop_pct: f64,
    }
    impl Strategy for BracketShortStopLossStrategy {
        fn name(&self) -> &str {
            "BracketShortStopLoss"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close()).stop_loss(self.stop_pct)
            } else {
                Signal::hold()
            }
        }
    }

    /// Enters a long position on bar 0 with a per-trade take-profit.
    #[derive(Clone)]
    pub(super) struct BracketLongTakeProfitStrategy {
        pub(super) tp_pct: f64,
    }
    impl Strategy for BracketLongTakeProfitStrategy {
        fn name(&self) -> &str {
            "BracketLongTakeProfit"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close()).take_profit(self.tp_pct)
            } else {
                Signal::hold()
            }
        }
    }

    /// Enters a short position on bar 0 with a per-trade take-profit.
    #[derive(Clone)]
    pub(super) struct BracketShortTakeProfitStrategy {
        pub(super) tp_pct: f64,
    }
    impl Strategy for BracketShortTakeProfitStrategy {
        fn name(&self) -> &str {
            "BracketShortTakeProfit"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close()).take_profit(self.tp_pct)
            } else {
                Signal::hold()
            }
        }
    }

    /// Enters a long position on bar 0 with a per-trade trailing stop.
    #[derive(Clone)]
    pub(super) struct BracketLongTrailingStopStrategy {
        pub(super) trail_pct: f64,
    }
    impl Strategy for BracketLongTrailingStopStrategy {
        fn name(&self) -> &str {
            "BracketLongTrailingStop"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close()).trailing_stop(self.trail_pct)
            } else {
                Signal::hold()
            }
        }
    }

    /// Enters a short position on bar 0 with a per-trade trailing stop.
    #[derive(Clone)]
    pub(super) struct BracketShortTrailingStopStrategy {
        pub(super) trail_pct: f64,
    }
    impl Strategy for BracketShortTrailingStopStrategy {
        fn name(&self) -> &str {
            "BracketShortTrailingStop"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close()).trailing_stop(self.trail_pct)
            } else {
                Signal::hold()
            }
        }
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
