//! McGinley Dynamic indicator.

use super::{IndicatorError, Result};

/// Calculate McGinley Dynamic.
///
/// Adaptive moving average that automatically adjusts for market speed.
///
/// ```text
/// MD[i] = MD[i-1] + (Price - MD[i-1]) / (N * (Price/MD[i-1])^4)
/// ```
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::mcginley_dynamic;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let result = mcginley_dynamic(&prices, 3).unwrap();
/// ```
pub fn mcginley_dynamic(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
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

    let mut result = vec![None; data.len()];

    // Initialize with SMA
    let initial_sum: f64 = data[..period].iter().sum();
    let mut md = initial_sum / period as f64;

    // The first value is at index period-1
    result[period - 1] = Some(md);

    for i in period..data.len() {
        let price = data[i];
        if md != 0.0 {
            let ratio = price / md;
            let factor = period as f64 * ratio.powi(4);
            if factor != 0.0 {
                md = md + (price - md) / factor;
            }
        }
        result[i] = Some(md);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcginley_dynamic() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let result = mcginley_dynamic(&prices, 3).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());
    }
}
