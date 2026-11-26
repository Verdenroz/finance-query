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

    let changes: Vec<f64> = data.windows(2).map(|w| w[1] - w[0]).collect();

    let mut gains_sum = 0.0;
    let mut losses_sum = 0.0;

    for &change in &changes[..period] {
        if change > 0.0 {
            gains_sum += change;
        } else {
            losses_sum += -change;
        }
    }
    result[period] = Some(chande_momentum(gains_sum, losses_sum));

    for ((out, &old_change), &new_change) in result[period + 1..]
        .iter_mut()
        .zip(&changes)
        .zip(&changes[period..])
    {
        if old_change > 0.0 {
            gains_sum -= old_change;
        } else {
            losses_sum -= -old_change;
        }

        if new_change > 0.0 {
            gains_sum += new_change;
        } else {
            losses_sum += -new_change;
        }

        *out = Some(chande_momentum(gains_sum, losses_sum));
    }

    Ok(result)
}

fn chande_momentum(gains_sum: f64, losses_sum: f64) -> f64 {
    let total = gains_sum + losses_sum;
    if total != 0.0 {
        ((gains_sum - losses_sum) / total) * 100.0
    } else {
        0.0
    }
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
