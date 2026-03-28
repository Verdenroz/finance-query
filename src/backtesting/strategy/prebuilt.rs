//! Pre-built trading strategies.
//!
//! Ready-to-use strategy implementations that can be used directly with the backtest engine.
//! Each strategy implements the [`Strategy`] trait and can be customized via builder methods.
//!
//! Short signals are always emitted when the condition is met. Whether they are
//! *executed* is controlled solely by [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig).
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
//! let strategy = SmaCrossover::new(10, 20);
//! let config = BacktestConfig::builder().allow_short(true).build().unwrap();
//! ```

use std::collections::HashMap;

use crate::indicators::Indicator;

use super::{Signal, Strategy, StrategyContext};
use crate::backtesting::signal::SignalStrength;

// ── Per-key pointer cache ─────────────────────────────────────────────────────

/// One-slot pointer cache for a pre-computed indicator `Vec`.
///
/// Set once by [`Strategy::setup`] before the simulation loop; dereferenced on
/// every bar in [`Strategy::on_candle`].  Clones as `None` so a cloned strategy
/// is safe to pass to a fresh `BacktestEngine::run` call (the engine will call
/// `setup` again).
///
/// # Safety invariant
/// The pointer is valid for the duration of the enclosing `simulate()` call: it
/// is taken from the `indicators` HashMap which is owned by that frame, never
/// mutated during the loop, and outlives all `on_candle` calls.
#[derive(Debug, Default)]
struct IndicatorSlot(Option<*const Vec<Option<f64>>>);

// SAFETY: The pointer is only read inside the engine's simulation loop (single-
// threaded).  We never send it across threads while it could be dangling.
unsafe impl Send for IndicatorSlot {}
unsafe impl Sync for IndicatorSlot {}

impl Clone for IndicatorSlot {
    /// Returns an empty slot — the clone must go through `setup()` before use.
    fn clone(&self) -> Self {
        IndicatorSlot(None)
    }
}

impl IndicatorSlot {
    fn set(&mut self, v: &Vec<Option<f64>>) {
        self.0 = Some(v as *const _);
    }

    /// Returns the cached slice, if set.
    ///
    /// # Safety
    /// Must only be called during a simulation loop whose `setup()` populated
    /// this slot from a HashMap that is still alive and unmodified.
    #[inline]
    unsafe fn get(&self) -> Option<&Vec<Option<f64>>> {
        self.0.map(|p| unsafe { &*p })
    }
}

/// SMA Crossover Strategy
///
/// Goes long when fast SMA crosses above slow SMA.
/// Exits when fast SMA crosses below slow SMA.
/// Emits short signals on bearish crossovers (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
#[derive(Debug, Clone)]
pub struct SmaCrossover {
    /// Fast SMA period
    pub fast_period: usize,
    /// Slow SMA period
    pub slow_period: usize,
    fast_key: String,
    slow_key: String,
    fast_slot: IndicatorSlot,
    slow_slot: IndicatorSlot,
}

