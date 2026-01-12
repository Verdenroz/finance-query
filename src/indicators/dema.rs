//! Double Exponential Moving Average (DEMA) indicator.

use super::{IndicatorError, Result, ema::ema};

/// Calculate Double Exponential Moving Average (DEMA).
///
/// DEMA = 2 * EMA - EMA(EMA)
/// Reduces lag compared to simple EMA.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Formula
///
/// DEMA = 2 * EMA - EMA(EMA)
///
/// # Example
///
/// ```
/// use finance_query::indicators::dema;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
/// let result = dema(&prices, 3).unwrap();
/// ```
pub fn dema(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
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

    let ema1 = ema(data, period);

    // Extract valid values for EMA2 calculation
    let valid_ema1: Vec<f64> = ema1.iter().filter_map(|&x| x).collect();

    if valid_ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }

    let ema2 = ema(&valid_ema1, period);

    let mut result = vec![None; data.len()];

    for i in 0..data.len() {
        if i >= period - 1 {
            let j = i - (period - 1);
            if j < ema2.len()
                && let (Some(e1), Some(e2)) = (ema1[i], ema2[j])
            {
                result[i] = Some(2.0 * e1 - e2);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dema() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
        let result = dema(&prices, 3).unwrap();

        assert_eq!(result.len(), prices.len());
        // First 2*period - 2 = 4 values should be None
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_none());
        assert!(result[3].is_none());
        assert!(result[4].is_some());
    }
}
