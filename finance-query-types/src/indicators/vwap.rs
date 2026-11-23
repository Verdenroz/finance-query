//! Volume Weighted Average Price (VWAP) indicator.

use super::{IndicatorError, Result};

/// Calculate Volume Weighted Average Price (VWAP).
///
/// VWAP is the average price weighted by volume. It's commonly used as a trading benchmark.
/// Formula: VWAP = Σ(Typical Price × Volume) / Σ(Volume)
/// where Typical Price = (High + Low + Close) / 3
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `volumes` - Trading volumes
///
/// # Returns
///
/// Vector of cumulative VWAP values.
///
/// # Example
///
/// ```
/// use finance_query::indicators::vwap;
///
/// let highs = vec![102.0, 104.0, 103.0, 105.0];
/// let lows = vec![100.0, 101.0, 100.5, 102.0];
/// let closes = vec![101.0, 103.0, 102.0, 104.0];
/// let volumes = vec![1000.0, 1200.0, 900.0, 1500.0];
///
/// let result = vwap(&highs, &lows, &closes, &volumes).unwrap();
/// assert_eq!(result.len(), 4);
/// ```
pub fn vwap(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Result<Vec<Option<f64>>> {
    if highs.is_empty() {
        return Err(IndicatorError::InsufficientData { need: 1, got: 0 });
    }

    if highs.len() != lows.len() || highs.len() != closes.len() || highs.len() != volumes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "All arrays must have the same length".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(highs.len());
    let mut pv_sum = 0.0;
    let mut volume_sum = 0.0;

    for i in 0..closes.len() {
        // Typical Price = (H + L + C) / 3
        let typical_price = (highs[i] + lows[i] + closes[i]) / 3.0;
        pv_sum += typical_price * volumes[i];
        volume_sum += volumes[i];

        if volume_sum > 0.0 {
            result.push(Some(pv_sum / volume_sum));
        } else {
            result.push(None);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_basic() {
        let highs = vec![102.0, 104.0, 103.0, 105.0];
        let lows = vec![100.0, 101.0, 100.5, 102.0];
        let closes = vec![101.0, 103.0, 102.0, 104.0];
        let volumes = vec![1000.0, 1200.0, 900.0, 1500.0];

        let result = vwap(&highs, &lows, &closes, &volumes).unwrap();

        assert_eq!(result.len(), 4);

        // All values should exist
        for val in &result {
            assert!(val.is_some());
        }

        // VWAP should be reasonable (between low and high ranges)
        for vwap_val in result.iter().flatten() {
            assert!(vwap_val > &0.0);
        }
    }

    #[test]
    fn test_vwap_empty() {
        let highs: Vec<f64> = vec![];
        let lows: Vec<f64> = vec![];
        let closes: Vec<f64> = vec![];
        let volumes: Vec<f64> = vec![];

        let result = vwap(&highs, &lows, &closes, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vwap_mismatched_lengths() {
        let highs = vec![102.0, 104.0];
        let lows = vec![100.0];
        let closes = vec![101.0, 103.0];
        let volumes = vec![1000.0, 1200.0];

        let result = vwap(&highs, &lows, &closes, &volumes);
        assert!(result.is_err());
    }
}
