//! Hull Moving Average (HMA) indicator.

use super::{IndicatorError, Result, wma::wma};

/// Calculate Hull Moving Average (HMA).
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
/// Responsive moving average with reduced lag.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Formula
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
///
/// # Example
///
/// ```
/// use finance_query::indicators::hma;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
/// let result = hma(&prices, 4).unwrap();
/// ```
pub fn hma(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let half_period = period / 2;
    let sqrt_period = (period as f64).sqrt() as usize;

    if sqrt_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Sqrt period is 0".to_string(),
        ));
    }

    let wma_half = wma(data, half_period)?;
    let wma_full = wma(data, period)?;

    // Calculate 2 * WMA(n/2) - WMA(n)
    // We collect valid values to pass to the next WMA
    let mut valid_diff_series = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        // Skip None values
        if let (Some(wh), Some(wf)) = (wma_half[i], wma_full[i]) {
            let val = 2.0 * wh - wf;
            valid_diff_series.push(val);
        }
    }

    if valid_diff_series.len() < sqrt_period {
        return Err(IndicatorError::InsufficientData {
            need: period + sqrt_period,
            got: data.len(),
        });
    }

    let hma_values = wma(&valid_diff_series, sqrt_period)?;

    let mut result = vec![None; data.len()];

    // valid_diff_series starts at index (period - 1) of original data.
    // hma_values[k] corresponds to valid_diff_series[k]

    for (k, val) in hma_values.into_iter().enumerate() {
        let original_idx = k + period - 1;
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
    fn test_hma() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
        let result = hma(&prices, 4).unwrap();

        assert_eq!(result.len(), prices.len());
        // period=4, half=2, sqrt=2
        // wma_half valid from index 1
        // wma_full valid from index 3
        // diff valid from index 3
        // hma (wma of diff, period 2) valid from index 3 + (2-1) = 4

        assert!(result[0].is_none());
        assert!(result[3].is_none());
        assert!(result[4].is_some());
    }
}
