//! Volume Indicators
//!
//! This module implements volume-based indicators used in technical analysis.

use super::round2;

/// Calculate On Balance Volume (OBV)
///
/// Cumulative indicator that adds/subtracts volume based on price direction.
pub(crate) fn obv(closes: &[f64], volumes: &[f64]) -> Option<f64> {
    if closes.len() < 2 || volumes.len() < 2 || closes.len() != volumes.len() {
        return None;
    }

    let mut obv_value = 0.0;

    for i in 1..closes.len() {
        if closes[i] > closes[i - 1] {
            obv_value += volumes[i];
        } else if closes[i] < closes[i - 1] {
            obv_value -= volumes[i];
        }
        // If close == previous close, OBV unchanged
    }

    Some(round2(obv_value))
}

/// Calculate Chaikin Money Flow (CMF)
///
/// Measures buying/selling pressure over a period.
/// Returns value between -1 and 1.
pub(crate) fn cmf(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
    period: usize,
) -> Option<f64> {
    if highs.len() < period
        || lows.len() < period
        || closes.len() < period
        || volumes.len() < period
        || period == 0
    {
        return None;
    }

    let mut mf_volume_sum = 0.0;
    let mut volume_sum = 0.0;

    for i in closes.len() - period..closes.len() {
        let high_low = highs[i] - lows[i];
        if high_low == 0.0 {
            continue;
        }

        // Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
        let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;

        // Money Flow Volume = MF Multiplier * Volume
        let mf_volume = mf_multiplier * volumes[i];

        mf_volume_sum += mf_volume;
        volume_sum += volumes[i];
    }

    if volume_sum == 0.0 {
        return None;
    }

    Some(round2(mf_volume_sum / volume_sum))
}

