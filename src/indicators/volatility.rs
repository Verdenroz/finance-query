//! Volatility Indicators
//!
//! This module implements volatility and price channel indicators.

use super::{BollingerBandsData, DonchianChannelsData, KeltnerChannelsData, round2};

/// Calculate Bollinger Bands
///
/// Middle Band = SMA(period)
/// Upper Band = Middle Band + (std_dev * standard deviation)
/// Lower Band = Middle Band - (std_dev * standard deviation)
pub(crate) fn bollinger_bands(
    prices: &[f64],
    period: usize,
    std_dev_mult: f64,
) -> BollingerBandsData {
    if prices.len() < period || period == 0 {
        return BollingerBandsData {
            upper: None,
            middle: None,
            lower: None,
        };
    }

    let recent = &prices[prices.len() - period..];

    // Calculate SMA (middle band)
    let sum: f64 = recent.iter().sum();
    let sma = sum / period as f64;

    // Calculate standard deviation
    let variance: f64 = recent.iter().map(|&x| (x - sma).powi(2)).sum::<f64>() / period as f64;
    let std_dev = variance.sqrt();

    let upper = sma + (std_dev_mult * std_dev);
    let lower = sma - (std_dev_mult * std_dev);

    BollingerBandsData {
        upper: Some(round2(upper)),
        middle: Some(round2(sma)),
        lower: Some(round2(lower)),
    }
}

/// Calculate Average True Range (ATR)
///
/// Measures market volatility.
/// Uses Wilder's smoothing method.
pub(crate) fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Option<f64> {
    if highs.len() < period + 1
        || lows.len() < period + 1
        || closes.len() < period + 1
        || period == 0
    {
        return None;
    }

    let mut tr_values = Vec::new();

    // Calculate True Range values
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

    // Initial ATR using SMA
    let mut atr_value: f64 = tr_values[..period].iter().sum::<f64>() / period as f64;

    // Apply Wilder's smoothing
    for &tr in &tr_values[period..] {
        atr_value = ((atr_value * (period - 1) as f64) + tr) / period as f64;
    }

    Some(round2(atr_value))
}

/// Calculate True Range (single value)
///
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
pub(crate) fn true_range(highs: &[f64], lows: &[f64], closes: &[f64]) -> Option<f64> {
    if highs.len() < 2 || lows.len() < 2 || closes.len() < 2 {
        return None;
    }

    let i = highs.len() - 1;
    let high_low = highs[i] - lows[i];
    let high_close = (highs[i] - closes[i - 1]).abs();
    let low_close = (lows[i] - closes[i - 1]).abs();

    Some(round2(high_low.max(high_close).max(low_close)))
}

/// Calculate Keltner Channels
///
/// Middle Line = EMA(period)
/// Upper Channel = EMA + (multiplier * ATR)
/// Lower Channel = EMA - (multiplier * ATR)
pub(crate) fn keltner_channels(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
    atr_period: usize,
    multiplier: f64,
) -> KeltnerChannelsData {
    if closes.len() < period {
        return KeltnerChannelsData {
            upper: None,
            middle: None,
            lower: None,
        };
    }

    // Calculate EMA (middle line)
    let ema_val = ema_internal(closes, period);
    if ema_val.is_none() {
        return KeltnerChannelsData {
            upper: None,
            middle: None,
            lower: None,
        };
    }

    let middle = ema_val.unwrap();

    // Calculate ATR
    let atr_val = atr(highs, lows, closes, atr_period);
    if atr_val.is_none() {
        return KeltnerChannelsData {
            upper: Some(round2(middle)),
            middle: Some(round2(middle)),
            lower: Some(round2(middle)),
        };
    }

    let atr_value = atr_val.unwrap();
    let upper = middle + (multiplier * atr_value);
    let lower = middle - (multiplier * atr_value);

    KeltnerChannelsData {
        upper: Some(round2(upper)),
        middle: Some(round2(middle)),
        lower: Some(round2(lower)),
    }
}

