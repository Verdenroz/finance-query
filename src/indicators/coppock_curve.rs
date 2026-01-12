//! Coppock Curve indicator.

use super::{IndicatorError, Result, wma::wma};

/// Calculate Coppock Curve.
///
/// Combines ROC with WMA smoothing.
/// Coppock = WMA(ROC(14) + ROC(11), 10)
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
///
/// # Example
///
/// ```
/// use finance_query::indicators::coppock_curve;
///
/// let prices = vec![10.0; 30];
/// let result = coppock_curve(&prices).unwrap();
/// ```
pub fn coppock_curve(data: &[f64]) -> Result<Vec<Option<f64>>> {
    if data.len() < 25 {
        return Err(IndicatorError::InsufficientData {
            need: 25,
            got: data.len(),
        });
    }

    let mut roc_sum_series = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        if i >= 14 {
            let roc14 = if data[i - 14] != 0.0 {
                ((data[i] - data[i - 14]) / data[i - 14]) * 100.0
            } else {
                0.0
            };

            let roc11 = if data[i - 11] != 0.0 {
                ((data[i] - data[i - 11]) / data[i - 11]) * 100.0
            } else {
                0.0
            };

            roc_sum_series.push(roc14 + roc11);
        } else {
            roc_sum_series.push(0.0);
        }
    }

    let valid_start = 14;
    let valid_roc_sums = &roc_sum_series[valid_start..];

    let wma_values = wma(valid_roc_sums, 10)?;

    let mut result = vec![None; data.len()];

    for (j, val) in wma_values.into_iter().enumerate() {
        let original_idx = j + valid_start;
        if original_idx < result.len() {
            result[original_idx] = val;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coppock_curve() {
        let prices = vec![10.0; 30];
        let result = coppock_curve(&prices).unwrap();

        assert_eq!(result.len(), 30);
        // Valid from index 14 + 10 - 1 = 23
        assert!(result[22].is_none());
        assert!(result[23].is_some());
    }
}