/// Calculate Money Flow Index (MFI)
///
/// Volume-weighted RSI. Returns value between 0-100.
pub(crate) fn mfi(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
    period: usize,
) -> Option<f64> {
    if highs.len() < period + 1
        || lows.len() < period + 1
        || closes.len() < period + 1
        || volumes.len() < period + 1
        || period == 0
    {
        return None;
    }

    // Calculate typical prices
    let mut typical_prices = Vec::new();
    for i in 0..closes.len() {
        typical_prices.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    // Calculate money flow
    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    for i in closes.len() - period..closes.len() {
        if i == 0 {
            continue;
        }

        let raw_money_flow = typical_prices[i] * volumes[i];

        if typical_prices[i] > typical_prices[i - 1] {
            positive_flow += raw_money_flow;
        } else if typical_prices[i] < typical_prices[i - 1] {
            negative_flow += raw_money_flow;
        }
    }

    if negative_flow == 0.0 {
        return Some(100.0);
    }

    let money_ratio = positive_flow / negative_flow;
    let mfi = 100.0 - (100.0 / (1.0 + money_ratio));

    Some(round2(mfi))
}

/// Calculate Accumulation/Distribution Line (A/D)
///
/// Cumulative indicator of money flow.
pub(crate) fn accumulation_distribution(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Option<f64> {
    if highs.is_empty() || lows.is_empty() || closes.is_empty() || volumes.is_empty() {
        return None;
    }

    let mut ad_value = 0.0;

    for i in 0..closes.len() {
        let high_low = highs[i] - lows[i];
        if high_low == 0.0 {
            continue;
        }

        // Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
        let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;

        // A/D = Previous A/D + (MF Multiplier * Volume)
        ad_value += mf_multiplier * volumes[i];
    }

    Some(round2(ad_value))
}

/// Calculate Chaikin Oscillator
///
/// Difference between 3-day EMA and 10-day EMA of A/D line.
pub(crate) fn chaikin_oscillator(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Option<f64> {
    if highs.len() < 10 || lows.len() < 10 || closes.len() < 10 || volumes.len() < 10 {
        return None;
    }

    // Calculate A/D line series
    let mut ad_series = Vec::new();
    let mut ad_cumulative = 0.0;

    for i in 0..closes.len() {
        let high_low = highs[i] - lows[i];
        if high_low != 0.0 {
            let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;
            ad_cumulative += mf_multiplier * volumes[i];
        }
        ad_series.push(ad_cumulative);
    }

    // Calculate EMA(3) of A/D
    let ema3 = ema_internal(&ad_series, 3);

    // Calculate EMA(10) of A/D
    let ema10 = ema_internal(&ad_series, 10);

    if ema3.is_none() || ema10.is_none() {
        return None;
    }

    Some(round2(ema3.unwrap() - ema10.unwrap()))
}

/// Calculate Volume Weighted Average Price (VWAP)
///
/// Average price weighted by volume.
pub(crate) fn vwap(highs: &[f64], lows: &[f64], closes: &[f64], volumes: &[f64]) -> Option<f64> {
    if highs.is_empty() || lows.is_empty() || closes.is_empty() || volumes.is_empty() {
        return None;
    }

    let mut pv_sum = 0.0;
    let mut volume_sum = 0.0;

    for i in 0..closes.len() {
        // Typical price
        let typical_price = (highs[i] + lows[i] + closes[i]) / 3.0;
        pv_sum += typical_price * volumes[i];
        volume_sum += volumes[i];
    }

    if volume_sum == 0.0 {
        return None;
    }

    Some(round2(pv_sum / volume_sum))
}

/// Calculate Balance of Power (BOP)
///
/// Measures strength of buyers vs sellers.
/// BOP = (Close - Open) / (High - Low)
pub(crate) fn balance_of_power(
    opens: &[f64],
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
) -> Option<f64> {
    if opens.is_empty() || highs.is_empty() || lows.is_empty() || closes.is_empty() {
        return None;
    }

    let i = closes.len() - 1;
    let high_low = highs[i] - lows[i];

    if high_low == 0.0 {
        return Some(0.0);
    }

    let bop = (closes[i] - opens[i]) / high_low;
    Some(round2(bop))
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
    fn test_obv() {
        let closes = vec![10.0, 11.0, 10.5, 11.5, 12.0];
        let volumes = vec![1000.0, 1500.0, 1200.0, 1800.0, 2000.0];

        let result = obv(&closes, &volumes);
        assert!(result.is_some());
        // Up, Down, Up, Up = 1500 - 1200 + 1800 + 2000 = 4100
    }

    #[test]
    fn test_cmf() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];
        let volumes = vec![1000.0, 1500.0, 1200.0, 1800.0, 2000.0];

        let result = cmf(&highs, &lows, &closes, &volumes, 5);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((-1.0..=1.0).contains(&val));
    }

    #[test]
    fn test_mfi() {
        let highs: Vec<f64> = (10..=25).map(|x| x as f64).collect();
        let lows: Vec<f64> = (8..=23).map(|x| x as f64).collect();
        let closes: Vec<f64> = (9..=24).map(|x| x as f64).collect();
        let volumes: Vec<f64> = (1000..=1015).map(|x| x as f64 * 100.0).collect();

        let result = mfi(&highs, &lows, &closes, &volumes, 14);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((0.0..=100.0).contains(&val));
    }

    #[test]
    fn test_accumulation_distribution() {
        let highs = vec![10.0, 11.0, 12.0];
        let lows = vec![8.0, 9.0, 10.0];
        let closes = vec![9.0, 10.0, 11.0];
        let volumes = vec![1000.0, 1500.0, 1200.0];

        let result = accumulation_distribution(&highs, &lows, &closes, &volumes);
        assert!(result.is_some());
    }

    #[test]
    fn test_vwap() {
        let highs = vec![10.0, 11.0, 12.0];
        let lows = vec![8.0, 9.0, 10.0];
        let closes = vec![9.0, 10.0, 11.0];
        let volumes = vec![1000.0, 1500.0, 1200.0];

        let result = vwap(&highs, &lows, &closes, &volumes);
        assert!(result.is_some());
        assert!(result.unwrap() > 0.0);
    }

    #[test]
    fn test_balance_of_power() {
        let opens = vec![9.0, 10.0, 11.0];
        let highs = vec![10.0, 11.0, 12.0];
        let lows = vec![8.0, 9.0, 10.0];
        let closes = vec![9.5, 10.5, 11.5];

        let result = balance_of_power(&opens, &highs, &lows, &closes);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((-1.0..=1.0).contains(&val));
    }
}
