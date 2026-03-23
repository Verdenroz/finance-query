//! Weighted Moving Average (WMA) indicator.

use super::{IndicatorError, Result};

/// Internal WMA returning only valid values as plain `f64` (no `Option` wrapping, no padding).
/// Length = `data.len() - (period - 1)`. Index `k` corresponds to original index `k + period - 1`.
pub(crate) fn wma_raw(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || data.len() < period {
        return Vec::new();
    }
    let weight_sum = (period * (period + 1) / 2) as f64;
    let period_f = period as f64;
    let initial_weighted: f64 = data[..period]
        .iter()
        .enumerate()
        .map(|(j, &x)| x * (j + 1) as f64)
        .sum();
    let mut wma_val = initial_weighted / weight_sum;
    let mut window_sum: f64 = data[..period].iter().sum();
    let mut result = Vec::with_capacity(data.len() - period + 1);
    result.push(wma_val);
    for i in period..data.len() {
        wma_val += (period_f * data[i] - window_sum) / weight_sum;
        result.push(wma_val);
        window_sum += data[i] - data[i - period];
    }
    result
}

/// Calculate Weighted Moving Average (WMA).
///
/// WMA gives more weight to recent prices in the calculation.
/// More recent prices have linearly increasing weights.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods for the WMA
///
/// # Returns
///
/// Vector of WMA values. Early values (before `period` data points) are None.
///
/// # Example
///
/// ```
/// use finance_query::indicators::wma;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
/// let result = wma(&prices, 3).unwrap();
///
/// // First 2 values are None (need 3 periods)
/// assert_eq!(result[0], None);
/// assert_eq!(result[1], None);
/// // Third value: (10*1 + 11*2 + 12*3) / (1+2+3) = 58/6 = 11.333...
/// assert!(result[2].is_some());
/// ```
pub fn wma(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
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

    let weight_sum = (period * (period + 1) / 2) as f64;
    let period_f = period as f64;

    // Compute first window weighted sum directly, then use O(N) recurrence:
    // WMA(i+1) = WMA(i) + (period * data[i+1] - window_sum) / weight_sum
    // where window_sum is the plain sum of the current window.
    let initial_weighted: f64 = data[..period]
        .iter()
        .enumerate()
        .map(|(j, &x)| x * (j + 1) as f64)
        .sum();
    let mut wma_val = initial_weighted / weight_sum;
    let mut window_sum: f64 = data[..period].iter().sum();

    let mut result = vec![None; period - 1];
    result.push(Some(wma_val));

    for i in period..data.len() {
        wma_val += (period_f * data[i] - window_sum) / weight_sum;
        result.push(Some(wma_val));
        window_sum += data[i] - data[i - period];
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wma_basic() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
        let result = wma(&prices, 3).unwrap();

        assert_eq!(result.len(), 6);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);

        // WMA(3) at index 2: (10*1 + 11*2 + 12*3) / 6 = 58/6 = 11.333...
        assert!((result[2].unwrap() - 11.333333).abs() < 0.001);

        // WMA(3) at index 3: (11*1 + 12*2 + 13*3) / 6 = 68/6 = 11.333...
        assert!((result[3].unwrap() - 12.333333).abs() < 0.001);
    }

    #[test]
    fn test_wma_insufficient_data() {
        let prices = vec![10.0, 11.0];
        let result = wma(&prices, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_wma_zero_period() {
        let prices = vec![10.0, 11.0, 12.0];
        let result = wma(&prices, 0);
        assert!(result.is_err());
    }
}
