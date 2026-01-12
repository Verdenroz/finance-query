//! True Range indicator.

use super::{IndicatorError, Result};

/// Calculate True Range.
///
/// TR = max(high - low, |high - prev_close|, |low - prev_close|)
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
///
/// # Example
///
/// ```
/// use finance_query::indicators::true_range;
///
/// let highs = vec![10.0, 11.0, 12.0];
/// let lows = vec![8.0, 9.0, 10.0];
/// let closes = vec![9.0, 10.0, 11.0];
/// let result = true_range(&highs, &lows, &closes).unwrap();
/// ```
pub fn true_range(highs: &[f64], lows: &[f64], closes: &[f64]) -> Result<Vec<Option<f64>>> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 2 {
        return Err(IndicatorError::InsufficientData { need: 2, got: len });
    }

    let mut result = vec![None; len];

    result[0] = Some(highs[0] - lows[0]);

    for i in 1..len {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        result[i] = Some(tr);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_range() {
        let highs = vec![10.0, 11.0, 12.0];
        let lows = vec![8.0, 9.0, 10.0];
        let closes = vec![9.0, 10.0, 11.0];
        let result = true_range(&highs, &lows, &closes).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result[0].is_some());
        assert!(result[1].is_some());
        assert!(result[2].is_some());
    }
}
