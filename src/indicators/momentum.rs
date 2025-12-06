//! Momentum Indicators
//!
//! This module implements momentum oscillators used in technical analysis.

use super::round2;

/// Calculate Relative Strength Index (RSI)
///
/// Uses Wilder's smoothing method for average gain/loss.
/// Returns RSI value between 0-100.
pub(crate) fn rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 || period == 0 {
        return None;
    }

    // Calculate price changes
    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    if gains.len() < period {
        return None;
    }

    // Initial average using SMA
    let mut avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    // Apply Wilder's smoothing for remaining values
    for i in period..gains.len() {
        avg_gain = ((avg_gain * (period - 1) as f64) + gains[i]) / period as f64;
        avg_loss = ((avg_loss * (period - 1) as f64) + losses[i]) / period as f64;
    }

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    let rsi = 100.0 - (100.0 / (1.0 + rs));

    Some(round2(rsi))
}

/// Calculate Stochastic Oscillator
///
/// Returns (%K, %D) where:
/// %K = (Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA of %K
pub(crate) fn stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    d_period: usize,
) -> (Option<f64>, Option<f64>) {
    if highs.len() < k_period || lows.len() < k_period || closes.len() < k_period || k_period == 0 {
        return (None, None);
    }

    // Calculate %K values
    let mut k_values = Vec::new();

    for i in k_period..=closes.len() {
        let start_idx = i - k_period;
        let end_idx = i - 1;
        let period_highs = &highs[start_idx..=end_idx];
        let period_lows = &lows[start_idx..=end_idx];

        let highest = period_highs
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = period_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        let range = highest - lowest;
        if range == 0.0 {
            k_values.push(50.0); // Neutral when no range
        } else {
            let k = ((closes[end_idx] - lowest) / range) * 100.0;
            k_values.push(k);
        }
    }

    let k = k_values.last().map(|&v| round2(v));

    // Calculate %D (SMA of %K)
    let d = if k_values.len() >= d_period {
        let sum: f64 = k_values[k_values.len() - d_period..].iter().sum();
        Some(round2(sum / d_period as f64))
    } else {
        None
    };

    (k, d)
}

/// Calculate Williams %R
///
/// Similar to Stochastic but inverted scale.
/// %R = (Highest High - Close) / (Highest High - Lowest Low) * -100
/// Returns value between -100 and 0.
pub(crate) fn williams_r(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Option<f64> {
    if highs.len() < period || lows.len() < period || closes.len() < period || period == 0 {
        return None;
    }

    let period_highs = &highs[highs.len() - period..];
    let period_lows = &lows[lows.len() - period..];
    let close = closes[closes.len() - 1];

    let highest = period_highs
        .iter()
        .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest = period_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    let range = highest - lowest;
    if range == 0.0 {
        return Some(-50.0); // Neutral
    }

    let wr = ((highest - close) / range) * -100.0;
    Some(round2(wr))
}

/// Calculate Stochastic RSI
///
/// Applies Stochastic formula to RSI values.
/// Returns value between 0-100.
pub(crate) fn stochastic_rsi(
    prices: &[f64],
    rsi_period: usize,
    stoch_period: usize,
) -> Option<f64> {
    if prices.len() < rsi_period + stoch_period || rsi_period == 0 || stoch_period == 0 {
        return None;
    }

    // Calculate RSI series
    let mut rsi_values = Vec::new();

    for i in rsi_period + 1..=prices.len() {
        if let Some(rsi_val) = rsi(&prices[..i], rsi_period) {
            rsi_values.push(rsi_val);
        }
    }

    if rsi_values.len() < stoch_period {
        return None;
    }

    // Apply Stochastic to RSI values
    let recent_rsi = &rsi_values[rsi_values.len() - stoch_period..];
    let current_rsi = rsi_values[rsi_values.len() - 1];

    let highest = recent_rsi.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let lowest = recent_rsi.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    let range = highest - lowest;
    if range == 0.0 {
        return Some(50.0);
    }

    let srsi = ((current_rsi - lowest) / range) * 100.0;
    Some(round2(srsi))
}

/// Calculate Chande Momentum Oscillator (CMO)
///
/// Similar to RSI but uses sum of gains - sum of losses.
/// Returns value between -100 and 100.
pub(crate) fn cmo(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 || period == 0 {
        return None;
    }

    let mut gains_sum = 0.0;
    let mut losses_sum = 0.0;

    // Calculate gains and losses over period
    for i in prices.len() - period..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains_sum += change;
        } else {
            losses_sum += -change;
        }
    }

    let total = gains_sum + losses_sum;
    if total == 0.0 {
        return Some(0.0);
    }

    let cmo = ((gains_sum - losses_sum) / total) * 100.0;
    Some(round2(cmo))
}

/// Calculate Rate of Change (ROC)
///
/// Percentage change over period.
/// ROC = ((Price - Price[n periods ago]) / Price[n periods ago]) * 100
pub(crate) fn roc(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() <= period || period == 0 {
        return None;
    }

    let current = prices[prices.len() - 1];
    let past = prices[prices.len() - 1 - period];

    if past == 0.0 {
        return None;
    }

    let roc = ((current - past) / past) * 100.0;
    Some(round2(roc))
}

/// Calculate Momentum
///
/// Simple difference between current price and price n periods ago.
pub(crate) fn momentum(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() <= period || period == 0 {
        return None;
    }

    let current = prices[prices.len() - 1];
    let past = prices[prices.len() - 1 - period];

    Some(round2(current - past))
}

