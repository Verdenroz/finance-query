//! Pre-built trading strategies.
//!
//! Ready-to-use strategy implementations that can be used directly with the backtest engine.
//! Each strategy implements the [`Strategy`] trait and can be customized via builder methods.
//!
//! # Available Strategies
//!
//! | Strategy | Description |
//! |----------|-------------|
//! | [`SmaCrossover`] | Dual SMA crossover (trend following) |
//! | [`RsiReversal`] | RSI mean reversion |
//! | [`MacdSignal`] | MACD line crossover |
//! | [`BollingerMeanReversion`] | Bollinger Bands mean reversion |
//! | [`SuperTrendFollow`] | SuperTrend trend following |
//! | [`DonchianBreakout`] | Donchian channel breakout |
//!
//! # Example
//!
//! ```ignore
//! use finance_query::backtesting::{SmaCrossover, BacktestConfig};
//!
//! // Use a pre-built strategy directly
//! let strategy = SmaCrossover::new(10, 20).with_short(true);
//! ```

use crate::indicators::Indicator;

use super::{Signal, Strategy, StrategyContext};
use crate::backtesting::signal::SignalStrength;

/// SMA Crossover Strategy
///
/// Goes long when fast SMA crosses above slow SMA.
/// Exits when fast SMA crosses below slow SMA.
/// Optionally goes short on bearish crossovers.
#[derive(Debug, Clone)]
pub struct SmaCrossover {
    /// Fast SMA period
    pub fast_period: usize,
    /// Slow SMA period
    pub slow_period: usize,
    /// Allow short positions
    pub allow_short: bool,
}

impl SmaCrossover {
    /// Create a new SMA crossover strategy
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            allow_short: false,
        }
    }

    /// Enable short positions on bearish crossovers
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }

    fn fast_key(&self) -> String {
        format!("sma_{}", self.fast_period)
    }

    fn slow_key(&self) -> String {
        format!("sma_{}", self.slow_period)
    }
}

impl Default for SmaCrossover {
    fn default() -> Self {
        Self::new(10, 20)
    }
}

impl Strategy for SmaCrossover {
    fn name(&self) -> &str {
        "SMA Crossover"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![
            (self.fast_key(), Indicator::Sma(self.fast_period)),
            (self.slow_key(), Indicator::Sma(self.slow_period)),
        ]
    }

    fn warmup_period(&self) -> usize {
        self.slow_period.max(self.fast_period) + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();

        // Bullish crossover: fast crosses above slow
        if ctx.crossed_above(&self.fast_key(), &self.slow_key()) {
            if ctx.is_short() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("SMA bullish crossover - close short");
            }
            if !ctx.has_position() {
                return Signal::long(candle.timestamp, candle.close)
                    .with_reason("SMA bullish crossover");
            }
        }

        // Bearish crossover: fast crosses below slow
        if ctx.crossed_below(&self.fast_key(), &self.slow_key()) {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("SMA bearish crossover - close long");
            }
            if !ctx.has_position() && self.allow_short {
                return Signal::short(candle.timestamp, candle.close)
                    .with_reason("SMA bearish crossover");
            }
        }

        Signal::hold()
    }
}

/// RSI Reversal Strategy
///
/// Goes long when RSI crosses above oversold level.
/// Exits when RSI reaches overbought level.
/// Optionally goes short when RSI crosses below overbought level.
#[derive(Debug, Clone)]
pub struct RsiReversal {
    /// RSI period
    pub period: usize,
    /// Oversold threshold (default 30)
    pub oversold: f64,
    /// Overbought threshold (default 70)
    pub overbought: f64,
    /// Allow short positions
    pub allow_short: bool,
}

impl RsiReversal {
    /// Create a new RSI reversal strategy
    pub fn new(period: usize) -> Self {
        Self {
            period,
            oversold: 30.0,
            overbought: 70.0,
            allow_short: false,
        }
    }

    /// Set custom oversold/overbought thresholds
    pub fn with_thresholds(mut self, oversold: f64, overbought: f64) -> Self {
        self.oversold = oversold;
        self.overbought = overbought;
        self
    }

    /// Enable short positions
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }

    fn rsi_key(&self) -> String {
        format!("rsi_{}", self.period)
    }
}

impl Default for RsiReversal {
    fn default() -> Self {
        Self::new(14)
    }
}

impl Strategy for RsiReversal {
    fn name(&self) -> &str {
        "RSI Reversal"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(self.rsi_key(), Indicator::Rsi(self.period))]
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let rsi = ctx.indicator(&self.rsi_key());

