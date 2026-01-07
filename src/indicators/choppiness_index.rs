//! Choppiness Index indicator.

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

    let mut tr_values = Vec::with_capacity(len);
    tr_values.push(highs[0] - lows[0]);

    for i in 1..len {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        tr_values.push(tr);
    }

    for i in period..len {
        let start_idx = i + 1 - period;

        let tr_sum: f64 = tr_values[start_idx..=i].iter().sum();

        let slice_highs = &highs[start_idx..=i];
        let slice_lows = &lows[start_idx..=i];

        let highest = slice_highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = slice_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let range = highest - lowest;

        if range == 0.0 || tr_sum == 0.0 {
            result[i] = Some(50.0);
        } else {
            let ci = 100.0 * (tr_sum / range).ln() / (period as f64).ln();
            result[i] = Some(ci);
        }
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
