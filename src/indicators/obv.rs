//! On-Balance Volume (OBV) indicator.

use super::{IndicatorError, Result};

/// Calculate On-Balance Volume (OBV).
///
/// OBV is a cumulative indicator that adds volume on up days and subtracts volume on down days.
/// It measures buying and selling pressure.
///
/// # Arguments
///
/// * `closes` - Close prices
/// * `volumes` - Trading volumes
///
/// # Returns
///
/// Vector of cumulative OBV values.
///
/// # Example
///
/// ```
/// use finance_query::indicators::obv;
///
/// let closes = vec![100.0, 102.0, 101.0, 103.0, 105.0];
/// let volumes = vec![1000.0, 1200.0, 900.0, 1500.0, 2000.0];
///
/// let result = obv(&closes, &volumes).unwrap();
/// assert_eq!(result.len(), 5);
/// ```
pub fn obv(closes: &[f64], volumes: &[f64]) -> Result<Vec<Option<f64>>> {
    if closes.len() < 2 {
        return Err(IndicatorError::InsufficientData {
            need: 2,
            got: closes.len(),
        });
    }

    if closes.len() != volumes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "Closes and volumes must have the same length".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(closes.len());
    let mut obv_value = 0.0;

    result.push(Some(obv_value)); // First value is 0

    for i in 1..closes.len() {
        if closes[i] > closes[i - 1] {
            obv_value += volumes[i];
        } else if closes[i] < closes[i - 1] {
            obv_value -= volumes[i];
        }
        // If close == previous close, OBV unchanged
        result.push(Some(obv_value));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv_basic() {
        let closes = vec![100.0, 102.0, 101.0, 103.0, 105.0];
        let volumes = vec![1000.0, 1200.0, 900.0, 1500.0, 2000.0];

        let result = obv(&closes, &volumes).unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], Some(0.0)); // Start at 0
        assert_eq!(result[1], Some(1200.0)); // Price up, add volume
        assert_eq!(result[2], Some(300.0)); // Price down, subtract volume
        assert_eq!(result[3], Some(1800.0)); // Price up, add volume
        assert_eq!(result[4], Some(3800.0)); // Price up, add volume
    }

    #[test]
    fn test_obv_insufficient_data() {
        let closes = vec![100.0];
        let volumes = vec![1000.0];

        let result = obv(&closes, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_obv_mismatched_lengths() {
        let closes = vec![100.0, 102.0, 101.0];
        let volumes = vec![1000.0, 1200.0];

        let result = obv(&closes, &volumes);
        assert!(result.is_err());
    }
}
