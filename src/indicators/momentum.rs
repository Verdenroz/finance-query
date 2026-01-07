//! Momentum indicator.

use super::{IndicatorError, Result};

/// Calculate Momentum.
///
/// Simple difference between current price and price n periods ago.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::momentum;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0];
/// let result = momentum(&prices, 2).unwrap();
/// ```
pub fn momentum(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
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
        result[i] = Some(current - past);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_momentum() {
        let prices = vec![10.0, 11.0, 12.0, 13.0];
        let result = momentum(&prices, 2).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());
        assert!(result[1].is_none());

        // i=2: 12-10 = 2
        assert_eq!(result[2], Some(2.0));
        // i=3: 13-11 = 2
        assert_eq!(result[3], Some(2.0));
    }
}
