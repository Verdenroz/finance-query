//! Arnaud Legoux Moving Average (ALMA) indicator.

use super::{IndicatorError, Result};

/// Calculate Arnaud Legoux Moving Average (ALMA).
///
/// Gaussian-weighted moving average with configurable offset and sigma.
/// Standard parameters: offset=0.85, sigma=6.0
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
/// * `offset` - Offset parameter (0.0 to 1.0)
/// * `sigma` - Sigma parameter
///
/// # Example
///
/// ```
/// use finance_query::indicators::alma;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let result = alma(&prices, 3, 0.85, 6.0).unwrap();
/// ```
pub fn alma(data: &[f64], period: usize, offset: f64, sigma: f64) -> Result<Vec<Option<f64>>> {
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

    let m = (period as f64 - 1.0) * offset;
    let s = period as f64 / sigma;

    // Precompute weights
    let mut weights = Vec::with_capacity(period);
    for i in 0..period {
        let weight = (-(i as f64 - m).powi(2) / (2.0 * s.powi(2))).exp();
        weights.push(weight);
    }

    for i in (period - 1)..data.len() {
        let start_idx = i + 1 - period;
        let price_slice = &data[start_idx..=i];

        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for (j, &price) in price_slice.iter().enumerate() {
            let weight = weights[j];
            weighted_sum += price * weight;
            weight_sum += weight;
        }

        if weight_sum != 0.0 {
            result[i] = Some(weighted_sum / weight_sum);
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
    fn test_alma() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let result = alma(&prices, 3, 0.85, 6.0).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
    }
}
