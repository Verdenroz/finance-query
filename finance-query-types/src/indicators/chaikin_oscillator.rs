//! Chaikin Oscillator indicator.

use super::{IndicatorError, Result, ema::ema_raw};

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

    let mut ad_cumulative = 0.0;
    let ad_series: Vec<f64> = highs
        .iter()
        .zip(lows)
        .zip(closes)
        .zip(volumes)
        .map(|(((&high, &low), &close), &volume)| {
            let high_low = high - low;
            if high_low != 0.0 {
                let mf_multiplier = ((close - low) - (high - close)) / high_low;
                ad_cumulative += mf_multiplier * volume;
            }
            ad_cumulative
        })
        .collect();

    // ema_raw returns only valid values — no Option overhead
    let ema3_raw = ema_raw(&ad_series, 3); // len = N-2, index k → orig k+2
    let ema10_raw = ema_raw(&ad_series, 10); // len = N-9, index k → orig k+9

    let mut result = vec![None; len];

    // ema10_raw[k] aligns with orig index k+9; ema3_raw at same orig = ema3_raw[k+7]
    for ((out, &e10), &e3) in result[9..].iter_mut().zip(&ema10_raw).zip(&ema3_raw[7..]) {
        *out = Some(e3 - e10);
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
