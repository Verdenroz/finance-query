//! Choppiness Index indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result};

/// Calculate Choppiness Index.
///
/// Measures market choppiness (consolidation vs trending).
/// Returns value between 0-100.
/// Above 61.8 = choppy/consolidating
/// Below 38.2 = trending
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
/// use finance_query::indicators::choppiness_index;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = choppiness_index(&highs, &lows, &closes, 14).unwrap();
/// ```
pub fn choppiness_index(
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
    if len < period + 1 {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: len,
        });
    }

    let mut result = vec![None; len];

    // Circular buffer of size `period` replaces the full tr_values Vec
    // tr_circ[i % period] holds TR[i-period] just before i's iteration overwrites it
    let mut tr_circ = vec![0.0f64; period];
    let mut max_deque: VecDeque<usize> = VecDeque::new();
    let mut min_deque: VecDeque<usize> = VecDeque::new();

    // Seed: TR[0] uses high-low only (no previous close); pre-fill deques for window [0, period)
    let first_tr = highs[0] - lows[0];
    tr_circ[0] = first_tr;
    let mut tr_window_sum = first_tr;
    max_deque.push_back(0);
    min_deque.push_back(0);

    for j in 1..period {
        let h_l = highs[j] - lows[j];
        let h_pc = (highs[j] - closes[j - 1]).abs();
        let l_pc = (lows[j] - closes[j - 1]).abs();
        let tr = h_l.max(h_pc).max(l_pc);
        tr_circ[j] = tr;
        tr_window_sum += tr;
        while max_deque.back().is_some_and(|&k| highs[k] <= highs[j]) {
            max_deque.pop_back();
        }
        while min_deque.back().is_some_and(|&k| lows[k] >= lows[j]) {
            min_deque.pop_back();
        }
        max_deque.push_back(j);
        min_deque.push_back(j);
    }

    // Precompute 100 / ln(period) to replace per-iteration division with multiplication
    let scale = 100.0 / (period as f64).ln();

    for i in period..len {
        let buf_pos = i % period;
        let old_tr = tr_circ[buf_pos]; // TR[i - period] being evicted

        let h_l = highs[i] - lows[i];
        let h_pc = (highs[i] - closes[i - 1]).abs();
        let l_pc = (lows[i] - closes[i - 1]).abs();
        let new_tr = h_l.max(h_pc).max(l_pc);
        tr_circ[buf_pos] = new_tr;
        tr_window_sum += new_tr - old_tr;

        let start_idx = i + 1 - period;
        while max_deque.front().is_some_and(|&j| j < start_idx) {
            max_deque.pop_front();
        }
        while min_deque.front().is_some_and(|&j| j < start_idx) {
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

        let highest = highs[*max_deque.front().unwrap()];
        let lowest = lows[*min_deque.front().unwrap()];
        let range = highest - lowest;

        result[i] = Some(if range == 0.0 || tr_window_sum == 0.0 {
            50.0
        } else {
            scale * (tr_window_sum / range).ln()
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choppiness_index() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = choppiness_index(&highs, &lows, &closes, 14).unwrap();

        assert_eq!(result.len(), 20);
        assert!(result[13].is_none());
        assert!(result[14].is_some());
    }
}
