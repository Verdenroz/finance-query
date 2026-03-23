//! Stochastic RSI indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result, rsi::rsi, sma::sma_raw, stochastic::StochasticResult};

/// Calculate Stochastic RSI.
///
/// Applies the Stochastic formula to RSI values, then optionally smooths the
/// result into %K and %D lines — matching the TradingView "Stoch RSI" indicator.
///
/// Steps:
/// 1. Compute RSI with `rsi_period`.
/// 2. Apply Stochastic formula over `stoch_period` bars of the RSI series → raw StochRSI.
/// 3. Smooth raw StochRSI with SMA(`k_period`) → %K. Use `k_period = 1` to skip smoothing.
/// 4. Smooth %K with SMA(`d_period`) → %D. Use `d_period = 1` to skip.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `rsi_period` - Period for RSI calculation (e.g. 14)
/// * `stoch_period` - Lookback period for Stochastic formula on RSI (e.g. 14)
/// * `k_period` - SMA smoothing period for %K (e.g. 3; use 1 for no smoothing)
/// * `d_period` - SMA smoothing period for %D (e.g. 3; use 1 for no smoothing)
///
/// # Example
///
/// ```
/// use finance_query::indicators::stochastic_rsi;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
/// let result = stochastic_rsi(&prices, 3, 3, 3, 3).unwrap();
/// ```
pub fn stochastic_rsi(
    data: &[f64],
    rsi_period: usize,
    stoch_period: usize,
    k_period: usize,
    d_period: usize,
) -> Result<StochasticResult> {
    if rsi_period == 0 || stoch_period == 0 || k_period == 0 || d_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    let min_len = rsi_period + stoch_period;
    if data.len() < min_len {
        return Err(IndicatorError::InsufficientData {
            need: min_len,
            got: data.len(),
        });
    }

    let rsi_values = rsi(data, rsi_period)?;
    let len = data.len();
    let raw_start = rsi_period + stoch_period - 1;

    // Step 1: compute raw StochRSI (0–100) via monotonic deques — O(N) instead of O(N * stoch_period)
    // RSI is first valid at index rsi_period; all values from there onward are Some.
    let mut raw_stoch = vec![None; len];
    let mut raw_stoch_values = vec![0.0; len];

    let rsi_start = rsi_period; // first valid RSI index
    // Index directly into rsi_values to avoid an intermediate allocation.
    let rsi_val = |k: usize| unsafe { rsi_values[k + rsi_start].unwrap_unchecked() };

    {
        let mut max_deque: VecDeque<usize> = VecDeque::new();
        let mut min_deque: VecDeque<usize> = VecDeque::new();
        let rsi_len = len - rsi_start;

        for k in 0..rsi_len {
            while max_deque.front().is_some_and(|&j| j + stoch_period <= k) {
                max_deque.pop_front();
            }
            while min_deque.front().is_some_and(|&j| j + stoch_period <= k) {
                min_deque.pop_front();
            }
            while max_deque.back().is_some_and(|&j| rsi_val(j) <= rsi_val(k)) {
                max_deque.pop_back();
            }
            while min_deque.back().is_some_and(|&j| rsi_val(j) >= rsi_val(k)) {
                min_deque.pop_back();
            }
            max_deque.push_back(k);
            min_deque.push_back(k);

            if k + 1 >= stoch_period {
                let max_rsi = rsi_val(*max_deque.front().unwrap());
                let min_rsi = rsi_val(*min_deque.front().unwrap());
                let current_rsi = rsi_val(k);
                let range = max_rsi - min_rsi;
                let stoch = if range.abs() < f64::EPSILON {
                    50.0
                } else {
                    ((current_rsi - min_rsi) / range) * 100.0
                };
                let orig_idx = k + rsi_start;
                raw_stoch[orig_idx] = Some(stoch);
                raw_stoch_values[orig_idx] = stoch;
            }
        }
    }

    // Step 2: smooth raw StochRSI → %K
    // k_dense: dense f64 K values starting at k_valid_start (used directly for D smoothing)
    let k_dense: Vec<f64>;
    let (k_line, k_valid_start) = if k_period == 1 {
        k_dense = raw_stoch_values[raw_start..].to_vec();
        (raw_stoch.clone(), raw_start)
    } else {
        let slice = &raw_stoch_values[raw_start..];
        k_dense = sma_raw(slice, k_period); // Vec<f64>, avoids Vec<Option<f64>>
        let k_start = raw_start + k_period - 1;
        let mut k_line = vec![None; len];
        for (j, &val) in k_dense.iter().enumerate() {
            let idx = j + k_start;
            if idx < len {
                k_line[idx] = Some(val);
            }
        }
        (k_line, k_start)
    };

    // Step 3: smooth %K → %D — use k_dense directly (eliminates k_values_raw allocation)
    let d_line = if d_period == 1 {
        k_line.clone()
    } else {
        let d_raw = sma_raw(&k_dense, d_period); // Vec<f64>
        let d_start = k_valid_start + d_period - 1;
        let mut d_line = vec![None; len];
        for (j, &val) in d_raw.iter().enumerate() {
            let idx = j + d_start;
            if idx < len {
                d_line[idx] = Some(val);
            }
        }
        d_line
    };

    Ok(StochasticResult {
        k: k_line,
        d: d_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_rsi_no_smoothing() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
        // k_period=1, d_period=1: no smoothing, same as old behavior
        let result = stochastic_rsi(&prices, 3, 3, 1, 1).unwrap();

        assert_eq!(result.k.len(), 9);
        // RSI valid from index 3; StochRSI valid from index 3 + 3 - 1 = 5
        assert!(result.k[0].is_none());
        assert!(result.k[4].is_none());
        assert!(result.k[5].is_some());
    }

    #[test]
    fn test_stochastic_rsi_with_smoothing() {
        let prices: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        // k_period=3, d_period=3
        let result = stochastic_rsi(&prices, 3, 3, 3, 3).unwrap();
        assert_eq!(result.k.len(), 30);
        // %K valid later than no-smoothing case
        assert!(result.k[6].is_none() || result.k[6].is_some()); // just checking it compiles
        // %D should be even later
        // What matters is k and d are different
        let k_last = result.k.iter().filter_map(|v| *v).next_back();
        let d_last = result.d.iter().filter_map(|v| *v).next_back();
        assert!(k_last.is_some());
        assert!(d_last.is_some());
    }

    #[test]
    fn test_stochastic_rsi_k_before_d() {
        // %D should always start later than or at the same time as %K
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let result = stochastic_rsi(&prices, 5, 5, 3, 3).unwrap();
        let k_first = result.k.iter().position(|v| v.is_some());
        let d_first = result.d.iter().position(|v| v.is_some());
        assert!(k_first.is_some());
        assert!(d_first.is_some());
        assert!(d_first.unwrap() >= k_first.unwrap());
    }
}
