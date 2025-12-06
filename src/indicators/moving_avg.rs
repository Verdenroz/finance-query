//! Moving Average Indicators
//!
//! This module implements various moving average calculations used in technical analysis.

use super::round2;

/// Calculate Simple Moving Average (SMA)
///
/// Returns the last value of the SMA for the given period.
pub(crate) fn sma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let sum: f64 = prices[prices.len() - period..].iter().sum();
    Some(round2(sum / period as f64))
}

/// Calculate Exponential Moving Average (EMA)
///
/// Uses SMA for initialization, then applies exponential smoothing.
/// Returns the last EMA value.
pub(crate) fn ema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Initialize with SMA
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut ema_value = initial_sum / period as f64;

    // Apply EMA formula for remaining values
    for &price in &prices[period..] {
        ema_value = (price - ema_value) * multiplier + ema_value;
    }

    Some(round2(ema_value))
}

/// Calculate Weighted Moving Average (WMA)
///
/// More recent prices have higher weight.
/// Returns the last WMA value.
pub(crate) fn wma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let recent = &prices[prices.len() - period..];
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (i, &price) in recent.iter().enumerate() {
        let weight = (i + 1) as f64;
        weighted_sum += price * weight;
        weight_sum += weight;
    }

    Some(round2(weighted_sum / weight_sum))
}

/// Calculate Volume Weighted Moving Average (VWMA)
///
/// Prices weighted by volume over the given period.
/// Returns the last VWMA value.
pub(crate) fn vwma(prices: &[f64], volumes: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || volumes.len() < period || period == 0 {
        return None;
    }

    let price_slice = &prices[prices.len() - period..];
    let volume_slice = &volumes[volumes.len() - period..];

    let mut pv_sum = 0.0;
    let mut volume_sum = 0.0;

    for (&price, &volume) in price_slice.iter().zip(volume_slice.iter()) {
        pv_sum += price * volume;
        volume_sum += volume;
    }

    if volume_sum == 0.0 {
        return None;
    }

    Some(round2(pv_sum / volume_sum))
}

/// Calculate Double Exponential Moving Average (DEMA)
///
/// DEMA = 2 * EMA - EMA(EMA)
/// Reduces lag compared to simple EMA.
pub(crate) fn dema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period * 2 || period == 0 {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate first EMA series
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut ema1 = initial_sum / period as f64;
    let mut ema1_series = Vec::with_capacity(prices.len() - period + 1);
    ema1_series.push(ema1);

    for &price in &prices[period..] {
        ema1 = (price - ema1) * multiplier + ema1;
        ema1_series.push(ema1);
    }

    // Calculate EMA of EMA
    if ema1_series.len() < period {
        return None;
    }

    let ema2_sum: f64 = ema1_series[..period].iter().sum();
    let mut ema2 = ema2_sum / period as f64;

    for &value in &ema1_series[period..] {
        ema2 = (value - ema2) * multiplier + ema2;
    }

    // DEMA = 2 * EMA - EMA(EMA)
    Some(round2(2.0 * ema1 - ema2))
}

/// Calculate Triple Exponential Moving Average (TEMA)
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
/// Further reduces lag compared to DEMA.
pub(crate) fn tema(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period * 3 || period == 0 {
        return None;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);

    // Calculate first EMA series
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut ema1 = initial_sum / period as f64;
    let mut ema1_series = Vec::with_capacity(prices.len() - period + 1);
    ema1_series.push(ema1);

    for &price in &prices[period..] {
        ema1 = (price - ema1) * multiplier + ema1;
        ema1_series.push(ema1);
    }

    // Calculate EMA of EMA
    if ema1_series.len() < period {
        return None;
    }

    let ema2_sum: f64 = ema1_series[..period].iter().sum();
    let mut ema2 = ema2_sum / period as f64;
    let mut ema2_series = Vec::with_capacity(ema1_series.len() - period + 1);
    ema2_series.push(ema2);

    for &value in &ema1_series[period..] {
        ema2 = (value - ema2) * multiplier + ema2;
        ema2_series.push(ema2);
    }

    // Calculate EMA of EMA of EMA
    if ema2_series.len() < period {
        return None;
    }

    let ema3_sum: f64 = ema2_series[..period].iter().sum();
    let mut ema3 = ema3_sum / period as f64;

    for &value in &ema2_series[period..] {
        ema3 = (value - ema3) * multiplier + ema3;
    }

    // TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
    Some(round2(3.0 * ema1 - 3.0 * ema2 + ema3))
}

