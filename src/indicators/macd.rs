//! Moving Average Convergence Divergence (MACD) indicator.

use super::{IndicatorError, Result, ema::ema_raw};
use serde::{Deserialize, Serialize};

/// MACD calculation result containing the MACD line, signal line, and histogram.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacdResult {
    /// MACD line (fast EMA - slow EMA)
    pub macd_line: Vec<Option<f64>>,

    /// Signal line (EMA of MACD line)
    pub signal_line: Vec<Option<f64>>,

    /// Histogram (MACD line - signal line)
    pub histogram: Vec<Option<f64>>,
}

/// Calculate Moving Average Convergence Divergence (MACD).
///
/// MACD shows the relationship between two moving averages and helps identify trend changes.
/// Standard parameters are (12, 26, 9).
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `fast_period` - Fast EMA period (typically 12)
/// * `slow_period` - Slow EMA period (typically 26)
/// * `signal_period` - Signal line EMA period (typically 9)
///
/// # Formula
///
/// - MACD Line = 12-period EMA - 26-period EMA
/// - Signal Line = 9-period EMA of MACD Line
/// - Histogram = MACD Line - Signal Line
///
/// # Example
///
/// ```
/// use finance_query::indicators::macd;
///
/// let prices: Vec<f64> = (1..=50).map(|x| x as f64).collect();
/// let result = macd(&prices, 12, 26, 9).unwrap();
///
/// assert_eq!(result.macd_line.len(), prices.len());
/// assert_eq!(result.signal_line.len(), prices.len());
/// assert_eq!(result.histogram.len(), prices.len());
/// ```
pub fn macd(
    data: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Result<MacdResult> {
    if fast_period == 0 || slow_period == 0 || signal_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "All periods must be greater than 0".to_string(),
        ));
    }

    if fast_period >= slow_period {
        return Err(IndicatorError::InvalidPeriod(
            "Fast period must be less than slow period".to_string(),
        ));
    }

    let min_data_points = slow_period + signal_period;
    if data.len() < min_data_points {
        return Err(IndicatorError::InsufficientData {
            need: min_data_points,
            got: data.len(),
        });
    }

    // Compute EMAs using raw variant (no Option/None padding)
    let fast_raw = ema_raw(data, fast_period); // len = N - (fast_period-1)
    let slow_raw = ema_raw(data, slow_period); // len = N - (slow_period-1)

    // MACD line: fast - slow, valid from original index slow_period - 1
    // fast_raw[k + (slow_period - fast_period)] aligns with slow_raw[k]
    let shift = slow_period - fast_period;
    let macd_values: Vec<f64> = slow_raw
        .iter()
        .enumerate()
        .map(|(k, &s)| fast_raw[k + shift] - s)
        .collect();

    // Signal line: EMA of MACD values
    let signal_raw = ema_raw(&macd_values, signal_period);

    // Build full-length output vectors
    let macd_start = slow_period - 1;
    let signal_start = macd_start + signal_period - 1;
    let n = data.len();

    let mut macd_line = vec![None; n];
    let mut signal_line = vec![None; n];
    let mut histogram = vec![None; n];

    for (k, &mv) in macd_values.iter().enumerate() {
        let i = k + macd_start;
        macd_line[i] = Some(mv);
    }
    for (k, &sv) in signal_raw.iter().enumerate() {
        let i = k + signal_start;
        signal_line[i] = Some(sv);
        if let Some(mv) = macd_line[i] {
            histogram[i] = Some(mv - sv);
        }
    }

    Ok(MacdResult {
        macd_line,
        signal_line,
        histogram,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macd_basic() {
        let data: Vec<f64> = (1..=50).map(|x| x as f64).collect();
        let result = macd(&data, 12, 26, 9).unwrap();

        assert_eq!(result.macd_line.len(), 50);
        assert_eq!(result.signal_line.len(), 50);
        assert_eq!(result.histogram.len(), 50);

        // Early values should be None
        assert!(result.macd_line[0].is_none());
        assert!(result.signal_line[0].is_none());
        assert!(result.histogram[0].is_none());

        // Later values should have data
        assert!(result.macd_line[40].is_some());
    }

    #[test]
    fn test_macd_invalid_periods() {
        let data: Vec<f64> = (1..=50).map(|x| x as f64).collect();

        // Fast period >= slow period
        let result = macd(&data, 26, 12, 9);
        assert!(result.is_err());

        // Zero period
        let result = macd(&data, 0, 26, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_macd_insufficient_data() {
        let data = vec![1.0, 2.0, 3.0];
        let result = macd(&data, 12, 26, 9);

        assert!(result.is_err());
    }
}