/// Calculate Awesome Oscillator
///
/// AO = SMA(median price, 5) - SMA(median price, 34)
/// Median price = (High + Low) / 2
pub(crate) fn awesome_oscillator(highs: &[f64], lows: &[f64]) -> Option<f64> {
    if highs.len() < 34 || lows.len() < 34 {
        return None;
    }

    // Calculate median prices
    let mut median_prices = Vec::new();
    for i in 0..highs.len() {
        median_prices.push((highs[i] + lows[i]) / 2.0);
    }

    // Calculate SMAs
    let sma5_sum: f64 = median_prices[median_prices.len() - 5..].iter().sum();
    let sma5 = sma5_sum / 5.0;

    let sma34_sum: f64 = median_prices[median_prices.len() - 34..].iter().sum();
    let sma34 = sma34_sum / 34.0;

    Some(round2(sma5 - sma34))
}

/// Calculate Coppock Curve
///
/// Combines ROC with WMA smoothing.
/// Coppock = WMA(ROC(14) + ROC(11), 10)
pub(crate) fn coppock_curve(prices: &[f64]) -> Option<f64> {
    if prices.len() < 25 {
        // Need at least 14 + 10 + 1
        return None;
    }

    // Calculate ROC series for both periods
    let mut roc_sum_series = Vec::new();

    for i in 14..prices.len() {
        let roc14 = if i >= 14 && prices[i - 14] != 0.0 {
            ((prices[i] - prices[i - 14]) / prices[i - 14]) * 100.0
        } else {
            0.0
        };

        let roc11 = if i >= 11 && prices[i - 11] != 0.0 {
            ((prices[i] - prices[i - 11]) / prices[i - 11]) * 100.0
        } else {
            0.0
        };

        roc_sum_series.push(roc14 + roc11);
    }

    // Apply WMA with period 10
    if roc_sum_series.len() < 10 {
        return None;
    }

    let recent = &roc_sum_series[roc_sum_series.len() - 10..];
    let mut weighted_sum = 0.0;
    let mut weight_sum = 0.0;

    for (i, &value) in recent.iter().enumerate() {
        let weight = (i + 1) as f64;
        weighted_sum += value * weight;
        weight_sum += weight;
    }

    Some(round2(weighted_sum / weight_sum))
}

/// Calculate Commodity Channel Index (CCI)
///
/// CCI measures the variation of a security's price from its statistical mean.
/// Formula: CCI = (Typical Price - SMA of Typical Price) / (0.015 * Mean Deviation)
///
/// Typical Price = (High + Low + Close) / 3
pub(crate) fn cci(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Option<f64> {
    if highs.len() < period || lows.len() < period || closes.len() < period || period == 0 {
        return None;
    }

    let len = highs.len().min(lows.len()).min(closes.len());
    if len < period {
        return None;
    }

    // Calculate Typical Prices
    let mut typical_prices = Vec::with_capacity(len);
    for i in 0..len {
        typical_prices.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    // Calculate SMA of Typical Price for the period
    let sum: f64 = typical_prices[len - period..].iter().sum();
    let sma = sum / period as f64;

    // Calculate Mean Deviation
    let deviations_sum: f64 = typical_prices[len - period..]
        .iter()
        .map(|&tp| (tp - sma).abs())
        .sum();
    let mean_deviation = deviations_sum / period as f64;

    // Avoid division by zero
    if mean_deviation == 0.0 {
        return Some(0.0);
    }

    // Calculate CCI
    let latest_tp = typical_prices[len - 1];
    let cci = (latest_tp - sma) / (0.015 * mean_deviation);

    Some(round2(cci))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        // Simple trending up prices should give high RSI
        let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = rsi(&prices, 5);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!(val > 50.0 && val <= 100.0); // Should be in upper range
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let prices = vec![1.0, 2.0, 3.0];
        assert_eq!(rsi(&prices, 5), None);
    }

    #[test]
    fn test_stochastic() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];

        let (k, _d) = stochastic(&highs, &lows, &closes, 3, 3);
        assert!(k.is_some());
        let k_val = k.unwrap();
        assert!((0.0..=100.0).contains(&k_val));
    }

    #[test]
    fn test_williams_r() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];

        let result = williams_r(&highs, &lows, &closes, 3);
        assert!(result.is_some());
        let val = result.unwrap();
        assert!((-100.0..=0.0).contains(&val));
    }

    #[test]
    fn test_roc() {
        let prices = vec![100.0, 105.0, 110.0, 115.0];
        let result = roc(&prices, 2);
        assert!(result.is_some());
        // (115 - 105) / 105 * 100 = ~9.52
        let val = result.unwrap();
        assert!((val - 9.52).abs() < 0.1);
    }

    #[test]
    fn test_momentum() {
        let prices = vec![100.0, 105.0, 110.0, 115.0];
        let result = momentum(&prices, 2);
        assert_eq!(result, Some(10.0)); // 115 - 105 = 10
    }

    #[test]
    fn test_cci() {
        // Test with sample data
        let highs = vec![50.0, 52.0, 51.0, 53.0, 54.0, 55.0];
        let lows = vec![48.0, 49.0, 48.5, 50.0, 51.0, 52.0];
        let closes = vec![49.0, 51.0, 50.0, 52.0, 53.0, 54.0];

        let result = cci(&highs, &lows, &closes, 5);
        assert!(result.is_some());
        // CCI typically ranges from -100 to +100, but can exceed these values
        let val = result.unwrap();
        assert!(val.is_finite());
    }

    #[test]
    fn test_cci_insufficient_data() {
        let highs = vec![50.0, 52.0];
        let lows = vec![48.0, 49.0];
        let closes = vec![49.0, 51.0];

        assert_eq!(cci(&highs, &lows, &closes, 5), None);
    }
}
