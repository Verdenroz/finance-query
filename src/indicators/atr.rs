//! Average True Range (ATR) indicator.

use super::{IndicatorError, Result};

/// Internal ATR returning only valid values as plain `f64` (no `Option` wrapping, no padding).
/// Length = `highs.len() - (period - 1)`. Index `k` corresponds to original index `k + period - 1`.
pub(crate) fn atr_raw(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Result<Vec<f64>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if highs.len() != lows.len() || highs.len() != closes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "All arrays must have the same length".to_string(),
        ));
    }
    if highs.len() <= period {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: highs.len(),
        });
    }
    let len = highs.len();
    let period_m1 = (period - 1) as f64;
    let period_f = period as f64;
    let mut tr_sum = highs[0] - lows[0];
    for i in 1..period {
        let h_l = highs[i] - lows[i];
        let h_pc = (highs[i] - closes[i - 1]).abs();
        let l_pc = (lows[i] - closes[i - 1]).abs();
        tr_sum += h_l.max(h_pc).max(l_pc);
    }
    let mut prev_atr = tr_sum / period_f;
    let mut result = Vec::with_capacity(len - period + 1);
    result.push(prev_atr);
    for i in period..len {
        let h_l = highs[i] - lows[i];
        let h_pc = (highs[i] - closes[i - 1]).abs();
        let l_pc = (lows[i] - closes[i - 1]).abs();
        let tr = h_l.max(h_pc).max(l_pc);
        prev_atr = (prev_atr * period_m1 + tr) / period_f;
        result.push(prev_atr);
    }
    Ok(result)
}

/// Calculate Average True Range (ATR).
///
/// ATR measures market volatility by calculating the average of true ranges over a period.
/// True range is the greatest of:
/// - Current high - Current low
/// - |Current high - Previous close|
/// - |Current low - Previous close|
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - Number of periods (typically 14)
///
/// # Returns
///
/// Vector of ATR values. First `period` values will be None.
///
/// # Example
///
/// ```
/// use finance_query::indicators::atr;
///
/// let highs = vec![50.0, 51.0, 52.0, 51.5, 53.0, 54.0, 53.5, 55.0];
/// let lows = vec![48.0, 49.0, 50.0, 49.5, 51.0, 52.0, 51.5, 53.0];
/// let closes = vec![49.0, 50.5, 51.0, 50.0, 52.0, 53.0, 52.5, 54.0];
///
/// let result = atr(&highs, &lows, &closes, 3).unwrap();
/// assert_eq!(result.len(), 8);
/// assert!(result[2].is_some()); // ATR available after period
/// ```
pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    let raw = atr_raw(highs, lows, closes, period)?;
    let len = highs.len();
    let mut result = vec![None; len];
    for (k, v) in raw.into_iter().enumerate() {
        result[k + period - 1] = Some(v);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr_basic() {
        let highs = vec![50.0, 51.0, 52.0, 51.5, 53.0, 54.0];
        let lows = vec![48.0, 49.0, 50.0, 49.5, 51.0, 52.0];
        let closes = vec![49.0, 50.5, 51.0, 50.0, 52.0, 53.0];

        let result = atr(&highs, &lows, &closes, 3).unwrap();

        assert_eq!(result.len(), 6);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());

        // ATR should be positive
        for val in result.iter().flatten() {
            assert!(val > &0.0);
        }
    }

    #[test]
    fn test_atr_insufficient_data() {
        let highs = vec![50.0, 51.0];
        let lows = vec![48.0, 49.0];
        let closes = vec![49.0, 50.0];

        let result = atr(&highs, &lows, &closes, 14);
        assert!(result.is_err());
    }

    #[test]
    fn test_atr_mismatched_lengths() {
        let highs = vec![50.0, 51.0, 52.0];
        let lows = vec![48.0, 49.0];
        let closes = vec![49.0, 50.0, 51.0];

        let result = atr(&highs, &lows, &closes, 3);
        assert!(result.is_err());
    }
}