        let Some(rsi_val) = rsi else {
            return Signal::hold();
        };

        // Calculate signal strength based on RSI extremity
        let strength = if !(20.0..=80.0).contains(&rsi_val) {
            SignalStrength::strong()
        } else if !(25.0..=75.0).contains(&rsi_val) {
            SignalStrength::medium()
        } else {
            SignalStrength::weak()
        };

        // Bullish: RSI crosses above oversold
        if ctx.indicator_crossed_above(&self.rsi_key(), self.oversold) {
            if ctx.is_short() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_strength(strength)
                    .with_reason(format!(
                        "RSI crossed above {:.0} - close short",
                        self.oversold
                    ));
            }
            if !ctx.has_position() {
                return Signal::long(candle.timestamp, candle.close)
                    .with_strength(strength)
                    .with_reason(format!("RSI crossed above {:.0}", self.oversold));
            }
        }

        // Bearish: RSI crosses below overbought
        if ctx.indicator_crossed_below(&self.rsi_key(), self.overbought) {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_strength(strength)
                    .with_reason(format!(
                        "RSI crossed below {:.0} - close long",
                        self.overbought
                    ));
            }
            if !ctx.has_position() && self.allow_short {
                return Signal::short(candle.timestamp, candle.close)
                    .with_strength(strength)
                    .with_reason(format!("RSI crossed below {:.0}", self.overbought));
            }
        }

        Signal::hold()
    }
}

/// MACD Signal Strategy
///
/// Goes long when MACD line crosses above signal line.
/// Exits when MACD line crosses below signal line.
/// Optionally goes short on bearish crossovers.
#[derive(Debug, Clone)]
pub struct MacdSignal {
    /// Fast EMA period
    pub fast: usize,
    /// Slow EMA period
    pub slow: usize,
    /// Signal line period
    pub signal: usize,
    /// Allow short positions
    pub allow_short: bool,
}

impl MacdSignal {
    /// Create a new MACD signal strategy
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            allow_short: false,
        }
    }

    /// Enable short positions
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }
}

impl Default for MacdSignal {
    fn default() -> Self {
        Self::new(12, 26, 9)
    }
}

impl Strategy for MacdSignal {
    fn name(&self) -> &str {
        "MACD Signal"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            "macd".to_string(),
            Indicator::Macd {
                fast: self.fast,
                slow: self.slow,
                signal: self.signal,
            },
        )]
    }

    fn warmup_period(&self) -> usize {
        self.slow + self.signal
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();

        // MACD line and signal line are stored separately by the engine
        // Bullish crossover
        if ctx.crossed_above("macd_line", "macd_signal") {
            if ctx.is_short() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("MACD bullish crossover - close short");
            }
            if !ctx.has_position() {
                return Signal::long(candle.timestamp, candle.close)
                    .with_reason("MACD bullish crossover");
            }
        }

        // Bearish crossover
        if ctx.crossed_below("macd_line", "macd_signal") {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("MACD bearish crossover - close long");
            }
            if !ctx.has_position() && self.allow_short {
                return Signal::short(candle.timestamp, candle.close)
                    .with_reason("MACD bearish crossover");
            }
        }

        Signal::hold()
    }
}

/// Bollinger Bands Mean Reversion Strategy
///
/// Goes long when price touches lower band (oversold).
/// Exits when price reaches middle or upper band.
/// Optionally goes short when price touches upper band.
#[derive(Debug, Clone)]
pub struct BollingerMeanReversion {
    /// SMA period for middle band
    pub period: usize,
    /// Standard deviation multiplier
    pub std_dev: f64,
    /// Allow short positions
    pub allow_short: bool,
    /// Exit at middle band (true) or upper/lower band (false)
    pub exit_at_middle: bool,
}

impl BollingerMeanReversion {
    /// Create a new Bollinger mean reversion strategy
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            allow_short: false,
            exit_at_middle: true,
        }
    }

    /// Enable short positions
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }

    /// Set exit target (middle band or opposite band)
    pub fn exit_at_middle(mut self, at_middle: bool) -> Self {
        self.exit_at_middle = at_middle;
        self
    }
}

impl Default for BollingerMeanReversion {
    fn default() -> Self {
        Self::new(20, 2.0)
    }
}

