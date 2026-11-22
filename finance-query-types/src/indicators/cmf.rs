//! Chaikin Money Flow (CMF) indicator.

use super::{IndicatorError, Result};

/// Calculate Chaikin Money Flow (CMF).
///
/// Measures buying/selling pressure over a period.
/// Returns value between -1 and 1.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `volumes` - Volume data
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::cmf;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];
/// let volumes = vec![100.0, 200.0, 150.0, 100.0, 50.0];
/// let result = cmf(&highs, &lows, &closes, &volumes, 3).unwrap();
/// ```
pub fn cmf(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
    period: usize,
) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len || volumes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: len,
        });
    }

    let mut result = vec![None; len];

    let mut mf_volumes = Vec::with_capacity(len);

    for i in 0..len {
        let high_low = highs[i] - lows[i];
        if high_low == 0.0 {
            mf_volumes.push(0.0);
        } else {
            let mf_multiplier = ((closes[i] - lows[i]) - (highs[i] - closes[i])) / high_low;
            mf_volumes.push(mf_multiplier * volumes[i]);
        }
    }

    let mut mf_volume_sum = 0.0;
    let mut volume_sum = 0.0;

    for i in 0..period {
        mf_volume_sum += mf_volumes[i];
        volume_sum += volumes[i];
    }

    if volume_sum != 0.0 {
        result[period - 1] = Some(mf_volume_sum / volume_sum);
    } else {
        result[period - 1] = Some(0.0);
    }

    for i in period..len {
        let old_idx = i - period;
        mf_volume_sum -= mf_volumes[old_idx];
        volume_sum -= volumes[old_idx];

        mf_volume_sum += mf_volumes[i];
        volume_sum += volumes[i];

        if volume_sum != 0.0 {
            result[i] = Some(mf_volume_sum / volume_sum);
        } else {
            result[i] = Some(0.0);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmf() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];
        let volumes = vec![100.0, 200.0, 150.0, 100.0, 50.0];
        let result = cmf(&highs, &lows, &closes, &volumes, 3).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[1].is_none());
        assert!(result[2].is_some());
    }
}
