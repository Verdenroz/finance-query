//! Price-based indicator references.
//!
//! This module provides references to OHLCV price data that can be used
//! in trading conditions without requiring indicator computation.

use crate::indicators::Indicator;

use super::IndicatorRef;
use crate::backtesting::strategy::StrategyContext;

/// Reference to the close price.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let close_above_sma = close().above_ref(sma(200));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ClosePrice;

impl IndicatorRef for ClosePrice {
    fn key(&self) -> String {
        "close".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![] // Price doesn't need pre-computation
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        Some(ctx.close())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.close)
    }
}

/// Reference to the open price.
#[derive(Debug, Clone, Copy)]
pub struct OpenPrice;

impl IndicatorRef for OpenPrice {
    fn key(&self) -> String {
        "open".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        Some(ctx.open())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.open)
    }
}

/// Reference to the high price.
#[derive(Debug, Clone, Copy)]
pub struct HighPrice;

impl IndicatorRef for HighPrice {
    fn key(&self) -> String {
        "high".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        Some(ctx.high())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.high)
    }
}

/// Reference to the low price.
#[derive(Debug, Clone, Copy)]
pub struct LowPrice;

impl IndicatorRef for LowPrice {
    fn key(&self) -> String {
        "low".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        Some(ctx.low())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.low)
    }
}

/// Reference to the volume.
#[derive(Debug, Clone, Copy)]
pub struct VolumeRef;

impl IndicatorRef for VolumeRef {
    fn key(&self) -> String {
        "volume".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        Some(ctx.volume() as f64)
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.volume as f64)
    }
}

/// Reference to the typical price: (high + low + close) / 3
#[derive(Debug, Clone, Copy)]
pub struct TypicalPrice;

impl IndicatorRef for TypicalPrice {
    fn key(&self) -> String {
        "typical_price".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some((candle.high + candle.low + candle.close) / 3.0)
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle()
            .map(|c| (c.high + c.low + c.close) / 3.0)
    }
}

/// Reference to the median price: (high + low) / 2
#[derive(Debug, Clone, Copy)]
pub struct MedianPrice;

impl IndicatorRef for MedianPrice {
    fn key(&self) -> String {
        "median_price".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some((candle.high + candle.low) / 2.0)
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| (c.high + c.low) / 2.0)
    }
}

// ============================================================================
// DERIVED PRICE REFERENCES
// ============================================================================

/// Reference to the price change percentage from previous close.
///
/// Returns the percentage change: ((current_close - prev_close) / prev_close) * 100
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Enter on big moves (>2% change)
/// let big_move = price_change_pct().above(2.0);
/// // Exit on reversal
/// let reversal = price_change_pct().below(-1.5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PriceChangePct;

