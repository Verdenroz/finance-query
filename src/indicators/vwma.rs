//! Volume Weighted Moving Average (VWMA) indicator.

use super::{IndicatorError, Result};

/// Calculate Volume Weighted Moving Average (VWMA).
///
/// Prices weighted by volume over the given period.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `volumes` - Volume data
/// * `period` - Number of periods
///
/// # Formula
///
/// VWMA = Sum(Price * Volume) / Sum(Volume)
///
/// # Example
///
/// ```
/// use finance_query::indicators::vwma;
///
/// let prices = vec![10.0, 10.0, 10.0];
/// let volumes = vec![100.0, 100.0, 100.0];
/// let result = vwma(&prices, &volumes, 2).unwrap();
/// ```
pub fn vwma(data: &[f64], volumes: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() != volumes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "Data and volumes must have same length".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let mut result = vec![None; data.len()];

    for i in (period - 1)..data.len() {
        let start_idx = i + 1 - period;
        let price_slice = &data[start_idx..=i];
        let volume_slice = &volumes[start_idx..=i];

        let mut pv_sum = 0.0;
        let mut volume_sum = 0.0;

        for (&p, &v) in price_slice.iter().zip(volume_slice.iter()) {
            pv_sum += p * v;
            volume_sum += v;
        }

        if volume_sum != 0.0 {
            result[i] = Some(pv_sum / volume_sum);
        } else {
            result[i] = None;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwma() {
        let prices = vec![10.0, 12.0, 14.0, 16.0];
        let volumes = vec![100.0, 200.0, 100.0, 200.0];
        let result = vwma(&prices, &volumes, 2).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());

        // i=1: (10*100 + 12*200) / (100+200) = (1000 + 2400) / 300 = 3400/300 = 11.333
        assert!(result[1].is_some());
        let val = result[1].unwrap();
        assert!((val - 11.3333).abs() < 0.001);
    }
}
