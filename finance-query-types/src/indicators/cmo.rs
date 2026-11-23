//! Chande Momentum Oscillator (CMO) indicator.

use super::{IndicatorError, Result};

/// Calculate Chande Momentum Oscillator (CMO).
///
/// Similar to RSI but uses sum of gains - sum of losses.
/// Returns value between -100 and 100.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::cmo;
///
/// let prices = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let result = cmo(&prices, 3).unwrap();
/// ```
pub fn cmo(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period + 1 {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: data.len(),
        });
    }

    let mut result = vec![None; data.len()];

    // Pre-calculate changes
    let mut changes = Vec::with_capacity(data.len());
    changes.push(0.0); // Dummy for index 0
    for i in 1..data.len() {
        changes.push(data[i] - data[i - 1]);
    }

    // Calculate initial window
    let mut gains_sum = 0.0;
    let mut losses_sum = 0.0;

    for &change in changes.iter().skip(1).take(period) {
        if change > 0.0 {
            gains_sum += change;
        } else {
            losses_sum += -change;
        }
    }

    // First value at index period
    let total = gains_sum + losses_sum;
    if total != 0.0 {
        result[period] = Some(((gains_sum - losses_sum) / total) * 100.0);
    } else {
        result[period] = Some(0.0);
    }

    // Slide window
    for i in (period + 1)..data.len() {
        // Remove old change (at index i - period)
        let old_change = changes[i - period];
        if old_change > 0.0 {
            gains_sum -= old_change;
        } else {
            losses_sum -= -old_change;
        }

        // Add new change (at index i)
        let new_change = changes[i];
        if new_change > 0.0 {
            gains_sum += new_change;
        } else {
            losses_sum += -new_change;
        }

        let total = gains_sum + losses_sum;
        if total != 0.0 {
            result[i] = Some(((gains_sum - losses_sum) / total) * 100.0);
        } else {
            result[i] = Some(0.0);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmo() {
        let prices = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let result = cmo(&prices, 3).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_none());

        // i=3: changes: 1, 1, -1. gains=2, losses=1. total=3. cmo=(1/3)*100 = 33.33
        assert!(result[3].is_some());
        let val = result[3].unwrap();
        assert!((val - 33.333).abs() < 0.01);
    }
}
