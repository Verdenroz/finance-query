//! Donchian Channels indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Result of Donchian Channels calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonchianChannelsResult {
    /// Upper channel
    pub upper: Vec<Option<f64>>,
    /// Middle channel
    pub middle: Vec<Option<f64>>,
    /// Lower channel
    pub lower: Vec<Option<f64>>,
}

/// Calculate Donchian Channels.
///
/// Upper Channel = Highest high over period
/// Lower Channel = Lowest low over period
/// Middle Channel = (Upper + Lower) / 2
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
/// use finance_query::indicators::donchian_channels;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let result = donchian_channels(&highs, &lows, 3).unwrap();
/// ```
pub fn donchian_channels(
    highs: &[f64],
    lows: &[f64],
    period: usize,
) -> Result<DonchianChannelsResult> {
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

    let mut upper = vec![None; len];
    let mut middle = vec![None; len];
    let mut lower = vec![None; len];

    // Use monotonic deques for O(N) sliding window max/min instead of O(N * period)
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
            upper[i] = Some(highest);
            lower[i] = Some(lowest);
            middle[i] = Some((highest + lowest) / 2.0);
        }
    }

    Ok(DonchianChannelsResult {
        upper,
        middle,
        lower,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_channels() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let result = donchian_channels(&highs, &lows, 3).unwrap();

        assert_eq!(result.upper.len(), 5);
        assert!(result.upper[0].is_none());
        assert!(result.upper[1].is_none());
        assert!(result.upper[2].is_some());
    }
}
