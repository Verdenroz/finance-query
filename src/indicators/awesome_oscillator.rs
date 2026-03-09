//! Awesome Oscillator (AO) indicator.

use super::{IndicatorError, Result, sma::sma};

/// Calculate Awesome Oscillator (AO).
///
/// AO = SMA(median price, fast) - SMA(median price, slow)
/// Median price = (High + Low) / 2
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `fast` - Fast SMA period (default: 5)
/// * `slow` - Slow SMA period (default: 34)
///
/// # Example
///
/// ```
/// use finance_query::indicators::awesome_oscillator;
///
/// let highs = vec![10.0; 35];
/// let lows = vec![8.0; 35];
/// let result = awesome_oscillator(&highs, &lows, 5, 34).unwrap();
/// ```
pub fn awesome_oscillator(
    highs: &[f64],
    lows: &[f64],
    fast: usize,
    slow: usize,
) -> Result<Vec<Option<f64>>> {
    if fast == 0 || slow == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    if fast >= slow {
        return Err(IndicatorError::InvalidPeriod(
            "fast period must be less than slow period".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < slow {
        return Err(IndicatorError::InsufficientData {
            need: slow,
            got: len,
        });
    }

    let median_prices: Vec<f64> = highs
        .iter()
        .zip(lows.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();

    let sma_fast = sma(&median_prices, fast);
    let sma_slow = sma(&median_prices, slow);

    let result = sma_fast
        .into_iter()
        .zip(sma_slow)
        .map(|(f, s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awesome_oscillator_defaults() {
        let highs = vec![10.0; 35];
        let lows = vec![8.0; 35];
        let result = awesome_oscillator(&highs, &lows, 5, 34).unwrap();

        assert_eq!(result.len(), 35);
        // Valid from index 33 (slow period - 1)
        assert!(result[32].is_none());
        assert!(result[33].is_some());
    }

    #[test]
    fn test_awesome_oscillator_custom_periods() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        // Custom: fast=3, slow=10
        let result = awesome_oscillator(&highs, &lows, 3, 10).unwrap();
        assert_eq!(result.len(), 20);
        assert!(result[8].is_none());
        assert!(result[9].is_some());
    }

    #[test]
    fn test_awesome_oscillator_custom_produces_different_output() {
        let highs: Vec<f64> = (1..=40).map(|i| i as f64 + 0.5).collect();
        let lows: Vec<f64> = (1..=40).map(|i| i as f64 - 0.5).collect();
        let default = awesome_oscillator(&highs, &lows, 5, 34).unwrap();
        let custom = awesome_oscillator(&highs, &lows, 3, 20).unwrap();
        // Different periods must yield different (or identically zero, but here trending) values
        let idx = 33;
        assert!(default[idx].is_some());
        assert!(custom[idx].is_some());
        assert_ne!(default[idx], custom[idx]);
    }

    #[test]
    fn test_awesome_oscillator_fast_must_be_less_than_slow() {
        let highs = vec![10.0; 35];
        let lows = vec![8.0; 35];
        assert!(awesome_oscillator(&highs, &lows, 34, 5).is_err());
        assert!(awesome_oscillator(&highs, &lows, 10, 10).is_err());
    }
}
