//! Williams %R indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result};

/// Calculate Williams %R.
///
/// Similar to Stochastic but inverted scale.
/// %R = (Highest High - Close) / (Highest High - Lowest Low) * -100
/// Returns value between -100 and 0.
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
/// use finance_query::indicators::williams_r;
///
/// let highs = vec![10.0, 11.0, 12.0, 13.0];
/// let lows = vec![8.0, 9.0, 10.0, 11.0];
/// let closes = vec![9.0, 10.0, 11.0, 12.0];
/// let result = williams_r(&highs, &lows, &closes, 3).unwrap();
/// ```
pub fn williams_r(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Result<Vec<Option<f64>>> {
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

    let mut result = vec![None; len];

    // Monotonic deques for O(N) sliding window max/min instead of O(N * period)
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
            let highest = highs[*max_deque.front().unwrap()];
            let lowest = lows[*min_deque.front().unwrap()];
            let range = highest - lowest;
            result[i] = Some(if range == 0.0 {
                -50.0
            } else {
                ((highest - closes[i]) / range) * -100.0
            });
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_williams_r() {
        let highs = vec![10.0, 11.0, 12.0, 13.0];
        let lows = vec![8.0, 9.0, 10.0, 11.0];
        let closes = vec![9.0, 10.0, 11.0, 12.0];
        let result = williams_r(&highs, &lows, &closes, 3).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());
    }
}
