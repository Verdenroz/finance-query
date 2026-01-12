//! Stochastic Oscillator indicator.

use super::{IndicatorError, Result, sma::sma};
use serde::{Deserialize, Serialize};

/// Result of Stochastic Oscillator calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StochasticResult {
    /// %K line
    pub k: Vec<Option<f64>>,
    /// %D line (Signal line)
    pub d: Vec<Option<f64>>,
}

/// Calculate Stochastic Oscillator.
///
/// Returns (%K, %D) where:
/// %K = (Close - Lowest Low) / (Highest High - Lowest Low) * 100
/// %D = SMA of %K
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `k_period` - Period for %K
/// * `d_period` - Period for %D (SMA of %K)
///
/// # Example
///
/// ```
/// use finance_query::indicators::stochastic;
///
/// let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let lows = vec![8.0, 9.0, 10.0, 11.0, 12.0];
/// let closes = vec![9.0, 10.0, 11.0, 12.0, 13.0];
/// let result = stochastic(&highs, &lows, &closes, 3, 2).unwrap();
/// ```
pub fn stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    d_period: usize,
) -> Result<StochasticResult> {
    if k_period == 0 || d_period == 0 {
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

    let mut k_values = vec![None; len];
    let mut k_series_for_sma = vec![0.0; len];

    // Calculate %K values
    for i in (k_period - 1)..len {
        let start_idx = i + 1 - k_period;
        let end_idx = i;

        let period_highs = &highs[start_idx..=end_idx];
        let period_lows = &lows[start_idx..=end_idx];

        let highest = period_highs
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = period_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        let range = highest - lowest;
        let k = if range == 0.0 {
            50.0 // Neutral when no range
        } else {
            ((closes[end_idx] - lowest) / range) * 100.0
        };

        k_values[i] = Some(k);
        k_series_for_sma[i] = k;
    }

    // Calculate %D (SMA of %K)
    let valid_k_start = k_period - 1;
    if len <= valid_k_start {
        return Ok(StochasticResult {
            k: k_values,
            d: vec![None; len],
        });
    }

    let valid_k_slice = &k_series_for_sma[valid_k_start..];

    let d_values_valid = sma(valid_k_slice, d_period);

    let mut d_values = vec![None; len];
    for (j, val) in d_values_valid.into_iter().enumerate() {
        let original_idx = j + valid_k_start;
        if original_idx < len {
            d_values[original_idx] = val;
        }
    }

    Ok(StochasticResult {
        k: k_values,
        d: d_values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic() {
        let highs = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let lows = vec![8.0, 9.0, 10.0, 11.0, 12.0];
        let closes = vec![9.0, 10.0, 11.0, 12.0, 13.0];
        let result = stochastic(&highs, &lows, &closes, 3, 2).unwrap();

        assert_eq!(result.k.len(), 5);
        assert_eq!(result.d.len(), 5);

        // k valid from index 2
        assert!(result.k[0].is_none());
        assert!(result.k[1].is_none());
        assert!(result.k[2].is_some());

        // d valid from index 2 + (2-1) = 3
        assert!(result.d[0].is_none());
        assert!(result.d[1].is_none());
        assert!(result.d[2].is_none());
        assert!(result.d[3].is_some());
    }
}
