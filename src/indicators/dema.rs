//! Double Exponential Moving Average (DEMA) indicator.

use super::{IndicatorError, Result, ema::ema_raw};

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

    // ema_raw returns only valid values (no Option/None padding)
    let ema1 = ema_raw(data, period); // len = N - (period-1)
    if ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }
    let ema2 = ema_raw(&ema1, period); // len = N - 2*(period-1)

    let mut result = vec![None; data.len()];
    let off = period - 1;

    // ema1[k1] → original index k1 + off
    // ema2[k2] → original index k2 + 2*off; matching ema1 index = k2 + off
    for (k2, &e2) in ema2.iter().enumerate() {
        let orig_idx = k2 + 2 * off;
        result[orig_idx] = Some(2.0 * ema1[k2 + off] - e2);
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
