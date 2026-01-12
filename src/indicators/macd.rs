//! Moving Average Convergence Divergence (MACD) indicator.

use super::{IndicatorError, Result, ema::ema};
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

    // Calculate fast and slow EMAs
    let fast_ema = ema(data, fast_period);
    let slow_ema = ema(data, slow_period);

    // Calculate MACD line (fast - slow)
    let mut macd_line = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        match (fast_ema[i], slow_ema[i]) {
            (Some(fast), Some(slow)) => macd_line.push(Some(fast - slow)),
            _ => macd_line.push(None),
        }
    }

    // Extract non-None MACD values for signal line calculation
    let macd_values: Vec<f64> = macd_line.iter().filter_map(|&v| v).collect();

    // Calculate signal line (EMA of MACD line)
    let signal_ema = ema(&macd_values, signal_period);

    // Map signal EMA back to full length vector
    let mut signal_line = vec![None; data.len()];
    let mut signal_idx = 0;
    for i in 0..data.len() {
        if macd_line[i].is_some() {
            signal_line[i] = signal_ema.get(signal_idx).copied().flatten();
            signal_idx += 1;
        }
    }

    // Calculate histogram (MACD - Signal)
    let mut histogram = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        match (macd_line[i], signal_line[i]) {
            (Some(macd), Some(signal)) => histogram.push(Some(macd - signal)),
            _ => histogram.push(None),
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
