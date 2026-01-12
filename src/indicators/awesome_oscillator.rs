//! Awesome Oscillator (AO) indicator.

use super::{IndicatorError, Result, sma::sma};

/// Calculate Awesome Oscillator (AO).
///
/// AO = SMA(median price, 5) - SMA(median price, 34)
/// Median price = (High + Low) / 2
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
///
/// # Example
///
/// ```
/// use finance_query::indicators::awesome_oscillator;
///
/// let highs = vec![10.0; 35];
/// let lows = vec![8.0; 35];
/// let result = awesome_oscillator(&highs, &lows).unwrap();
/// ```
pub fn awesome_oscillator(highs: &[f64], lows: &[f64]) -> Result<Vec<Option<f64>>> {
    let len = highs.len();
    if lows.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 34 {
        return Err(IndicatorError::InsufficientData { need: 34, got: len });
    }

    let mut median_prices = Vec::with_capacity(len);
    for i in 0..len {
        median_prices.push((highs[i] + lows[i]) / 2.0);
    }

    let sma5 = sma(&median_prices, 5);
    let sma34 = sma(&median_prices, 34);

    let mut result = vec![None; len];

    for i in 0..len {
        if let (Some(s5), Some(s34)) = (sma5[i], sma34[i]) {
            result[i] = Some(s5 - s34);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awesome_oscillator() {
        let highs = vec![10.0; 35];
        let lows = vec![8.0; 35];
        let result = awesome_oscillator(&highs, &lows).unwrap();

        assert_eq!(result.len(), 35);
        // Valid from index 33 (34th element)
        assert!(result[32].is_none());
        assert!(result[33].is_some());
    }
}
