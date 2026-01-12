//! Bollinger Bands indicator.

use super::{IndicatorError, Result, sma::sma};
use serde::{Deserialize, Serialize};

/// Bollinger Bands result containing upper, middle, and lower bands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BollingerBands {
    /// Upper band (SMA + std_dev * multiplier)
    pub upper: Vec<Option<f64>>,

    /// Middle band (SMA)
    pub middle: Vec<Option<f64>>,

    /// Lower band (SMA - std_dev * multiplier)
    pub lower: Vec<Option<f64>>,
}

/// Calculate Bollinger Bands.
///
/// Bollinger Bands consist of a middle band (SMA) and upper/lower bands that are
/// standard deviations away from the middle band. They help identify volatility and
/// potential overbought/oversold conditions.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods for the SMA (typically 20)
/// * `std_dev_multiplier` - Number of standard deviations (typically 2.0)
///
/// # Formula
///
/// - Middle Band = SMA(period)
/// - Upper Band = Middle Band + (std_dev_multiplier × standard deviation)
/// - Lower Band = Middle Band - (std_dev_multiplier × standard deviation)
///
/// # Example
///
/// ```
/// use finance_query::indicators::bollinger_bands;
///
/// let prices: Vec<f64> = (1..=30).map(|x| x as f64 + (x % 3) as f64).collect();
/// let result = bollinger_bands(&prices, 20, 2.0).unwrap();
///
/// assert_eq!(result.upper.len(), prices.len());
/// assert_eq!(result.middle.len(), prices.len());
/// assert_eq!(result.lower.len(), prices.len());
/// ```
pub fn bollinger_bands(
    data: &[f64],
    period: usize,
    std_dev_multiplier: f64,
) -> Result<BollingerBands> {
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

    // Calculate middle band (SMA)
    let middle = sma(data, period);

    let mut upper = Vec::with_capacity(data.len());
    let mut lower = Vec::with_capacity(data.len());

    // Calculate upper and lower bands
    for i in 0..data.len() {
        if i + 1 < period {
            upper.push(None);
            lower.push(None);
        } else {
            // Calculate standard deviation for this window
            let window = &data[i + 1 - period..=i];
            let mean = middle[i].unwrap(); // We know this exists because i >= period - 1

            let variance: f64 =
                window.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;

            let std_dev = variance.sqrt();

            upper.push(Some(mean + std_dev_multiplier * std_dev));
            lower.push(Some(mean - std_dev_multiplier * std_dev));
        }
    }

    Ok(BollingerBands {
        upper,
        middle,
        lower,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands_basic() {
        let data: Vec<f64> = (1..=30).map(|x| x as f64).collect();
        let result = bollinger_bands(&data, 20, 2.0).unwrap();

        assert_eq!(result.upper.len(), 30);
        assert_eq!(result.middle.len(), 30);
        assert_eq!(result.lower.len(), 30);

        // First 19 values should be None
        for i in 0..19 {
            assert_eq!(result.upper[i], None);
            assert_eq!(result.middle[i], None);
            assert_eq!(result.lower[i], None);
        }

        // Values after period should exist
        assert!(result.upper[19].is_some());
        assert!(result.middle[19].is_some());
        assert!(result.lower[19].is_some());

        // Upper should be > Middle > Lower
        for i in 19..30 {
            let upper = result.upper[i].unwrap();
            let middle = result.middle[i].unwrap();
            let lower = result.lower[i].unwrap();

            assert!(
                upper > middle,
                "Upper ({}) should be > middle ({}) at index {}",
                upper,
                middle,
                i
            );
            assert!(
                middle > lower,
                "Middle ({}) should be > lower ({}) at index {}",
                middle,
                lower,
                i
            );
        }
    }

    #[test]
    fn test_bollinger_bands_constant_price() {
        // Constant price should have zero standard deviation
        let data = vec![50.0; 30];
        let result = bollinger_bands(&data, 20, 2.0).unwrap();

        // All bands should be equal when std dev is 0
        for i in 19..30 {
            let upper = result.upper[i].unwrap();
            let middle = result.middle[i].unwrap();
            let lower = result.lower[i].unwrap();

            assert!((upper - middle).abs() < 0.0001);
            assert!((middle - lower).abs() < 0.0001);
            assert!((middle - 50.0).abs() < 0.0001);
        }
    }

    #[test]
    fn test_bollinger_bands_insufficient_data() {
        let data = vec![1.0, 2.0, 3.0];
        let result = bollinger_bands(&data, 20, 2.0);

        assert!(result.is_err());
    }

    #[test]
    fn test_bollinger_bands_volatility() {
        // Higher volatility should create wider bands
        let low_vol_data: Vec<f64> = (1..=30).map(|x| 50.0 + (x % 2) as f64).collect();
        let high_vol_data: Vec<f64> = (1..=30).map(|x| 50.0 + (x % 10) as f64 * 5.0).collect();

        let low_vol_result = bollinger_bands(&low_vol_data, 20, 2.0).unwrap();
        let high_vol_result = bollinger_bands(&high_vol_data, 20, 2.0).unwrap();

        // Compare band width at the last data point
        let low_vol_width = low_vol_result.upper[29].unwrap() - low_vol_result.lower[29].unwrap();
        let high_vol_width =
            high_vol_result.upper[29].unwrap() - high_vol_result.lower[29].unwrap();

        assert!(
            high_vol_width > low_vol_width,
            "High volatility bands ({}) should be wider than low volatility bands ({})",
            high_vol_width,
            low_vol_width
        );
    }
}
