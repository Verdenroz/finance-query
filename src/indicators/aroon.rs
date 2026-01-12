//! Aroon indicator.

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Result of Aroon calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AroonResult {
    /// Aroon Up line
    pub aroon_up: Vec<Option<f64>>,
    /// Aroon Down line
    pub aroon_down: Vec<Option<f64>>,
}

/// Calculate Aroon Indicator.
///
/// Aroon Up = ((period - periods since highest high) / period) * 100
/// Aroon Down = ((period - periods since lowest low) / period) * 100
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
/// use finance_query::indicators::aroon;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let result = aroon(&highs, &lows, 3).unwrap();
/// ```
pub fn aroon(highs: &[f64], lows: &[f64], period: usize) -> Result<AroonResult> {
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

    let mut aroon_up = vec![None; len];
    let mut aroon_down = vec![None; len];

    for i in (period - 1)..len {
        let start_idx = i + 1 - period;
        let slice_highs = &highs[start_idx..=i];
        let slice_lows = &lows[start_idx..=i];

        let mut highest_idx = 0;
        let mut highest_val = f64::NEG_INFINITY;

        for (j, &val) in slice_highs.iter().enumerate() {
            if val >= highest_val {
                highest_val = val;
                highest_idx = j;
            }
        }

        let periods_since_high = (period - 1) - highest_idx;
        let up = ((period as f64 - periods_since_high as f64) / period as f64) * 100.0;

        let mut lowest_idx = 0;
        let mut lowest_val = f64::INFINITY;

        for (j, &val) in slice_lows.iter().enumerate() {
            if val <= lowest_val {
                lowest_val = val;
                lowest_idx = j;
            }
        }

        let periods_since_low = (period - 1) - lowest_idx;
        let down = ((period as f64 - periods_since_low as f64) / period as f64) * 100.0;

        aroon_up[i] = Some(up);
        aroon_down[i] = Some(down);
    }

    Ok(AroonResult {
        aroon_up,
        aroon_down,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aroon() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let result = aroon(&highs, &lows, 3).unwrap();

        assert_eq!(result.aroon_up.len(), 5);
        assert!(result.aroon_up[0].is_none());
        assert!(result.aroon_up[1].is_none());
        assert!(result.aroon_up[2].is_some());
    }
}
