//! Triple Exponential Moving Average (TEMA) indicator.

use super::{IndicatorError, Result, ema::ema};

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

    let ema1 = ema(data, period);

    let valid_ema1: Vec<f64> = ema1.iter().filter_map(|&x| x).collect();
    if valid_ema1.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period - 1,
            got: data.len(),
        });
    }

    let ema2 = ema(&valid_ema1, period);

    let valid_ema2: Vec<f64> = ema2.iter().filter_map(|&x| x).collect();
    if valid_ema2.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: 3 * period - 2,
            got: data.len(),
        });
    }

    let ema3 = ema(&valid_ema2, period);

    let mut result = vec![None; data.len()];

    for i in 0..data.len() {
        let offset1 = period - 1;
        let offset2 = 2 * (period - 1);

        if i >= offset2 {
            let j = i - offset1;
            let k = i - offset2;

            if j < ema2.len()
                && k < ema3.len()
                && let (Some(e1), Some(e2), Some(e3)) = (ema1[i], ema2[j], ema3[k])
            {
                result[i] = Some(3.0 * e1 - 3.0 * e2 + e3);
            }
        }
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
