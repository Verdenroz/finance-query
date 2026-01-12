//! Chaikin Oscillator indicator.

use super::{IndicatorError, Result, ema::ema};

/// Calculate Chaikin Oscillator.
///
/// Difference between 3-day EMA and 10-day EMA of A/D line.
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
/// use finance_query::indicators::chaikin_oscillator;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let volumes = vec![100.0; 20];
/// let result = chaikin_oscillator(&highs, &lows, &closes, &volumes).unwrap();
/// ```
pub fn chaikin_oscillator(
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
    if len < 10 {
        return Err(IndicatorError::InsufficientData { need: 10, got: len });
    }

    let mut ad_series = Vec::with_capacity(len);
    let mut ad_cumulative = 0.0;

    for i in 0..len {
        let high_low = highs[i] - lows[i];
        if high_low != 0.0 {
            let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;
            ad_cumulative += mf_multiplier * volumes[i];
        }
        ad_series.push(ad_cumulative);
    }

    let ema3 = ema(&ad_series, 3);
    let ema10 = ema(&ad_series, 10);

    let mut result = vec![None; len];

    for i in 0..len {
        if let (Some(e3), Some(e10)) = (ema3[i], ema10[i]) {
            result[i] = Some(e3 - e10);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaikin_oscillator() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let volumes = vec![100.0; 20];
        let result = chaikin_oscillator(&highs, &lows, &closes, &volumes).unwrap();

        assert_eq!(result.len(), 20);
        assert!(result[8].is_none());
        assert!(result[9].is_some());
    }
}