impl IndicatorRef for PriceChangePct {
    fn key(&self) -> String {
        "price_change_pct".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let current = ctx.close();
        ctx.previous_candle().map(|prev| {
            if prev.close != 0.0 {
                ((current - prev.close) / prev.close) * 100.0
            } else {
                0.0
            }
        })
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        // Need candle at idx-1 and idx-2
        let candles = &ctx.candles;
        let idx = ctx.index;
        if idx >= 2 {
            let prev = &candles[idx - 1];
            let prev_prev = &candles[idx - 2];
            if prev_prev.close != 0.0 {
                Some(((prev.close - prev_prev.close) / prev_prev.close) * 100.0)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
}

/// Reference to the gap percentage (open vs previous close).
///
/// Returns: ((current_open - prev_close) / prev_close) * 100
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Gap up strategy
/// let gap_up = gap_pct().above(1.0);
/// // Gap down reversal
/// let gap_down = gap_pct().below(-2.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct GapPct;

impl IndicatorRef for GapPct {
    fn key(&self) -> String {
        "gap_pct".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let current_open = ctx.open();
        ctx.previous_candle().map(|prev| {
            if prev.close != 0.0 {
                ((current_open - prev.close) / prev.close) * 100.0
            } else {
                0.0
            }
        })
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candles = &ctx.candles;
        let idx = ctx.index;
        if idx >= 2 {
            let prev = &candles[idx - 1];
            let prev_prev = &candles[idx - 2];
            if prev_prev.close != 0.0 {
                Some(((prev.open - prev_prev.close) / prev_prev.close) * 100.0)
            } else {
                Some(0.0)
            }
        } else {
            None
        }
    }
}

/// Reference to the candle range (high - low).
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Filter for high volatility candles
/// let wide_range = candle_range().above(5.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CandleRange;

impl IndicatorRef for CandleRange {
    fn key(&self) -> String {
        "candle_range".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some(candle.high - candle.low)
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| c.high - c.low)
    }
}

/// Reference to the candle body size (absolute difference between open and close).
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Strong candle filter
/// let strong_body = candle_body().above(2.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct CandleBody;

impl IndicatorRef for CandleBody {
    fn key(&self) -> String {
        "candle_body".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some((candle.close - candle.open).abs())
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle().map(|c| (c.close - c.open).abs())
    }
}

/// Reference to whether the current candle is bullish (close > open).
///
/// Returns 1.0 if bullish, 0.0 if bearish or doji.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Only enter on bullish candles
/// let bullish = is_bullish().above(0.5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct IsBullish;

impl IndicatorRef for IsBullish {
    fn key(&self) -> String {
        "is_bullish".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some(if candle.close > candle.open { 1.0 } else { 0.0 })
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle()
            .map(|c| if c.close > c.open { 1.0 } else { 0.0 })
    }
}

/// Reference to whether the current candle is bearish (close < open).
///
/// Returns 1.0 if bearish, 0.0 if bullish or doji.
#[derive(Debug, Clone, Copy)]
pub struct IsBearish;

impl IndicatorRef for IsBearish {
    fn key(&self) -> String {
        "is_bearish".to_string()
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![]
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candle = ctx.current_candle();
        Some(if candle.close < candle.open { 1.0 } else { 0.0 })
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        ctx.previous_candle()
            .map(|c| if c.close < c.open { 1.0 } else { 0.0 })
    }
}

// === Convenience Functions ===

/// Get a reference to the close price.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// let above_sma = price().above_ref(sma(200));
/// let crosses_ema = close().crosses_above_ref(ema(50));
/// ```
#[inline]
pub fn price() -> ClosePrice {
    ClosePrice
}

/// Get a reference to the close price.
///
/// Alias for [`price()`].
#[inline]
pub fn close() -> ClosePrice {
    ClosePrice
}

/// Get a reference to the open price.
#[inline]
pub fn open() -> OpenPrice {
    OpenPrice
}

/// Get a reference to the high price.
#[inline]
pub fn high() -> HighPrice {
    HighPrice
}

/// Get a reference to the low price.
#[inline]
pub fn low() -> LowPrice {
    LowPrice
}

/// Get a reference to the volume.
#[inline]
pub fn volume() -> VolumeRef {
    VolumeRef
}

/// Get a reference to the typical price: (high + low + close) / 3.
#[inline]
pub fn typical_price() -> TypicalPrice {
    TypicalPrice
}

/// Get a reference to the median price: (high + low) / 2.
#[inline]
pub fn median_price() -> MedianPrice {
    MedianPrice
}

/// Get a reference to the price change percentage from previous close.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Big move filter (>2% change)
/// let big_move = price_change_pct().above(2.0);
/// ```
#[inline]
pub fn price_change_pct() -> PriceChangePct {
    PriceChangePct
}

/// Get a reference to the gap percentage (open vs previous close).
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Gap up entry
/// let gap_up = gap_pct().above(1.0);
/// ```
#[inline]
pub fn gap_pct() -> GapPct {
    GapPct
}

/// Get a reference to the candle range (high - low).
#[inline]
pub fn candle_range() -> CandleRange {
    CandleRange
}

/// Get a reference to the candle body size.
#[inline]
pub fn candle_body() -> CandleBody {
    CandleBody
}

/// Returns 1.0 if the candle is bullish (close > open), 0.0 otherwise.
///
/// Use with `.above(0.5)` to check for bullish candle.
#[inline]
pub fn is_bullish() -> IsBullish {
    IsBullish
}

/// Returns 1.0 if the candle is bearish (close < open), 0.0 otherwise.
///
/// Use with `.above(0.5)` to check for bearish candle.
#[inline]
pub fn is_bearish() -> IsBearish {
    IsBearish
}

/// Reference to relative volume (current volume / average volume over N periods).
///
/// Returns ratio: 1.0 = average, 2.0 = double average, etc.
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // High volume breakout (volume > 1.5x average)
/// let high_volume = relative_volume(20).above(1.5);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RelativeVolume {
    /// Number of periods for volume average
    pub period: usize,
}

impl IndicatorRef for RelativeVolume {
    fn key(&self) -> String {
        format!("relative_volume_{}", self.period)
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        vec![] // We compute from candle history directly
    }

