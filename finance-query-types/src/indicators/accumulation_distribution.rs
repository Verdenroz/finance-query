//! Accumulation/Distribution (A/D) indicator.

use super::{IndicatorError, Result};

/// Calculate Accumulation/Distribution (A/D).
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `volumes` - Volume data
///
/// # Example
///
/// ```
/// use finance_query::indicators::accumulation_distribution;
///
/// let highs = vec![10.0; 10];
/// let lows = vec![8.0; 10];
/// let closes = vec![9.0; 10];
/// let volumes = vec![100.0; 10];
/// let result = accumulation_distribution(&highs, &lows, &closes, &volumes).unwrap();
/// ```
pub fn accumulation_distribution(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
) -> Result<Vec<Option<f64>>> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len || volumes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(len);
    let mut ad_cumulative = 0.0;

    for i in 0..len {
        let high_low = highs[i] - lows[i];
        if high_low != 0.0 {
            let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;
            ad_cumulative += mf_multiplier * volumes[i];
        }
        result.push(Some(ad_cumulative));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulation_distribution() {
        let highs = vec![10.0; 10];
        let lows = vec![8.0; 10];
        let closes = vec![9.0; 10];
        let volumes = vec![100.0; 10];
        let result = accumulation_distribution(&highs, &lows, &closes, &volumes).unwrap();

        assert_eq!(result.len(), 10);
        assert!(result[0].is_some());
    }
}