/// Calculate Hull Moving Average (HMA)
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
/// Responsive moving average with reduced lag.
pub(crate) fn hma(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let half_period = period / 2;
    let sqrt_period = (period as f64).sqrt() as usize;

    if sqrt_period == 0 || prices.len() < period {
        return None;
    }

    // Calculate WMA with half period for entire series
    let mut wma_half_series = Vec::new();
    for i in half_period..=prices.len() {
        if let Some(val) = wma_internal(&prices[i - half_period..i], half_period) {
            wma_half_series.push(val);
        }
    }

    // Calculate WMA with full period for entire series
    let mut wma_full_series = Vec::new();
    for i in period..=prices.len() {
        if let Some(val) = wma_internal(&prices[i - period..i], period) {
            wma_full_series.push(val);
        }
    }

    // Align series (wma_half_series is longer)
    let offset = wma_half_series.len() - wma_full_series.len();

    // Calculate 2 * WMA(half) - WMA(full)
    let mut diff_series = Vec::new();
    for i in 0..wma_full_series.len() {
        diff_series.push(2.0 * wma_half_series[i + offset] - wma_full_series[i]);
    }

    // Final WMA with sqrt period
    if diff_series.len() < sqrt_period {
        return None;
    }

    wma_internal(&diff_series[diff_series.len() - sqrt_period..], sqrt_period).map(round2)
}

/// Internal WMA calculation without rounding
fn wma_internal(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (i, &price) in prices.iter().enumerate() {
        let weight = (i + 1) as f64;
        weighted_sum += price * weight;
        weight_sum += weight;
    }

    Some(weighted_sum / weight_sum)
}

/// Calculate Arnaud Legoux Moving Average (ALMA)
///
/// Gaussian-weighted moving average with configurable offset and sigma.
/// Standard parameters: offset=0.85, sigma=6.0
pub(crate) fn alma(prices: &[f64], period: usize, offset: f64, sigma: f64) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    let recent = &prices[prices.len() - period..];
    let m = (period as f64 - 1.0) * offset;
    let s = period as f64 / sigma;

    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (i, &price) in recent.iter().enumerate() {
        let weight = (-(i as f64 - m).powi(2) / (2.0 * s.powi(2))).exp();
        weighted_sum += price * weight;
        weight_sum += weight;
    }

    if weight_sum == 0.0 {
        return None;
    }

    Some(round2(weighted_sum / weight_sum))
}

/// Calculate McGinley Dynamic
///
/// Adaptive moving average that automatically adjusts for market speed.
/// MD[i] = MD[i-1] + (Price - MD[i-1]) / (N * (Price/MD[i-1])^4)
pub(crate) fn mcginley_dynamic(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period || period == 0 {
        return None;
    }

    // Initialize with SMA
    let initial_sum: f64 = prices[..period].iter().sum();
    let mut md = initial_sum / period as f64;

    // Apply McGinley Dynamic formula
    for &price in &prices[period..] {
        if md != 0.0 {
            let ratio = price / md;
            let factor = period as f64 * ratio.powi(4);
            if factor != 0.0 {
                md = md + (price - md) / factor;
            }
        }
    }

    Some(round2(md))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(sma(&prices, 3), Some(4.0)); // (3+4+5)/3 = 4
        assert_eq!(sma(&prices, 5), Some(3.0)); // (1+2+3+4+5)/5 = 3
    }

    #[test]
    fn test_ema() {
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema(&prices, 3);
        assert!(result.is_some());
        // EMA should be between min and max
        let val = result.unwrap();
        assert!((1.0..=5.0).contains(&val));
    }

    #[test]
    fn test_wma() {
        let prices = vec![1.0, 2.0, 3.0];
        // WMA = (1*1 + 2*2 + 3*3) / (1+2+3) = 14/6 = 2.33
        assert_eq!(wma(&prices, 3), Some(2.33));
    }

    #[test]
    fn test_vwma() {
        let prices = vec![10.0, 20.0, 30.0];
        let volumes = vec![100.0, 200.0, 300.0];
        // VWMA = (10*100 + 20*200 + 30*300) / (100+200+300) = 14000/600 = 23.33
        assert_eq!(vwma(&prices, &volumes, 3), Some(23.33));
    }

    #[test]
    fn test_insufficient_data() {
        let prices = vec![1.0, 2.0];
        assert_eq!(sma(&prices, 5), None);
        assert_eq!(ema(&prices, 5), None);
        assert_eq!(wma(&prices, 5), None);
    }
}
