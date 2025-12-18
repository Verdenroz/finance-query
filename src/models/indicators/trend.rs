//! Trend Indicators
//!
//! This module implements trend-following indicators used in technical analysis.

use super::{
    AroonData, BullBearPowerData, ElderRayData, IchimokuData, MacdData, SuperTrendData, round2,
};

/// Calculate Moving Average Convergence Divergence (MACD)
///
/// MACD Line = EMA(12) - EMA(26)
/// Signal Line = EMA(9) of MACD Line
/// Histogram = MACD Line - Signal Line
pub(crate) fn macd(
    prices: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> MacdData {
    if prices.len() < slow_period + signal_period {
        return MacdData {
            macd: None,
            signal: None,
            histogram: None,
        };
    }

    // Calculate fast and slow EMAs
    let fast_ema = ema_internal(prices, fast_period);
    let slow_ema = ema_internal(prices, slow_period);

    if fast_ema.is_none() || slow_ema.is_none() {
        return MacdData {
            macd: None,
            signal: None,
            histogram: None,
        };
    }

    // Calculate MACD line series
    let mut macd_series = Vec::new();
    let multiplier_fast = 2.0 / (fast_period as f64 + 1.0);
    let multiplier_slow = 2.0 / (slow_period as f64 + 1.0);

    let fast_sum: f64 = prices[..fast_period].iter().sum();
    let mut fast_ema_val = fast_sum / fast_period as f64;

    let slow_sum: f64 = prices[..slow_period].iter().sum();
    let mut slow_ema_val = slow_sum / slow_period as f64;

    // Start from slow_period since that's when both EMAs are valid
    #[allow(clippy::needless_range_loop)]
    for i in slow_period..prices.len() {
        // Update fast EMA (it needs catch-up for the gap)
        if i >= fast_period {
            fast_ema_val = (prices[i] - fast_ema_val) * multiplier_fast + fast_ema_val;
        }
        slow_ema_val = (prices[i] - slow_ema_val) * multiplier_slow + slow_ema_val;

        macd_series.push(fast_ema_val - slow_ema_val);
    }

    if macd_series.is_empty() {
        return MacdData {
            macd: None,
            signal: None,
            histogram: None,
        };
    }

    let macd_line = macd_series[macd_series.len() - 1];

    // Calculate signal line (EMA of MACD)
    let signal_line = if macd_series.len() >= signal_period {
        ema_internal(&macd_series, signal_period)
    } else {
        None
    };

    let histogram = signal_line.map(|sig| macd_line - sig);

    MacdData {
        macd: Some(round2(macd_line)),
        signal: signal_line.map(round2),
        histogram: histogram.map(round2),
    }
}

/// Calculate Average Directional Index (ADX)
///
/// Measures trend strength (not direction).
/// Returns value between 0-100.
pub(crate) fn adx(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Option<f64> {
    if highs.len() < period + 1
        || lows.len() < period + 1
        || closes.len() < period + 1
        || period == 0
    {
        return None;
    }

    let mut tr_values = Vec::new();
    let mut plus_dm = Vec::new();
    let mut minus_dm = Vec::new();

    // Calculate True Range and Directional Movement
    for i in 1..closes.len() {
        // True Range
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        tr_values.push(tr);

        // Directional Movement
        let up_move = highs[i] - highs[i - 1];
        let down_move = lows[i - 1] - lows[i];

        let plus = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };

        plus_dm.push(plus);
        minus_dm.push(minus);
    }

    if tr_values.len() < period {
        return None;
    }

    // Smooth using Wilder's method
    let mut atr: f64 = tr_values[..period].iter().sum::<f64>() / period as f64;
    let mut smoothed_plus: f64 = plus_dm[..period].iter().sum::<f64>() / period as f64;
    let mut smoothed_minus: f64 = minus_dm[..period].iter().sum::<f64>() / period as f64;

    let mut dx_values = Vec::new();

    for i in period..tr_values.len() {
        atr = ((atr * (period - 1) as f64) + tr_values[i]) / period as f64;
        smoothed_plus = ((smoothed_plus * (period - 1) as f64) + plus_dm[i]) / period as f64;
        smoothed_minus = ((smoothed_minus * (period - 1) as f64) + minus_dm[i]) / period as f64;

        let plus_di = if atr != 0.0 {
            100.0 * smoothed_plus / atr
        } else {
            0.0
        };
        let minus_di = if atr != 0.0 {
            100.0 * smoothed_minus / atr
        } else {
            0.0
        };

        let di_sum = plus_di + minus_di;
        let dx = if di_sum != 0.0 {
            100.0 * ((plus_di - minus_di).abs() / di_sum)
        } else {
            0.0
        };

        dx_values.push(dx);
    }

    // ADX is smoothed DX
    if dx_values.len() < period {
        return None;
    }

    let mut adx_val: f64 = dx_values[..period].iter().sum::<f64>() / period as f64;

    for &dx in &dx_values[period..] {
        adx_val = ((adx_val * (period - 1) as f64) + dx) / period as f64;
    }

    Some(round2(adx_val))
}

/// Calculate Aroon Indicator
///
/// Aroon Up = ((period - periods since highest high) / period) * 100
/// Aroon Down = ((period - periods since lowest low) / period) * 100
pub(crate) fn aroon(highs: &[f64], lows: &[f64], period: usize) -> AroonData {
    if highs.len() < period || lows.len() < period || period == 0 {
        return AroonData {
            aroon_up: None,
            aroon_down: None,
        };
    }

    let recent_highs = &highs[highs.len() - period..];
    let recent_lows = &lows[lows.len() - period..];

    // Find periods since highest/lowest
    let mut periods_since_high = 0;
    let mut highest = f64::NEG_INFINITY;

    for (i, &high) in recent_highs.iter().enumerate().rev() {
        if high >= highest {
            highest = high;
            periods_since_high = recent_highs.len() - 1 - i;
        }
    }

    let mut periods_since_low = 0;
    let mut lowest = f64::INFINITY;

    for (i, &low) in recent_lows.iter().enumerate().rev() {
        if low <= lowest {
            lowest = low;
            periods_since_low = recent_lows.len() - 1 - i;
        }
    }

    let aroon_up = ((period - periods_since_high) as f64 / period as f64) * 100.0;
    let aroon_down = ((period - periods_since_low) as f64 / period as f64) * 100.0;

    AroonData {
        aroon_up: Some(round2(aroon_up)),
        aroon_down: Some(round2(aroon_down)),
    }
}

/// Calculate Parabolic SAR
///
/// Stop and Reverse indicator for trend-following.
/// Returns the current SAR value.
pub(crate) fn parabolic_sar(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    acceleration: f64,
    maximum: f64,
) -> Option<f64> {
    if highs.len() < 2 || lows.len() < 2 || closes.len() < 2 {
        return None;
    }

    let mut is_long = closes[1] > closes[0];
    let mut sar = if is_long { lows[0] } else { highs[0] };
    let mut ep = if is_long { highs[1] } else { lows[1] };
    let mut af = acceleration;

    for i in 2..closes.len() {
        // Calculate new SAR
        sar = sar + af * (ep - sar);

        // Check for reversal
        let reversed = if is_long {
            closes[i] < sar
        } else {
            closes[i] > sar
        };

        if reversed {
            is_long = !is_long;
            sar = ep;
            ep = if is_long { highs[i] } else { lows[i] };
            af = acceleration;
        } else {
            // Update EP and AF
            if is_long && highs[i] > ep {
                ep = highs[i];
                af = (af + acceleration).min(maximum);
            } else if !is_long && lows[i] < ep {
                ep = lows[i];
                af = (af + acceleration).min(maximum);
            }
        }
    }

    Some(round2(sar))
}

/// Calculate SuperTrend
///
/// Trend-following indicator based on ATR.
/// Returns (supertrend value, is_uptrend).
pub(crate) fn supertrend(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
    multiplier: f64,
) -> SuperTrendData {
    if highs.len() < period || lows.len() < period || closes.len() < period {
        return SuperTrendData {
            value: None,
            trend: None,
        };
    }

    // Calculate ATR
    let atr = calculate_atr(highs, lows, closes, period);
    if atr.is_none() {
        return SuperTrendData {
            value: None,
            trend: None,
        };
    }

    let atr_val = atr.unwrap();
    let hl_avg = (highs[highs.len() - 1] + lows[lows.len() - 1]) / 2.0;

    let basic_upper = hl_avg + (multiplier * atr_val);
    let basic_lower = hl_avg - (multiplier * atr_val);

    // Simplified SuperTrend calculation (last value only)
    let close = closes[closes.len() - 1];
    let is_uptrend = close > basic_lower;

    let supertrend = if is_uptrend { basic_lower } else { basic_upper };

    SuperTrendData {
        value: Some(round2(supertrend)),
        trend: Some(if is_uptrend {
            "up".to_string()
        } else {
            "down".to_string()
        }),
    }
}

/// Calculate Ichimoku Cloud
///
/// Returns all five Ichimoku lines.
pub(crate) fn ichimoku(highs: &[f64], lows: &[f64], closes: &[f64]) -> IchimokuData {
    if highs.len() < 52 || lows.len() < 52 || closes.len() < 52 {
        return IchimokuData {
            conversion_line: None,
            base_line: None,
            leading_span_a: None,
            leading_span_b: None,
            lagging_span: None,
        };
    }

    // Conversion Line (Tenkan-sen): (9-period high + 9-period low) / 2
    let conversion = calculate_midpoint(&highs[highs.len() - 9..], &lows[lows.len() - 9..]);

    // Base Line (Kijun-sen): (26-period high + 26-period low) / 2
    let base = calculate_midpoint(&highs[highs.len() - 26..], &lows[lows.len() - 26..]);

    // Leading Span A (Senkou Span A): (Conversion + Base) / 2, plotted 26 periods ahead
    let lead_a = if let (Some(conv), Some(b)) = (conversion, base) {
        Some(round2((conv + b) / 2.0))
    } else {
        None
    };

    // Leading Span B (Senkou Span B): (52-period high + 52-period low) / 2, plotted 26 periods ahead
    let lead_b = calculate_midpoint(&highs[highs.len() - 52..], &lows[lows.len() - 52..]);

    // Lagging Span (Chikou Span): Close plotted 26 periods behind
    let lagging = if closes.len() >= 26 {
        Some(round2(closes[closes.len() - 1]))
    } else {
        None
    };

    IchimokuData {
        conversion_line: conversion.map(round2),
        base_line: base.map(round2),
        leading_span_a: lead_a,
        leading_span_b: lead_b.map(round2),
        lagging_span: lagging,
    }
}

/// Calculate Bull Bear Power
///
/// Bull Power = High - EMA(13)
/// Bear Power = Low - EMA(13)
pub(crate) fn bull_bear_power(highs: &[f64], lows: &[f64], closes: &[f64]) -> BullBearPowerData {
    let ema13 = ema_internal(closes, 13);

    if ema13.is_none() || highs.is_empty() || lows.is_empty() {
        return BullBearPowerData {
            bull_power: None,
            bear_power: None,
        };
    }

    let ema_val = ema13.unwrap();
    let bull = highs[highs.len() - 1] - ema_val;
    let bear = lows[lows.len() - 1] - ema_val;

    BullBearPowerData {
        bull_power: Some(round2(bull)),
        bear_power: Some(round2(bear)),
    }
}

/// Calculate Elder Ray Index
///
/// Similar to Bull Bear Power but uses different interpretation.
pub(crate) fn elder_ray(highs: &[f64], lows: &[f64], closes: &[f64]) -> ElderRayData {
    let ema13 = ema_internal(closes, 13);

    if ema13.is_none() || highs.is_empty() || lows.is_empty() {
        return ElderRayData {
            bull_power: None,
            bear_power: None,
        };
    }

    let ema_val = ema13.unwrap();
    let bull = highs[highs.len() - 1] - ema_val;
    let bear = lows[lows.len() - 1] - ema_val;

    ElderRayData {
        bull_power: Some(round2(bull)),
        bear_power: Some(round2(bear)),
    }
}

// Helper functions

fn ema_internal(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut ema_value = initial_sum / period as f64;

    for &price in &prices[period..] {
        ema_value = (price - ema_value) * multiplier + ema_value;
    }

    Some(ema_value)
}

fn calculate_atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Option<f64> {
    if highs.len() < period + 1 || lows.len() < period + 1 || closes.len() < period + 1 {
        return None;
    }

    let mut tr_values = Vec::new();

    for i in 1..closes.len() {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        tr_values.push(tr);
    }

    if tr_values.len() < period {
        return None;
    }

    let mut atr: f64 = tr_values[..period].iter().sum::<f64>() / period as f64;

    for &tr in &tr_values[period..] {
        atr = ((atr * (period - 1) as f64) + tr) / period as f64;
    }

    Some(atr)
}

fn calculate_midpoint(highs: &[f64], lows: &[f64]) -> Option<f64> {
    if highs.is_empty() || lows.is_empty() {
        return None;
    }

    let highest = highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest = lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    Some((highest + lowest) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd() {
        let prices: Vec<f64> = (1..=50).map(|x| x as f64).collect();
        let result = macd(&prices, 12, 26, 9);
        assert!(result.macd.is_some());
    }

    #[test]
    fn test_adx() {
        // Need at least period*2 + 1 values for ADX calculation
        let highs: Vec<f64> = (10..=40).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=38).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=39).map(|x| x as f64).collect();

        let result = adx(&highs, &lows, &closes, 14);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((0.0..=100.0).contains(&val));
    }

    #[test]
    fn test_aroon() {
        let highs: Vec<f64> = vec![10.0, 11.0, 12.0, 11.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
        let lows: Vec<f64> = vec![8.0, 9.0, 10.0, 9.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0];

        let result = aroon(&highs, &lows, 5);
        assert!(result.aroon_up.is_some());
        assert!(result.aroon_down.is_some());
    }

    #[test]
    fn test_parabolic_sar() {
        let highs: Vec<f64> = (10..=20).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=18).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=19).map(|x| x as f64).collect();

        let result = parabolic_sar(&highs, &lows, &closes, 0.02, 0.2);
        assert!(result.is_some());
    }
}