impl Strategy for BollingerMeanReversion {
    fn name(&self) -> &str {
        "Bollinger Mean Reversion"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            "bollinger".to_string(),
            Indicator::Bollinger {
                period: self.period,
                std_dev: self.std_dev,
            },
        )]
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let close = candle.close;

        let lower = ctx.indicator("bollinger_lower");
        let middle = ctx.indicator("bollinger_middle");
        let upper = ctx.indicator("bollinger_upper");

        let (Some(lower_val), Some(middle_val), Some(upper_val)) = (lower, middle, upper) else {
            return Signal::hold();
        };

        // Long entry: price at or below lower band
        if close <= lower_val && !ctx.has_position() {
            return Signal::long(candle.timestamp, close)
                .with_reason("Price at lower Bollinger Band");
        }

        // Long exit
        if ctx.is_long() {
            let exit_level = if self.exit_at_middle {
                middle_val
            } else {
                upper_val
            };
            if close >= exit_level {
                return Signal::exit(candle.timestamp, close).with_reason(format!(
                    "Price reached {} Bollinger Band",
                    if self.exit_at_middle {
                        "middle"
                    } else {
                        "upper"
                    }
                ));
            }
        }

        // Short entry: price at or above upper band
        if close >= upper_val && !ctx.has_position() && self.allow_short {
            return Signal::short(candle.timestamp, close)
                .with_reason("Price at upper Bollinger Band");
        }

        // Short exit
        if ctx.is_short() {
            let exit_level = if self.exit_at_middle {
                middle_val
            } else {
                lower_val
            };
            if close <= exit_level {
                return Signal::exit(candle.timestamp, close).with_reason(format!(
                    "Price reached {} Bollinger Band",
                    if self.exit_at_middle {
                        "middle"
                    } else {
                        "lower"
                    }
                ));
            }
        }

        Signal::hold()
    }
}

/// SuperTrend Following Strategy
///
/// Goes long when SuperTrend turns bullish (uptrend).
/// Goes short when SuperTrend turns bearish (downtrend).
#[derive(Debug, Clone)]
pub struct SuperTrendFollow {
    /// ATR period
    pub period: usize,
    /// ATR multiplier
    pub multiplier: f64,
    /// Allow short positions
    pub allow_short: bool,
}

impl SuperTrendFollow {
    /// Create a new SuperTrend following strategy
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            allow_short: false,
        }
    }

    /// Enable short positions
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }
}

impl Default for SuperTrendFollow {
    fn default() -> Self {
        Self::new(10, 3.0)
    }
}

impl Strategy for SuperTrendFollow {
    fn name(&self) -> &str {
        "SuperTrend Follow"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            "supertrend".to_string(),
            Indicator::Supertrend {
                period: self.period,
                multiplier: self.multiplier,
            },
        )]
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();

        // SuperTrend uptrend stored as 1.0, downtrend as 0.0
        let trend_now = ctx.indicator("supertrend_uptrend");
        let trend_prev = ctx.indicator_prev("supertrend_uptrend");

        let (Some(now), Some(prev)) = (trend_now, trend_prev) else {
            return Signal::hold();
        };

        let is_uptrend = now > 0.5;
        let was_uptrend = prev > 0.5;

        // Trend changed to bullish
        if is_uptrend && !was_uptrend {
            if ctx.is_short() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("SuperTrend turned bullish - close short");
            }
            if !ctx.has_position() {
                return Signal::long(candle.timestamp, candle.close)
                    .with_reason("SuperTrend turned bullish");
            }
        }

        // Trend changed to bearish
        if !is_uptrend && was_uptrend {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("SuperTrend turned bearish - close long");
            }
            if !ctx.has_position() && self.allow_short {
                return Signal::short(candle.timestamp, candle.close)
                    .with_reason("SuperTrend turned bearish");
            }
        }

        Signal::hold()
    }
}

/// Donchian Channel Breakout Strategy
///
/// Goes long when price breaks above upper channel (new high).
/// Exits when price breaks below lower channel (new low).
/// Optionally goes short on downward breakouts.
#[derive(Debug, Clone)]
pub struct DonchianBreakout {
    /// Channel period
    pub period: usize,
    /// Allow short positions
    pub allow_short: bool,
    /// Use middle channel for exit (true) or opposite channel (false)
    pub exit_at_middle: bool,
}

impl DonchianBreakout {
    /// Create a new Donchian breakout strategy
    pub fn new(period: usize) -> Self {
        Self {
            period,
            allow_short: false,
            exit_at_middle: true,
        }
    }

    /// Enable short positions
    pub fn with_short(mut self, allow: bool) -> Self {
        self.allow_short = allow;
        self
    }