    fn value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candles = &ctx.candles;
        let idx = ctx.index;

        // Need at least `period` candles to compute average
        if idx < self.period {
            return None;
        }

        // Calculate average volume over the last N periods (not including current)
        let avg_volume: f64 = candles[idx.saturating_sub(self.period)..idx]
            .iter()
            .map(|c| c.volume as f64)
            .sum::<f64>()
            / self.period as f64;

        if avg_volume > 0.0 {
            let current_volume = ctx.volume() as f64;
            Some(current_volume / avg_volume)
        } else {
            None
        }
    }

    fn prev_value(&self, ctx: &StrategyContext) -> Option<f64> {
        let candles = &ctx.candles;
        let idx = ctx.index;

        if idx < self.period + 1 {
            return None;
        }

        let prev_idx = idx - 1;
        let avg_volume: f64 = candles[prev_idx.saturating_sub(self.period)..prev_idx]
            .iter()
            .map(|c| c.volume as f64)
            .sum::<f64>()
            / self.period as f64;

        if avg_volume > 0.0 {
            Some(candles[prev_idx].volume as f64 / avg_volume)
        } else {
            None
        }
    }
}

/// Get a reference to relative volume (current volume / N-period average volume).
///
/// # Arguments
///
/// * `period` - Number of periods for the volume average
///
/// # Example
///
/// ```ignore
/// use finance_query::backtesting::refs::*;
///
/// // Volume spike filter
/// let volume_spike = relative_volume(20).above(2.0);
/// ```
#[inline]
pub fn relative_volume(period: usize) -> RelativeVolume {
    RelativeVolume { period }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_keys() {
        assert_eq!(price().key(), "close");
        assert_eq!(close().key(), "close");
        assert_eq!(open().key(), "open");
        assert_eq!(high().key(), "high");
        assert_eq!(low().key(), "low");
        assert_eq!(volume().key(), "volume");
        assert_eq!(typical_price().key(), "typical_price");
        assert_eq!(median_price().key(), "median_price");
    }

    #[test]
    fn test_price_no_indicators_required() {
        assert!(price().required_indicators().is_empty());
        assert!(open().required_indicators().is_empty());
        assert!(high().required_indicators().is_empty());
        assert!(low().required_indicators().is_empty());
        assert!(volume().required_indicators().is_empty());
    }

    #[test]
    fn test_derived_price_keys() {
        assert_eq!(price_change_pct().key(), "price_change_pct");
        assert_eq!(gap_pct().key(), "gap_pct");
        assert_eq!(candle_range().key(), "candle_range");
        assert_eq!(candle_body().key(), "candle_body");
        assert_eq!(is_bullish().key(), "is_bullish");
        assert_eq!(is_bearish().key(), "is_bearish");
    }

    #[test]
    fn test_derived_price_no_indicators_required() {
        assert!(price_change_pct().required_indicators().is_empty());
        assert!(gap_pct().required_indicators().is_empty());
        assert!(candle_range().required_indicators().is_empty());
        assert!(candle_body().required_indicators().is_empty());
        assert!(is_bullish().required_indicators().is_empty());
        assert!(is_bearish().required_indicators().is_empty());
    }

    #[test]
    fn test_relative_volume_key() {
        assert_eq!(relative_volume(20).key(), "relative_volume_20");
        assert_eq!(relative_volume(10).key(), "relative_volume_10");
    }
}
