//! Stochastic Oscillator indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result, sma::sma_raw};
use serde::{Deserialize, Serialize};

/// Result of Stochastic Oscillator calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StochasticResult {
    /// %K line (optionally slow-smoothed)
    pub k: Vec<Option<f64>>,
    /// %D line (Signal line — SMA of %K)
    pub d: Vec<Option<f64>>,
}

/// Calculate Stochastic Oscillator.
///
/// Returns (%K, %D) where:
/// - Raw %K = (Close − Lowest Low) / (Highest High − Lowest Low) × 100
/// - Slow %K = SMA(Raw %K, k_slow) — set `k_slow = 1` for no smoothing
/// - %D = SMA(Slow %K, d_period)
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `k_period` - Lookback period for raw %K (number of bars)
/// * `k_slow` - Smoothing period applied to raw %K before computing %D; `1` = no smoothing
/// * `d_period` - Period for %D signal line (SMA of slow %K)
///
/// # Example
///
/// ```
/// use finance_query::indicators::stochastic;
///
/// let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let lows = vec![8.0, 9.0, 10.0, 11.0, 12.0];
/// let closes = vec![9.0, 10.0, 11.0, 12.0, 13.0];
/// let result = stochastic(&highs, &lows, &closes, 3, 1, 2).unwrap();
/// ```
pub fn stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    k_slow: usize,
    d_period: usize,
) -> Result<StochasticResult> {
    if k_period == 0 || k_slow == 0 || d_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < k_period {
        return Err(IndicatorError::InsufficientData {
            need: k_period,
            got: len,
        });
    }

    // Step 1: compute raw %K using monotonic deques — O(N) instead of O(N * k_period)
    let mut raw_k = vec![None; len];
    let mut raw_k_for_sma = vec![0.0; len];
    {
        let mut max_deque: VecDeque<usize> = VecDeque::new(); // tracks highest high
        let mut min_deque: VecDeque<usize> = VecDeque::new(); // tracks lowest low

        for i in 0..len {
            // Evict indices that have fallen outside the k_period window
            while max_deque.front().is_some_and(|&j| j + k_period <= i) {
                max_deque.pop_front();
            }
            while min_deque.front().is_some_and(|&j| j + k_period <= i) {
                min_deque.pop_front();
            }
            // Maintain decreasing monotone for max(highs)
            while max_deque.back().is_some_and(|&j| highs[j] <= highs[i]) {
                max_deque.pop_back();
            }
            // Maintain increasing monotone for min(lows)
            while min_deque.back().is_some_and(|&j| lows[j] >= lows[i]) {
                min_deque.pop_back();
            }
            max_deque.push_back(i);
            min_deque.push_back(i);

            if i + 1 >= k_period {
                let highest = highs[*max_deque.front().unwrap()];
                let lowest = lows[*min_deque.front().unwrap()];
                let k = if (highest - lowest).abs() < f64::EPSILON {
                    50.0 // Neutral when no range
                } else {
                    ((closes[i] - lowest) / (highest - lowest)) * 100.0
                };
                raw_k[i] = Some(k);
                raw_k_for_sma[i] = k;
            }
        }
    }

    // Step 2: apply k_slow smoothing to raw %K
    // slow_dense: dense f64 slow-K values starting at slow_k_valid_start (used for D smoothing)
    let raw_k_valid_start = k_period - 1;
    let slow_dense: Vec<f64>;
    let (slow_k, slow_k_valid_start) = if k_slow == 1 {
        slow_dense = raw_k_for_sma[raw_k_valid_start..].to_vec();
        (raw_k.clone(), raw_k_valid_start)
    } else {
        let raw_k_slice = &raw_k_for_sma[raw_k_valid_start..];
        slow_dense = sma_raw(raw_k_slice, k_slow); // Vec<f64>, avoids Vec<Option<f64>>
        let slow_valid_start = raw_k_valid_start + k_slow - 1;

        let mut slow_k = vec![None; len];
        for (j, &val) in slow_dense.iter().enumerate() {
            let idx = j + slow_valid_start;
            if idx < len {
                slow_k[idx] = Some(val);
            }
        }
        (slow_k, slow_valid_start)
    };

    // Step 3: %D = SMA of slow_dense — eliminates slow_k_values extraction allocation
    let d_raw = sma_raw(&slow_dense, d_period);
    let d_off = slow_k_valid_start + d_period - 1;
    let mut d_values = vec![None; len];
    for (j, &val) in d_raw.iter().enumerate() {
        let idx = j + d_off;
        if idx < len {
            d_values[idx] = Some(val);
        }
    }

    Ok(StochasticResult {
        k: slow_k,
        d: d_values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_no_k_slow() {
        let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let lows = vec![8.0, 9.0, 10.0, 11.0, 12.0];
        let closes = vec![9.0, 10.0, 11.0, 12.0, 13.0];
        let result = stochastic(&highs, &lows, &closes, 3, 1, 2).unwrap();

        assert_eq!(result.k.len(), 5);
        assert_eq!(result.d.len(), 5);

        // raw %K valid from index 2
        assert!(result.k[0].is_none());
        assert!(result.k[1].is_none());
        assert!(result.k[2].is_some());

        // %D valid from index 2 + (2-1) = 3 (k_slow=1 means no additional delay)
        assert!(result.d[0].is_none());
        assert!(result.d[1].is_none());
        assert!(result.d[2].is_none());
        assert!(result.d[3].is_some());
    }

    #[test]
    fn test_stochastic_with_k_slow() {
        let highs = vec![10.0; 10];
        let lows = vec![8.0; 10];
        let closes = vec![9.0; 10];
        // k_period=3, k_slow=3, d_period=3: slow k valid from idx 4, d from idx 6
        let result = stochastic(&highs, &lows, &closes, 3, 3, 3).unwrap();
        // raw k valid from 2; slow k starts 2+2=4; d starts 4+2=6
        assert!(result.k[3].is_none());
        assert!(result.k[4].is_some());
        assert!(result.d[5].is_none());
        assert!(result.d[6].is_some());
    }

    #[test]
    fn test_stochastic_k_slow_produces_different_k_than_no_slow() {
        // Alternating high/low closes make raw %K oscillate, so SMA smoothing produces
        // a noticeably different value than the unsmoothed raw %K.
        let closes: Vec<f64> = (0..20)
            .map(|i| if i % 2 == 0 { 10.0 } else { 20.0 })
            .collect();
        let highs: Vec<f64> = closes.iter().map(|&c| c + 0.5).collect();
        let lows: Vec<f64> = closes.iter().map(|&c| c - 0.5).collect();

        // fast: no k_slow smoothing — reads raw %K at each bar
        let fast = stochastic(&highs, &lows, &closes, 5, 1, 3).unwrap();
        // slow: SMA(3) over raw %K — averages three oscillating values
        let slow = stochastic(&highs, &lows, &closes, 5, 3, 3).unwrap();

        // Both must be valid at index 10; slow starts at 4 + (3-1) = 6
        let idx = 10;
        assert!(fast.k[idx].is_some());
        assert!(slow.k[idx].is_some());
        // raw %K oscillates ~4.5 / ~95.5; SMA-3 of those three values ≈ 34.8 ≠ raw value
        assert_ne!(fast.k[idx], slow.k[idx]);
    }
}
