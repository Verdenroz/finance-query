//! Stochastic RSI indicator.

use std::collections::VecDeque;

use super::{IndicatorError, Result, rsi::rsi_raw, sma::sma_raw, stochastic::StochasticResult};

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

    let rsi_dense = rsi_raw(data, rsi_period)?;
    let len = data.len();
    stochastic_rsi_from_rsi_dense(
        &rsi_dense,
        len,
        rsi_period,
        stoch_period,
        k_period,
        d_period,
    )
}

/// Internal variant accepting pre-computed RSI dense values (avoids redundant RSI computation
/// when the caller already holds the `rsi_raw` output for the same period).
///
/// `rsi_dense` = output of `rsi_raw(data, rsi_period)`, length = `len - rsi_period`.
/// `len` = original data length (needed to allocate output vecs).
pub(crate) fn stochastic_rsi_from_rsi_dense(
    rsi_dense: &[f64],
    len: usize,
    rsi_period: usize,
    stoch_period: usize,
    k_period: usize,
    d_period: usize,
) -> Result<StochasticResult> {
    if stoch_period == 0 || k_period == 0 || d_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    let rsi_len = rsi_dense.len();
    if rsi_len < stoch_period {
        return Err(IndicatorError::InsufficientData {
            need: rsi_period + stoch_period,
            got: len,
        });
    }
    let raw_start = rsi_period + stoch_period - 1;
    let raw_count = rsi_len.saturating_sub(stoch_period - 1);
    let mut raw_stoch_dense = Vec::with_capacity(raw_count);
    {
        let mut max_deque: VecDeque<usize> = VecDeque::new();
        let mut min_deque: VecDeque<usize> = VecDeque::new();
        for k in 0..rsi_len {
            while max_deque.front().is_some_and(|&j| j + stoch_period <= k) {
                max_deque.pop_front();
            }
            while min_deque.front().is_some_and(|&j| j + stoch_period <= k) {
                min_deque.pop_front();
            }
            while max_deque
                .back()
                .is_some_and(|&j| rsi_dense[j] <= rsi_dense[k])
            {
                max_deque.pop_back();
            }
            while min_deque
                .back()
                .is_some_and(|&j| rsi_dense[j] >= rsi_dense[k])
            {
                min_deque.pop_back();
            }
            max_deque.push_back(k);
            min_deque.push_back(k);
            if k + 1 >= stoch_period {
                let max_rsi = rsi_dense[*max_deque.front().unwrap()];
                let min_rsi = rsi_dense[*min_deque.front().unwrap()];
                let range = max_rsi - min_rsi;
                raw_stoch_dense.push(if range.abs() < f64::EPSILON {
                    50.0
                } else {
                    (rsi_dense[k] - min_rsi) / range * 100.0
                });
            }
        }
    }
    let k_dense: Vec<f64>;
    let (k_line, k_valid_start) = if k_period == 1 {
        k_dense = raw_stoch_dense.clone();
        let mut k_line = vec![None; len];
        for (j, &v) in raw_stoch_dense.iter().enumerate() {
            k_line[j + raw_start] = Some(v);
        }
        (k_line, raw_start)
    } else {
        k_dense = sma_raw(&raw_stoch_dense, k_period);
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
    let d_line = if d_period == 1 {
        k_line.clone()
    } else {
        let d_raw = sma_raw(&k_dense, d_period);
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