/// Calculate Donchian Channels
///
/// Upper Channel = Highest high over period
/// Lower Channel = Lowest low over period
/// Middle Channel = (Upper + Lower) / 2
pub(crate) fn donchian_channels(
    highs: &[f64],
    lows: &[f64],
    period: usize,
) -> DonchianChannelsData {
    if highs.len() < period || lows.len() < period || period == 0 {
        return DonchianChannelsData {
            upper: None,
            middle: None,
            lower: None,
        };
    }

    let recent_highs = &highs[highs.len() - period..];
    let recent_lows = &lows[lows.len() - period..];

    let upper = recent_highs
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lower = recent_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let middle = (upper + lower) / 2.0;

    DonchianChannelsData {
        upper: Some(round2(upper)),
        middle: Some(round2(middle)),
        lower: Some(round2(lower)),
    }
}

/// Calculate Choppiness Index
///
/// Measures market choppiness (consolidation vs trending).
/// Returns value between 0-100.
/// Above 61.8 = choppy/consolidating
/// Below 38.2 = trending
pub(crate) fn choppiness_index(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Option<f64> {
    if highs.len() < period + 1
        || lows.len() < period + 1
        || closes.len() < period + 1
        || period == 0
    {
        return None;
    }

    // Calculate sum of True Range over period
    let mut tr_sum = 0.0;
    for i in closes.len() - period..closes.len() {
        if i > 0 {
            let high_low = highs[i] - lows[i];
            let high_close = (highs[i] - closes[i - 1]).abs();
            let low_close = (lows[i] - closes[i - 1]).abs();
            let tr = high_low.max(high_close).max(low_close);
            tr_sum += tr;
        }
    }

    // Calculate high-low range over period
    let recent_highs = &highs[highs.len() - period..];
    let recent_lows = &lows[lows.len() - period..];

    let highest = recent_highs
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest = recent_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let range = highest - lowest;

    if range == 0.0 || tr_sum == 0.0 {
        return Some(50.0); // Neutral
    }

    // Choppiness Index formula
    let ci = 100.0 * (tr_sum / range).ln() / (period as f64).ln();

    Some(round2(ci))
}

// Helper function

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands() {
        let prices = vec![100.0, 102.0, 101.0, 103.0, 104.0, 103.0, 105.0];
        let result = bollinger_bands(&prices, 5, 2.0);

        assert!(result.upper.is_some());
        assert!(result.middle.is_some());
        assert!(result.lower.is_some());

        let upper = result.upper.unwrap();
        let middle = result.middle.unwrap();
        let lower = result.lower.unwrap();

        assert!(upper > middle);
        assert!(middle > lower);
    }

    #[test]
    fn test_atr() {
        let highs: Vec<f64> = (10..=25).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=23).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=24).map(|x| x as f64).collect();

        let result = atr(&highs, &lows, &closes, 14);
        assert!(result.is_some());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_true_range() {
        let highs = vec![10.0, 11.0, 12.0];
        let lows = vec![8.0, 9.0, 10.0];
        let closes = vec![9.0, 10.0, 11.0];

        let result = true_range(&highs, &lows, &closes);
        assert!(result.is_some());
        assert!(result.unwrap() >= 0.0);
    }

    #[test]
    fn test_keltner_channels() {
        let highs: Vec<f64> = (10..=30).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=28).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=29).map(|x| x as f64).collect();

        let result = keltner_channels(&highs, &lows, &closes, 20, 10, 2.0);
        assert!(result.upper.is_some());
        assert!(result.middle.is_some());
        assert!(result.lower.is_some());
    }

    #[test]
    fn test_donchian_channels() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];

        let result = donchian_channels(&highs, &lows, 5);
        assert_eq!(result.upper, Some(12.0));
        assert_eq!(result.lower, Some(8.0));
        assert_eq!(result.middle, Some(10.0));
    }

    #[test]
    fn test_choppiness_index() {
        let highs: Vec<f64> = (10..=25).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=23).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=24).map(|x| x as f64).collect();

        let result = choppiness_index(&highs, &lows, &closes, 14);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((0.0..=100.0).contains(&val));
    }
}