    /// Set exit at middle channel
    pub fn exit_at_middle(mut self, at_middle: bool) -> Self {
        self.exit_at_middle = at_middle;
        self
    }
}

impl Default for DonchianBreakout {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Strategy for DonchianBreakout {
    fn name(&self) -> &str {
        "Donchian Breakout"
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![(
            "donchian".to_string(),
            Indicator::DonchianChannels(self.period),
        )]
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let close = candle.close;

        let upper = ctx.indicator("donchian_upper");
        let middle = ctx.indicator("donchian_middle");
        let lower = ctx.indicator("donchian_lower");
        let prev_upper = ctx.indicator_prev("donchian_upper");
        let prev_lower = ctx.indicator_prev("donchian_lower");

        let (Some(_upper_val), Some(middle_val), Some(_lower_val)) = (upper, middle, lower) else {
            return Signal::hold();
        };

        // Breakout above previous upper channel -> go long
        if let Some(prev_up) = prev_upper
            && close > prev_up
            && !ctx.has_position()
        {
            return Signal::long(candle.timestamp, close)
                .with_reason("Donchian upper channel breakout");
        }

        // Breakout below previous lower channel
        if let Some(prev_low) = prev_lower
            && close < prev_low
        {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, close)
                    .with_reason("Donchian lower channel breakdown - close long");
            }
            if !ctx.has_position() && self.allow_short {
                return Signal::short(candle.timestamp, close)
                    .with_reason("Donchian lower channel breakdown");
            }
        }

        // Exit long at middle
        if ctx.is_long() && self.exit_at_middle && close <= middle_val {
            return Signal::exit(candle.timestamp, close)
                .with_reason("Price reached Donchian middle channel");
        }

        // Exit short at middle
        if ctx.is_short() && self.exit_at_middle && close >= middle_val {
            return Signal::exit(candle.timestamp, close)
                .with_reason("Price reached Donchian middle channel");
        }

        Signal::hold()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_crossover_default() {
        let s = SmaCrossover::default();
        assert_eq!(s.fast_period, 10);
        assert_eq!(s.slow_period, 20);
        assert!(!s.allow_short);
    }

    #[test]
    fn test_sma_crossover_with_short() {
        let s = SmaCrossover::new(5, 15).with_short(true);
        assert_eq!(s.fast_period, 5);
        assert_eq!(s.slow_period, 15);
        assert!(s.allow_short);
    }

    #[test]
    fn test_rsi_default() {
        let s = RsiReversal::default();
        assert_eq!(s.period, 14);
        assert!((s.oversold - 30.0).abs() < 0.01);
        assert!((s.overbought - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_rsi_with_thresholds() {
        let s = RsiReversal::new(10).with_thresholds(25.0, 75.0);
        assert_eq!(s.period, 10);
        assert!((s.oversold - 25.0).abs() < 0.01);
        assert!((s.overbought - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_macd_default() {
        let s = MacdSignal::default();
        assert_eq!(s.fast, 12);
        assert_eq!(s.slow, 26);
        assert_eq!(s.signal, 9);
    }

    #[test]
    fn test_bollinger_default() {
        let s = BollingerMeanReversion::default();
        assert_eq!(s.period, 20);
        assert!((s.std_dev - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_supertrend_default() {
        let s = SuperTrendFollow::default();
        assert_eq!(s.period, 10);
        assert!((s.multiplier - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_donchian_default() {
        let s = DonchianBreakout::default();
        assert_eq!(s.period, 20);
        assert!(s.exit_at_middle);
    }

    #[test]
    fn test_strategy_names() {
        assert_eq!(SmaCrossover::default().name(), "SMA Crossover");
        assert_eq!(RsiReversal::default().name(), "RSI Reversal");
        assert_eq!(MacdSignal::default().name(), "MACD Signal");
        assert_eq!(
            BollingerMeanReversion::default().name(),
            "Bollinger Mean Reversion"
        );
        assert_eq!(SuperTrendFollow::default().name(), "SuperTrend Follow");
        assert_eq!(DonchianBreakout::default().name(), "Donchian Breakout");
    }

    #[test]
    fn test_required_indicators() {
        let sma = SmaCrossover::new(5, 10);
        let indicators = sma.required_indicators();
        assert_eq!(indicators.len(), 2);
        assert_eq!(indicators[0].0, "sma_5");
        assert_eq!(indicators[1].0, "sma_10");

        let rsi = RsiReversal::new(14);
        let indicators = rsi.required_indicators();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].0, "rsi_14");
    }
}