impl SmaCrossover {
    /// Create a new SMA crossover strategy
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            fast_period,
            slow_period,
            fast_key: format!("sma_{fast_period}"),
            slow_key: format!("sma_{slow_period}"),
            fast_slot: IndicatorSlot::default(),
            slow_slot: IndicatorSlot::default(),
        }
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
            (self.fast_key.clone(), Indicator::Sma(self.fast_period)),
            (self.slow_key.clone(), Indicator::Sma(self.slow_period)),
        ]
    }

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.fast_key) {
            self.fast_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.slow_key) {
            self.slow_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.slow_period.max(self.fast_period) + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let i = ctx.index;
        if i == 0 {
            return Signal::hold();
        }

        // Use cached pointer (0 HashMap lookups); fall back to map lookup if
        // setup() was not called (e.g., strategy used outside the engine).
        // SAFETY: setup() was called from simulate() with the indicators map
        // that is alive and unmodified for the duration of the loop.
        let fast_vals =
            unsafe { self.fast_slot.get() }.or_else(|| ctx.indicators.get(&self.fast_key));
        let slow_vals =
            unsafe { self.slow_slot.get() }.or_else(|| ctx.indicators.get(&self.slow_key));
        let (Some(fast_vals), Some(slow_vals)) = (fast_vals, slow_vals) else {
            return Signal::hold();
        };

        let get = |vals: &Vec<Option<f64>>, idx: usize| vals.get(idx).and_then(|&v| v);
        let (Some(fn_), Some(sn), Some(fp), Some(sp)) = (
            get(fast_vals, i),
            get(slow_vals, i),
            get(fast_vals, i - 1),
            get(slow_vals, i - 1),
        ) else {
            return Signal::hold();
        };

        // Bullish crossover: fast crosses above slow
        if fp < sp && fn_ > sn {
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
        if fp > sp && fn_ < sn {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("SMA bearish crossover - close long");
            }
            if !ctx.has_position() {
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
/// Emits short signals when RSI crosses below overbought (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
#[derive(Debug, Clone)]
pub struct RsiReversal {
    /// RSI period
    pub period: usize,
    /// Oversold threshold (default 30)
    pub oversold: f64,
    /// Overbought threshold (default 70)
    pub overbought: f64,
    rsi_key: String,
    rsi_slot: IndicatorSlot,
}

impl RsiReversal {
    /// Create a new RSI reversal strategy
    pub fn new(period: usize) -> Self {
        Self {
            period,
            oversold: 30.0,
            overbought: 70.0,
            rsi_key: format!("rsi_{period}"),
            rsi_slot: IndicatorSlot::default(),
        }
    }

    /// Set custom oversold/overbought thresholds
    pub fn with_thresholds(mut self, oversold: f64, overbought: f64) -> Self {
        self.oversold = oversold;
        self.overbought = overbought;
        self
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
        vec![(self.rsi_key.clone(), Indicator::Rsi(self.period))]
    }

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.rsi_key) {
            self.rsi_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let i = ctx.index;

        // SAFETY: see SmaCrossover::on_candle.
        let rsi_vals = unsafe { self.rsi_slot.get() }.or_else(|| ctx.indicators.get(&self.rsi_key));
        let Some(rsi_vals) = rsi_vals else {
            return Signal::hold();
        };
        let get = |idx: usize| rsi_vals.get(idx).and_then(|&v| v);
        let Some(rsi_val) = get(i) else {
            return Signal::hold();
        };
        let rsi_prev = if i > 0 { get(i - 1) } else { None };

        // Calculate signal strength based on RSI extremity
        let strength = if !(20.0..=80.0).contains(&rsi_val) {
            SignalStrength::strong()
        } else if !(25.0..=75.0).contains(&rsi_val) {
            SignalStrength::medium()
        } else {
            SignalStrength::weak()
        };

        // Bullish: RSI crosses above oversold
        let crossed_above_oversold =
            rsi_prev.is_some_and(|p| p <= self.oversold) && rsi_val > self.oversold;
        if crossed_above_oversold {
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
        let crossed_below_overbought =
            rsi_prev.is_some_and(|p| p >= self.overbought) && rsi_val < self.overbought;
        if crossed_below_overbought {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_strength(strength)
                    .with_reason(format!(
                        "RSI crossed below {:.0} - close long",
                        self.overbought
                    ));
            }
            if !ctx.has_position() {
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
/// Emits short signals on bearish crossovers (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
#[derive(Debug, Clone)]
pub struct MacdSignal {
    /// Fast EMA period
    pub fast: usize,
    /// Slow EMA period
    pub slow: usize,
    /// Signal line period
    pub signal: usize,
    line_key: String,
    sig_key: String,
    line_slot: IndicatorSlot,
    sig_slot: IndicatorSlot,
}

impl MacdSignal {
    /// Create a new MACD signal strategy
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            line_key: format!("macd_line_{fast}_{slow}_{signal}"),
            sig_key: format!("macd_signal_{fast}_{slow}_{signal}"),
            line_slot: IndicatorSlot::default(),
            sig_slot: IndicatorSlot::default(),
        }
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

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.line_key) {
            self.line_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.sig_key) {
            self.sig_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.slow + self.signal
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let i = ctx.index;
        if i == 0 {
            return Signal::hold();
        }

        // SAFETY: see SmaCrossover::on_candle.
        let line_vals =
            unsafe { self.line_slot.get() }.or_else(|| ctx.indicators.get(&self.line_key));
        let sig_vals = unsafe { self.sig_slot.get() }.or_else(|| ctx.indicators.get(&self.sig_key));
        let (Some(line_vals), Some(sig_vals)) = (line_vals, sig_vals) else {
            return Signal::hold();
        };

        let get = |vals: &Vec<Option<f64>>, idx: usize| vals.get(idx).and_then(|&v| v);
        let (Some(ln), Some(sn), Some(lp), Some(sp)) = (
            get(line_vals, i),
            get(sig_vals, i),
            get(line_vals, i - 1),
            get(sig_vals, i - 1),
        ) else {
            return Signal::hold();
        };

        // MACD line and signal line are stored separately by the engine
        // Bullish crossover
        if lp < sp && ln > sn {
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
        if lp > sp && ln < sn {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, candle.close)
                    .with_reason("MACD bearish crossover - close long");
            }
            if !ctx.has_position() {
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
/// Emits short signals when price touches upper band (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
///
/// # Signal Strength
///
/// All entry signals emit at default strength (`1.0`). Strength is **not** scaled
/// by how far price has penetrated through the band. This differs from
/// [`RsiReversal`], which grades strength by RSI extremity. If you are relying
/// on [`BacktestConfig::min_signal_strength`] to filter signals in a portfolio
/// context, all Bollinger entries will pass the threshold equally.
#[derive(Debug, Clone)]
pub struct BollingerMeanReversion {
    /// SMA period for middle band
    pub period: usize,
    /// Standard deviation multiplier
    pub std_dev: f64,
    /// Exit at middle band (true) or upper/lower band (false)
    pub exit_at_middle: bool,
    lower_key: String,
    middle_key: String,
    upper_key: String,
    lower_slot: IndicatorSlot,
    middle_slot: IndicatorSlot,
    upper_slot: IndicatorSlot,
}

impl BollingerMeanReversion {
    /// Create a new Bollinger mean reversion strategy
    pub fn new(period: usize, std_dev: f64) -> Self {
        Self {
            period,
            std_dev,
            exit_at_middle: true,
            lower_key: format!("bollinger_lower_{period}_{std_dev}"),
            middle_key: format!("bollinger_middle_{period}_{std_dev}"),
            upper_key: format!("bollinger_upper_{period}_{std_dev}"),
            lower_slot: IndicatorSlot::default(),
            middle_slot: IndicatorSlot::default(),
            upper_slot: IndicatorSlot::default(),
        }
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

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.lower_key) {
            self.lower_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.middle_key) {
            self.middle_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.upper_key) {
            self.upper_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let close = candle.close;
        let i = ctx.index;

        // SAFETY: see SmaCrossover::on_candle.
        let lower_vals =
            unsafe { self.lower_slot.get() }.or_else(|| ctx.indicators.get(&self.lower_key));
        let middle_vals =
            unsafe { self.middle_slot.get() }.or_else(|| ctx.indicators.get(&self.middle_key));
        let upper_vals =
            unsafe { self.upper_slot.get() }.or_else(|| ctx.indicators.get(&self.upper_key));
        let (Some(lower_vals), Some(middle_vals), Some(upper_vals)) =
            (lower_vals, middle_vals, upper_vals)
        else {
            return Signal::hold();
        };

        let get = |vals: &Vec<Option<f64>>, idx: usize| vals.get(idx).and_then(|&v| v);
        let (Some(lower_val), Some(middle_val), Some(upper_val)) =
            (get(lower_vals, i), get(middle_vals, i), get(upper_vals, i))
        else {
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
        if close >= upper_val && !ctx.has_position() {
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
/// Emits short signals when SuperTrend turns bearish (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
#[derive(Debug, Clone)]
pub struct SuperTrendFollow {
    /// ATR period
    pub period: usize,
    /// ATR multiplier
    pub multiplier: f64,
    uptrend_key: String,
    uptrend_slot: IndicatorSlot,
}

impl SuperTrendFollow {
    /// Create a new SuperTrend following strategy
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            uptrend_key: format!("supertrend_uptrend_{period}_{multiplier}"),
            uptrend_slot: IndicatorSlot::default(),
        }
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

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.uptrend_key) {
            self.uptrend_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let i = ctx.index;

        // SAFETY: see SmaCrossover::on_candle.
        let vals =
            unsafe { self.uptrend_slot.get() }.or_else(|| ctx.indicators.get(&self.uptrend_key));
        let Some(vals) = vals else {
            return Signal::hold();
        };
        let get = |idx: usize| vals.get(idx).and_then(|&v| v);
        let (Some(now), Some(prev)) = (get(i), if i > 0 { get(i - 1) } else { None }) else {
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
            if !ctx.has_position() {
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
/// Emits short signals on downward breakouts (execution gated by
/// [`BacktestConfig::allow_short`](crate::backtesting::BacktestConfig)).
#[derive(Debug, Clone)]
pub struct DonchianBreakout {
    /// Channel period
    pub period: usize,
    /// Use middle channel for exit (true) or opposite channel (false)
    pub exit_at_middle: bool,
    upper_key: String,
    middle_key: String,
    lower_key: String,
    upper_slot: IndicatorSlot,
    middle_slot: IndicatorSlot,
    lower_slot: IndicatorSlot,
}

impl DonchianBreakout {
    /// Create a new Donchian breakout strategy
    pub fn new(period: usize) -> Self {
        Self {
            period,
            exit_at_middle: true,
            upper_key: format!("donchian_upper_{period}"),
            middle_key: format!("donchian_middle_{period}"),
            lower_key: format!("donchian_lower_{period}"),
            upper_slot: IndicatorSlot::default(),
            middle_slot: IndicatorSlot::default(),
            lower_slot: IndicatorSlot::default(),
        }
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

    fn setup(&mut self, indicators: &HashMap<String, Vec<Option<f64>>>) {
        if let Some(v) = indicators.get(&self.upper_key) {
            self.upper_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.middle_key) {
            self.middle_slot.set(v);
        }
        if let Some(v) = indicators.get(&self.lower_key) {
            self.lower_slot.set(v);
        }
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let candle = ctx.current_candle();
        let close = candle.close;
        let i = ctx.index;

        // SAFETY: see SmaCrossover::on_candle.
        let upper_vals =
            unsafe { self.upper_slot.get() }.or_else(|| ctx.indicators.get(&self.upper_key));
        let middle_vals =
            unsafe { self.middle_slot.get() }.or_else(|| ctx.indicators.get(&self.middle_key));
        let lower_vals =
            unsafe { self.lower_slot.get() }.or_else(|| ctx.indicators.get(&self.lower_key));
        let (Some(upper_vals), Some(middle_vals), Some(lower_vals)) =
            (upper_vals, middle_vals, lower_vals)
        else {
            return Signal::hold();
        };
        let get = |vals: &Vec<Option<f64>>, idx: usize| vals.get(idx).and_then(|&v| v);
        let (Some(_upper_val), Some(middle_val), Some(_lower_val)) =
            (get(upper_vals, i), get(middle_vals, i), get(lower_vals, i))
        else {
            return Signal::hold();
        };
        let prev_upper = if i > 0 { get(upper_vals, i - 1) } else { None };
        let prev_lower = if i > 0 { get(lower_vals, i - 1) } else { None };

        // Breakout above the *previous* bar's upper channel level → go long.
        // Using the lagged level rather than the current bar's channel prevents
        // look-ahead bias: the current bar's Donchian high is computed using the
        // close of that same bar, so comparing `close > current_upper` would
        // trivially never trigger (the close can equal but not exceed the max
        // of the window it belongs to).  The lagged level is the natural
        // reference point for a confirmed breakout signal.
        if let Some(prev_up) = prev_upper
            && close > prev_up
            && !ctx.has_position()
        {
            return Signal::long(candle.timestamp, close)
                .with_reason("Donchian upper channel breakout");
        }

        // Breakdown below the *previous* bar's lower channel level (same
        // lagged-reference rationale as the upper channel breakout above).
        if let Some(prev_low) = prev_lower
            && close < prev_low
        {
            if ctx.is_long() {
                return Signal::exit(candle.timestamp, close)
                    .with_reason("Donchian lower channel breakdown - close long");
            }
            if !ctx.has_position() {
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
    }

    #[test]
    fn test_sma_crossover_custom() {
        let s = SmaCrossover::new(5, 15);
        assert_eq!(s.fast_period, 5);
        assert_eq!(s.slow_period, 15);
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
