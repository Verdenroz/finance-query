//! Rate of Change (ROC) indicator.

use super::{IndicatorError, Result};

/// Calculate Rate of Change (ROC).
///
/// Percentage change over period.
/// ROC = ((Price - Price[n periods ago]) / Price[n periods ago]) * 100
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::roc;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0];
/// let result = roc(&prices, 2).unwrap();
/// ```
pub fn roc(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() <= period {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: data.len(),
        });
    }

    let mut result = vec![None; data.len()];

    for i in period..data.len() {
        let current = data[i];
        let past = data[i - period];

        if past != 0.0 {
            let roc_val = ((current - past) / past) * 100.0;
            result[i] = Some(roc_val);
        } else {
            result[i] = None;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc() {
        let prices = vec![10.0, 11.0, 12.0, 13.0];
        let result = roc(&prices, 2).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());
        assert!(result[1].is_none());

        // i=2: (12-10)/10 * 100 = 20
        assert_eq!(result[2], Some(20.0));
        // i=3: (13-11)/11 * 100 = 18.18...
        assert!(result[3].is_some());
    }
}
