//! Fibonacci Retracement indicator.
//!
//! Computes the standard Fibonacci retracement levels between the swing
//! high and swing low of a rolling lookback window.

use std::collections::VecDeque;

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Fibonacci retracement levels between a swing high and swing low.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FibonacciLevels {
    /// Swing high over the lookback window (0% retracement level)
    pub swing_high: f64,
    /// Swing low over the lookback window (100% retracement level)
    pub swing_low: f64,
    /// 23.6% retracement level
    pub level_23_6: f64,
    /// 38.2% retracement level
    pub level_38_2: f64,
    /// 50% retracement level
    pub level_50: f64,
    /// 61.8% retracement level
    pub level_61_8: f64,
    /// 78.6% retracement level
    pub level_78_6: f64,
}

/// Calculate rolling Fibonacci retracement levels.
///
/// For each bar, the swing high/low are the highest high and lowest low over
/// the trailing `period` bars (inclusive), and the standard retracement
/// levels are interpolated between them (0% = swing high, 100% = swing low —
/// the conventional orientation for retracing a preceding up-move).
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `period` - Lookback window size (typically 50–100)
///
/// # Example
///
/// ```
/// use finance_query::indicators::fibonacci_retracement;
///
/// let highs = vec![10.0, 12.0, 11.0, 9.0, 13.0];
/// let lows = vec![8.0, 9.0, 8.5, 7.0, 10.0];
/// let result = fibonacci_retracement(&highs, &lows, 3).unwrap();
///
/// assert!(result[0].is_none());
/// assert!(result[2].is_some());
/// ```
pub fn fibonacci_retracement(
    highs: &[f64],
    lows: &[f64],
    period: usize,
) -> Result<Vec<Option<FibonacciLevels>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "highs and lows must have the same length".to_string(),
        ));
    }
    if len < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: len,
        });
    }

    let mut result = vec![None; len];

    // Monotonic deques for O(N) sliding window max/min (same technique as donchian_channels).
    let mut max_deque: VecDeque<usize> = VecDeque::new();
    let mut min_deque: VecDeque<usize> = VecDeque::new();

    for i in 0..len {
        while max_deque.front().is_some_and(|&j| j + period <= i) {
            max_deque.pop_front();
        }
        while min_deque.front().is_some_and(|&j| j + period <= i) {
            min_deque.pop_front();
        }
        while max_deque.back().is_some_and(|&j| highs[j] <= highs[i]) {
            max_deque.pop_back();
        }
        while min_deque.back().is_some_and(|&j| lows[j] >= lows[i]) {
            min_deque.pop_back();
        }
        max_deque.push_back(i);
        min_deque.push_back(i);

        if i + 1 >= period {
            let swing_high = highs[*max_deque.front().unwrap()];
            let swing_low = lows[*min_deque.front().unwrap()];
            let range = swing_high - swing_low;
            result[i] = Some(FibonacciLevels {
                swing_high,
                swing_low,
                level_23_6: swing_high - range * 0.236,
                level_38_2: swing_high - range * 0.382,
                level_50: swing_high - range * 0.5,
                level_61_8: swing_high - range * 0.618,
                level_78_6: swing_high - range * 0.786,
            });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_retracement_basic() {
        let highs = vec![10.0, 12.0, 11.0, 9.0, 13.0];
        let lows = vec![8.0, 9.0, 8.5, 7.0, 10.0];
        let result = fibonacci_retracement(&highs, &lows, 3).unwrap();

        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());

        // Window [10,12,11]/[8,9,8.5] -> swing_high=12, swing_low=8, range=4
        let l = result[2].unwrap();
        assert!((l.swing_high - 12.0).abs() < 1e-9);
        assert!((l.swing_low - 8.0).abs() < 1e-9);
        assert!((l.level_50 - 10.0).abs() < 1e-9);
        assert!((l.level_23_6 - (12.0 - 4.0 * 0.236)).abs() < 1e-9);
        assert!((l.level_61_8 - (12.0 - 4.0 * 0.618)).abs() < 1e-9);
    }

    #[test]
    fn test_fibonacci_levels_ordering() {
        let highs = vec![10.0, 12.0, 11.0, 9.0, 13.0];
        let lows = vec![8.0, 9.0, 8.5, 7.0, 10.0];
        let result = fibonacci_retracement(&highs, &lows, 3).unwrap();

        for l in result.into_iter().flatten() {
            assert!(l.swing_high >= l.level_23_6);
            assert!(l.level_23_6 >= l.level_38_2);
            assert!(l.level_38_2 >= l.level_50);
            assert!(l.level_50 >= l.level_61_8);
            assert!(l.level_61_8 >= l.level_78_6);
            assert!(l.level_78_6 >= l.swing_low);
        }
    }

    #[test]
    fn test_fibonacci_retracement_insufficient_data() {
        assert!(fibonacci_retracement(&[1.0, 2.0], &[1.0, 2.0], 5).is_err());
    }

    #[test]
    fn test_fibonacci_retracement_invalid_period() {
        assert!(fibonacci_retracement(&[1.0], &[1.0], 0).is_err());
    }

    #[test]
    fn test_fibonacci_retracement_mismatched_lengths() {
        assert!(fibonacci_retracement(&[1.0, 2.0], &[1.0], 1).is_err());
    }
}
