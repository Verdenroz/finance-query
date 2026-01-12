//! Williams %R indicator.

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

    for (i, item) in result.iter_mut().enumerate().skip(period - 1) {
        let start_idx = i + 1 - period;
        let end_idx = i;

        let period_highs = &highs[start_idx..=end_idx];
        let period_lows = &lows[start_idx..=end_idx];
        let close = closes[end_idx];

        let highest = period_highs
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = period_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        let range = highest - lowest;
        if range == 0.0 {
            *item = Some(-50.0); // Neutral
        } else {
            let wr = ((highest - close) / range) * -100.0;
            *item = Some(wr);
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
