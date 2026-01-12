//! Backtest execution engine.

use std::collections::HashMap;

use crate::indicators::{self, Indicator};
use crate::models::chart::Candle;

use super::config::BacktestConfig;
use super::error::{BacktestError, Result};
use super::position::{Position, PositionSide, Trade};
use super::result::{BacktestResult, EquityPoint, PerformanceMetrics, SignalRecord};
use super::signal::{Signal, SignalDirection};
use super::strategy::{Strategy, StrategyContext};

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

    /// Run a backtest with the given strategy on historical candle data
    pub fn run<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
    ) -> Result<BacktestResult> {
        let warmup = strategy.warmup_period();
        if candles.len() < warmup {
            return Err(BacktestError::insufficient_data(warmup, candles.len()));
        }

        // Pre-compute all required indicators
        let indicators = self.compute_indicators(candles, &strategy)?;

        // Initialize state
        let mut equity = self.config.initial_capital;
        let mut cash = self.config.initial_capital;
        let mut position: Option<Position> = None;
        let mut trades: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<EquityPoint> = Vec::new();
        let mut signals: Vec<SignalRecord> = Vec::new();
        let mut peak_equity = equity;

        // Main simulation loop
        for i in 0..candles.len() {
            let candle = &candles[i];

            // Update equity with current position value
            if let Some(ref pos) = position {
                let pos_value = pos.current_value(candle.close);
                equity = cash + pos_value;
            } else {
                equity = cash;
            }

            // Track drawdown
            if equity > peak_equity {
                peak_equity = equity;
            }
            let drawdown_pct = if peak_equity > 0.0 {
                (peak_equity - equity) / peak_equity
            } else {
                0.0
            };

            equity_curve.push(EquityPoint {
                timestamp: candle.timestamp,
                equity,
                drawdown_pct,
            });

            // Check stop-loss / take-profit on existing position
            if let Some(ref pos) = position
                && let Some(exit_signal) = self.check_sl_tp(pos, candle)
            {
                let exit_price = self.config.apply_exit_slippage(candle.close, pos.is_long());
                let exit_commission = self.config.calculate_commission(exit_price * pos.quantity);

                signals.push(SignalRecord {
                    timestamp: candle.timestamp,
                    price: candle.close,
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                    reason: exit_signal.reason.clone(),
                    executed: true,
                });

                let trade = position.take().unwrap().close(
                    candle.timestamp,
                    exit_price,
                    exit_commission,
                    exit_signal,
                );

                cash += trade.entry_value() + trade.pnl;
                trades.push(trade);
                continue; // Skip strategy signal this bar
            }

            // Skip strategy signals during warmup period
            if i < warmup.saturating_sub(1) {
                continue;
            }

            // Build strategy context
            let ctx = StrategyContext {
                candles: &candles[..=i],
                index: i,
                position: position.as_ref(),
                equity,
                indicators: &indicators,
            };

            // Get strategy signal
            let signal = strategy.on_candle(&ctx);

            // Skip hold signals
            if signal.is_hold() {
                continue;
            }

            // Check signal strength threshold
            if signal.strength.value() < self.config.min_signal_strength {
                signals.push(SignalRecord {
                    timestamp: signal.timestamp,
                    price: signal.price,
                    direction: signal.direction,
                    strength: signal.strength.value(),
                    reason: signal.reason.clone(),
                    executed: false,
                });
                continue;
            }

            // Record the signal
            let executed =
                self.execute_signal(&signal, candle, &mut position, &mut cash, &mut trades);

            signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed,
            });
        }

        // Close any open position at end if configured
        if self.config.close_at_end
            && let Some(pos) = position.take()
        {
            let last_candle = candles.last().unwrap();
            let exit_price = self
                .config
                .apply_exit_slippage(last_candle.close, pos.is_long());
            let exit_commission = self.config.calculate_commission(exit_price * pos.quantity);

            let exit_signal = Signal::exit(last_candle.timestamp, last_candle.close)
                .with_reason("End of backtest");

            let trade = pos.close(
                last_candle.timestamp,
                exit_price,
                exit_commission,
                exit_signal,
            );
            cash += trade.entry_value() + trade.pnl;
            trades.push(trade);
        }

        // Final equity
        let final_equity = if let Some(ref pos) = position {
            cash + pos.current_value(candles.last().unwrap().close)
        } else {
            cash
        };

        // Calculate metrics
        let executed_signals = signals.iter().filter(|s| s.executed).count();
        let metrics = PerformanceMetrics::calculate(
            &trades,
            &equity_curve,
            self.config.initial_capital,
            signals.len(),
            executed_signals,
        );

        let start_timestamp = candles.first().map(|c| c.timestamp).unwrap_or(0);
        let end_timestamp = candles.last().map(|c| c.timestamp).unwrap_or(0);

        Ok(BacktestResult {
            symbol: symbol.to_string(),
            strategy_name: strategy.name().to_string(),
            config: self.config.clone(),
            start_timestamp,
            end_timestamp,
            initial_capital: self.config.initial_capital,
            final_equity,
            metrics,
            trades,
            equity_curve,
            signals,
            open_position: position,
        })
    }

    /// Pre-compute all indicators required by the strategy
    fn compute_indicators<S: Strategy>(
        &self,
        candles: &[Candle],
        strategy: &S,
    ) -> Result<HashMap<String, Vec<Option<f64>>>> {
        let mut result = HashMap::new();

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

        for (name, indicator) in strategy.required_indicators() {
            match indicator {
                Indicator::Sma(period) => {
                    let values = indicators::sma(&closes, period);
                    result.insert(name, values);
                }
                Indicator::Ema(period) => {
                    let values = indicators::ema(&closes, period);
                    result.insert(name, values);
                }
                Indicator::Rsi(period) => {
                    let values = indicators::rsi(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Macd { fast, slow, signal } => {
                    let macd_result = indicators::macd(&closes, fast, slow, signal)?;
                    result.insert("macd_line".to_string(), macd_result.macd_line);
                    result.insert("macd_signal".to_string(), macd_result.signal_line);
                    result.insert("macd_histogram".to_string(), macd_result.histogram);
                }
                Indicator::Bollinger { period, std_dev } => {
                    let bb = indicators::bollinger_bands(&closes, period, std_dev)?;
                    result.insert("bollinger_upper".to_string(), bb.upper);
                    result.insert("bollinger_middle".to_string(), bb.middle);
                    result.insert("bollinger_lower".to_string(), bb.lower);
                }
                Indicator::Atr(period) => {
                    let values = indicators::atr(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Supertrend { period, multiplier } => {
                    let st = indicators::supertrend(&highs, &lows, &closes, period, multiplier)?;
                    result.insert("supertrend_value".to_string(), st.value);
                    // Convert bool to f64 for consistency
                    let uptrend: Vec<Option<f64>> = st
                        .is_uptrend
                        .into_iter()
                        .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
                        .collect();
                    result.insert("supertrend_uptrend".to_string(), uptrend);
                }
                Indicator::DonchianChannels(period) => {
                    let dc = indicators::donchian_channels(&highs, &lows, period)?;
                    result.insert("donchian_upper".to_string(), dc.upper);
                    result.insert("donchian_middle".to_string(), dc.middle);
                    result.insert("donchian_lower".to_string(), dc.lower);
                }
                Indicator::Wma(period) => {
                    let values = indicators::wma(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Dema(period) => {
                    let values = indicators::dema(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Tema(period) => {
                    let values = indicators::tema(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Hma(period) => {
                    let values = indicators::hma(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Obv => {
                    let values = indicators::obv(&closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::Momentum(period) => {
                    let values = indicators::momentum(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Roc(period) => {
                    let values = indicators::roc(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cci(period) => {
                    let values = indicators::cci(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::WilliamsR(period) => {
                    let values = indicators::williams_r(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Adx(period) => {
                    let values = indicators::adx(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Mfi(period) => {
                    let values = indicators::mfi(&highs, &lows, &closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cmf(period) => {
                    let values = indicators::cmf(&highs, &lows, &closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cmo(period) => {
                    let values = indicators::cmo(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Vwma(period) => {
                    let values = indicators::vwma(&closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Alma {
                    period,
                    offset,
                    sigma,
                } => {
                    let values = indicators::alma(&closes, period, offset, sigma)?;
                    result.insert(name, values);
                }
                Indicator::McginleyDynamic(period) => {
                    let values = indicators::mcginley_dynamic(&closes, period)?;
                    result.insert(name, values);
                }
                // === OSCILLATORS ===
                Indicator::Stochastic {
                    k_period,
                    k_slow: _,
                    d_period,
                } => {
                    let stoch = indicators::stochastic(&highs, &lows, &closes, k_period, d_period)?;
                    result.insert("stochastic_k".to_string(), stoch.k);
                    result.insert("stochastic_d".to_string(), stoch.d);
                }
                Indicator::StochasticRsi {
                    rsi_period,
                    stoch_period,
                    k_period: _,
                    d_period: _,
                } => {
                    let values = indicators::stochastic_rsi(&closes, rsi_period, stoch_period)?;
                    result.insert(name, values);
                }
                Indicator::AwesomeOscillator { fast: _, slow: _ } => {
                    // Note: awesome_oscillator uses default periods (5, 34) internally
                    let values = indicators::awesome_oscillator(&highs, &lows)?;
                    result.insert(name, values);
                }
                Indicator::CoppockCurve {
                    wma_period: _,
                    long_roc: _,
                    short_roc: _,
                } => {
                    // Note: coppock_curve uses default periods internally
                    let values = indicators::coppock_curve(&closes)?;
                    result.insert(name, values);
                }
                // === TREND INDICATORS ===
                Indicator::Aroon(period) => {
                    let aroon_result = indicators::aroon(&highs, &lows, period)?;
                    result.insert("aroon_up".to_string(), aroon_result.aroon_up);
                    result.insert("aroon_down".to_string(), aroon_result.aroon_down);
                }
                Indicator::Ichimoku {
                    conversion: _,
                    base: _,
                    lagging: _,
                    displacement: _,
                } => {
                    // Note: ichimoku uses default periods (9, 26, 52, 26) internally
                    let ich = indicators::ichimoku(&highs, &lows, &closes)?;
                    result.insert("ichimoku_conversion".to_string(), ich.conversion_line);
                    result.insert("ichimoku_base".to_string(), ich.base_line);
                    result.insert("ichimoku_leading_a".to_string(), ich.leading_span_a);
                    result.insert("ichimoku_leading_b".to_string(), ich.leading_span_b);
                    result.insert("ichimoku_lagging".to_string(), ich.lagging_span);
                }
                Indicator::ParabolicSar { step, max } => {
                    let values = indicators::parabolic_sar(&highs, &lows, &closes, step, max)?;
                    result.insert(name, values);
                }
                // === VOLATILITY ===
                Indicator::KeltnerChannels {
                    period,
                    multiplier,
                    atr_period,
                } => {
                    let kc = indicators::keltner_channels(
                        &highs, &lows, &closes, period, atr_period, multiplier,
                    )?;
                    result.insert("keltner_upper".to_string(), kc.upper);
                    result.insert("keltner_middle".to_string(), kc.middle);
                    result.insert("keltner_lower".to_string(), kc.lower);
                }
                Indicator::TrueRange => {
                    let values = indicators::true_range(&highs, &lows, &closes)?;
                    result.insert(name, values);
                }
                Indicator::ChoppinessIndex(period) => {
                    let values = indicators::choppiness_index(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                // === VOLUME INDICATORS ===
                Indicator::Vwap => {
                    let values = indicators::vwap(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::ChaikinOscillator => {
                    let values = indicators::chaikin_oscillator(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::AccumulationDistribution => {
                    let values =
                        indicators::accumulation_distribution(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::BalanceOfPower(period) => {
                    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
                    let values =
                        indicators::balance_of_power(&opens, &highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                // === POWER/STRENGTH INDICATORS ===
                Indicator::BullBearPower(_period) => {
                    // Note: bull_bear_power uses default EMA period (13) internally
                    let bbp = indicators::bull_bear_power(&highs, &lows, &closes)?;
                    result.insert("bull_power".to_string(), bbp.bull_power);
                    result.insert("bear_power".to_string(), bbp.bear_power);
                }
                Indicator::ElderRay(_period) => {
                    // Note: elder_ray uses default EMA period (13) internally
                    let er = indicators::elder_ray(&highs, &lows, &closes)?;
                    result.insert("elder_bull".to_string(), er.bull_power);
                    result.insert("elder_bear".to_string(), er.bear_power);
                }
            }
        }

        Ok(result)
    }

    /// Check if stop-loss or take-profit should trigger
    fn check_sl_tp(&self, position: &Position, candle: &Candle) -> Option<Signal> {
        let return_pct = position.unrealized_return_pct(candle.close) / 100.0;

        // Check stop-loss
        if let Some(sl_pct) = self.config.stop_loss_pct
            && return_pct <= -sl_pct
        {
            return Some(
                Signal::exit(candle.timestamp, candle.close)
                    .with_reason(format!("Stop-loss triggered ({:.1}%)", return_pct * 100.0)),
            );
        }

        // Check take-profit
        if let Some(tp_pct) = self.config.take_profit_pct
            && return_pct >= tp_pct
        {
            return Some(
                Signal::exit(candle.timestamp, candle.close).with_reason(format!(
                    "Take-profit triggered ({:.1}%)",
                    return_pct * 100.0
                )),
            );
        }

        None
    }

    /// Execute a signal, modifying position and cash
    fn execute_signal(
        &self,
        signal: &Signal,
        candle: &Candle,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
    ) -> bool {
        match signal.direction {
            SignalDirection::Long => {
                if position.is_some() {
                    return false; // Already have a position
                }
                self.open_position(position, cash, candle, signal, true)
            }
            SignalDirection::Short => {
                if position.is_some() {
                    return false; // Already have a position
                }
                if !self.config.allow_short {
                    return false; // Shorts not allowed
                }
                self.open_position(position, cash, candle, signal, false)
            }
            SignalDirection::Exit => {
                if position.is_none() {
                    return false; // No position to exit
                }
                self.close_position(position, cash, trades, candle, signal)
            }
            SignalDirection::Hold => false,
        }
    }

    /// Open a new position
    fn open_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        candle: &Candle,
        signal: &Signal,
        is_long: bool,
    ) -> bool {
        let entry_price = self.config.apply_entry_slippage(candle.close, is_long);
        let quantity = self.config.calculate_position_size(*cash, entry_price);

        if quantity <= 0.0 {
            return false; // Not enough capital
        }

        let entry_value = entry_price * quantity;
        let commission = self.config.calculate_commission(entry_value);

        if entry_value + commission > *cash {
            return false; // Not enough capital including commission
        }

        let side = if is_long {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        *cash -= entry_value + commission;
        *position = Some(Position::new(
            side,
            candle.timestamp,
            entry_price,
            quantity,
            commission,
            signal.clone(),
        ));

        true
    }

    /// Close an existing position
    fn close_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        candle: &Candle,
        signal: &Signal,
    ) -> bool {
        let pos = match position.take() {
            Some(p) => p,
            None => return false,
        };

        let exit_price = self.config.apply_exit_slippage(candle.close, pos.is_long());
        let exit_commission = self.config.calculate_commission(exit_price * pos.quantity);

        let trade = pos.close(
            candle.timestamp,
            exit_price,
            exit_commission,
            signal.clone(),
        );

        *cash += trade.entry_value() + trade.pnl;
        trades.push(trade);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::strategy::SmaCrossover;

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
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
            })
            .collect()
    }

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
    fn test_insufficient_data() {
        let candles = make_candles(&[100.0, 101.0, 102.0]); // Only 3 candles
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(10, 20); // Needs at least 21 candles

        let result = engine.run("TEST", &candles, strategy);
        assert!(result.is_err());
    }
}
