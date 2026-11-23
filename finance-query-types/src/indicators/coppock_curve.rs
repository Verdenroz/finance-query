//! Coppock Curve indicator.

use super::{IndicatorError, Result, wma::wma_raw};

/// Calculate Coppock Curve.
///
/// Combines two Rate-of-Change values with WMA smoothing:
/// `Coppock = WMA(ROC(long_roc) + ROC(short_roc), wma_period)`
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `long_roc` - Long ROC period (default: 14)
/// * `short_roc` - Short ROC period (default: 11)
/// * `wma_period` - WMA smoothing period (default: 10)
///
/// # Example
///
/// ```
/// use finance_query::indicators::coppock_curve;
///
/// let prices = vec![10.0; 30];
/// let result = coppock_curve(&prices, 14, 11, 10).unwrap();
/// ```
pub fn coppock_curve(
    data: &[f64],
    long_roc: usize,
    short_roc: usize,
    wma_period: usize,
) -> Result<Vec<Option<f64>>> {
    if long_roc == 0 || short_roc == 0 || wma_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    let roc_period = long_roc.max(short_roc);
    let min_len = roc_period + wma_period - 1;
    if data.len() < min_len {
        return Err(IndicatorError::InsufficientData {
            need: min_len,
            got: data.len(),
        });
    }

    // Build only the valid ROC sums (starting from index roc_period) — no leading zeros
    let valid_roc_sums: Vec<f64> = (roc_period..data.len())
        .map(|i| {
            let roc_long = if data[i - long_roc] != 0.0 {
                (data[i] - data[i - long_roc]) / data[i - long_roc] * 100.0
            } else {
                0.0
            };
            let roc_short = if data[i - short_roc] != 0.0 {
                (data[i] - data[i - short_roc]) / data[i - short_roc] * 100.0
            } else {
                0.0
            };
            roc_long + roc_short
        })
        .collect();

    let wma_vals = wma_raw(&valid_roc_sums, wma_period);

    let mut result = vec![None; data.len()];
    // wma_vals[j] → valid_roc_sums[j + wma_period - 1] → original index j + roc_period + wma_period - 1
    let base = roc_period + wma_period - 1;
    for (j, &v) in wma_vals.iter().enumerate() {
        let orig = j + base;
        if orig < data.len() {
            result[orig] = Some(v);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coppock_curve_defaults() {
        let prices = vec![10.0; 30];
        let result = coppock_curve(&prices, 14, 11, 10).unwrap();

        assert_eq!(result.len(), 30);
        // Valid from index max(14,11) + 10 - 1 = 23
        assert!(result[22].is_none());
        assert!(result[23].is_some());
    }

    #[test]
    fn test_coppock_curve_custom_periods() {
        let prices = vec![10.0; 20];
        let result = coppock_curve(&prices, 5, 3, 4).unwrap();
        assert_eq!(result.len(), 20);
        // Valid from index max(5,3) + 4 - 1 = 8
        assert!(result[7].is_none());
        assert!(result[8].is_some());
    }

    #[test]
    fn test_coppock_curve_custom_produces_different_output() {
        let prices: Vec<f64> = (1..=40).map(|i| i as f64).collect();
        let default = coppock_curve(&prices, 14, 11, 10).unwrap();
        let custom = coppock_curve(&prices, 7, 5, 5).unwrap();
        let idx = 23;
        assert!(default[idx].is_some());
        assert!(custom[idx].is_some());
        assert_ne!(default[idx], custom[idx]);
    }
}
