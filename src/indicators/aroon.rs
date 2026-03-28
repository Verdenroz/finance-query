//! Aroon indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Result of Aroon calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AroonResult {
    /// Aroon Up line
    pub aroon_up: Vec<Option<f64>>,
    /// Aroon Down line
    pub aroon_down: Vec<Option<f64>>,
}

/// Calculate Aroon Indicator.
///
/// Aroon Up = ((period - periods since highest high) / period) * 100
/// Aroon Down = ((period - periods since lowest low) / period) * 100
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::aroon;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let result = aroon(&highs, &lows, 3).unwrap();
/// ```
pub fn aroon(highs: &[f64], lows: &[f64], period: usize) -> Result<AroonResult> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len {
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

    let mut aroon_up = vec![None; len];
    let mut aroon_down = vec![None; len];

    // Monotonic deques for O(N) sliding window argmax/argmin instead of O(N * period)
    // Pop back on <= (highs) / >= (lows) so newest index wins on ties, matching original.
    let mut max_deque: VecDeque<usize> = VecDeque::new();
    let mut min_deque: VecDeque<usize> = VecDeque::new();
    let period_f = period as f64;

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
            let high_idx = *max_deque.front().unwrap();
            let low_idx = *min_deque.front().unwrap();
            let periods_since_high = i - high_idx;
            let periods_since_low = i - low_idx;
            aroon_up[i] = Some(((period_f - periods_since_high as f64) / period_f) * 100.0);
            aroon_down[i] = Some(((period_f - periods_since_low as f64) / period_f) * 100.0);
        }
    }

    Ok(AroonResult {
        aroon_up,
        aroon_down,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aroon() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let result = aroon(&highs, &lows, 3).unwrap();

        assert_eq!(result.aroon_up.len(), 5);
        assert!(result.aroon_up[0].is_none());
        assert!(result.aroon_up[1].is_none());
        assert!(result.aroon_up[2].is_some());
    }
}
