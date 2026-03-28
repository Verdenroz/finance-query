//! Triple Exponential Moving Average (TEMA) indicator.

use super::{IndicatorError, Result, ema::ema_raw};

/// Calculate Triple Exponential Moving Average (TEMA).
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
/// Further reduces lag compared to DEMA.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Formula
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
///
/// # Example
///
/// ```
/// use finance_query::indicators::tema;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
/// let result = tema(&prices, 3).unwrap();
/// ```
pub fn tema(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
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

    // ema_raw returns only valid values — no Option/None padding or filter_map needed
    let ema1 = ema_raw(data, period); // len = N - (period-1)
    if ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }
    let ema2 = ema_raw(&ema1, period); // len = N - 2*(period-1)
    if ema2.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 3 * period - 2,
            got: data.len(),
        });
    }
    let ema3 = ema_raw(&ema2, period); // len = N - 3*(period-1)

    let mut result = vec![None; data.len()];
    let off = period - 1;

    // ema3[k3] → original index k3 + 3*off
    // matching ema2 index = k3 + off, ema1 index = k3 + 2*off
    for (k3, &e3) in ema3.iter().enumerate() {
        let orig_idx = k3 + 3 * off;
        let e1 = ema1[k3 + 2 * off];
        let e2 = ema2[k3 + off];
        result[orig_idx] = Some(3.0 * e1 - 3.0 * e2 + e3);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tema() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
        let result = tema(&prices, 3).unwrap();

        assert_eq!(result.len(), prices.len());
        // First 3*period - 3 = 6 values should be None
        assert!(result[0].is_none());
        assert!(result[5].is_none());
        assert!(result[6].is_some());
    }
}
