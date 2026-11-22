//! Commodity Channel Index (CCI) indicator.

use super::{IndicatorError, Result};

/// Calculate Commodity Channel Index (CCI).
///
/// CCI measures the variation of a security's price from its statistical mean.
/// Formula: CCI = (Typical Price - SMA of Typical Price) / (0.015 * Mean Deviation)
///
/// Typical Price = (High + Low + Close) / 3
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::cci;
///
/// let highs = vec![10.0, 11.0, 12.0, 13.0];
/// let lows = vec![8.0, 9.0, 10.0, 11.0];
/// let closes = vec![9.0, 10.0, 11.0, 12.0];
/// let result = cci(&highs, &lows, &closes, 3).unwrap();
/// ```
pub fn cci(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: len,
        });
    }

    let mut typical_prices = Vec::with_capacity(len);
    for i in 0..len {
        typical_prices.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    let mut result = vec![None; len];

    // Sliding window sum for SMA — O(N) instead of O(N * period) for the sum pass.
    // Mean deviation still requires O(period) per step (depends on current window mean).
    let period_f = period as f64;
    let mut window_sum: f64 = typical_prices[..period].iter().sum();

    for i in (period - 1)..len {
        if i > period - 1 {
            window_sum += typical_prices[i] - typical_prices[i - period];
        }
        let sma = window_sum / period_f;
        let start_idx = i + 1 - period;
        let slice = &typical_prices[start_idx..=i];
        let deviations_sum: f64 = slice.iter().map(|&tp| (tp - sma).abs()).sum();
        let mean_deviation = deviations_sum / period_f;

        result[i] = Some(if mean_deviation == 0.0 {
            0.0
        } else {
            (typical_prices[i] - sma) / (0.015 * mean_deviation)
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cci() {
        let highs = vec![10.0, 11.0, 12.0, 13.0];
        let lows = vec![8.0, 9.0, 10.0, 11.0];
        let closes = vec![9.0, 10.0, 11.0, 12.0];
        let result = cci(&highs, &lows, &closes, 3).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());
    }
}
