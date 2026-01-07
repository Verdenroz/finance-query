//! Donchian Channels indicator.

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Result of Donchian Channels calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonchianChannelsResult {
    /// Upper channel
    pub upper: Vec<Option<f64>>,
    /// Middle channel
    pub middle: Vec<Option<f64>>,
    /// Lower channel
    pub lower: Vec<Option<f64>>,
}

/// Calculate Donchian Channels.
///
/// Upper Channel = Highest high over period
/// Lower Channel = Lowest low over period
/// Middle Channel = (Upper + Lower) / 2
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::donchian_channels;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let result = donchian_channels(&highs, &lows, 3).unwrap();
/// ```
pub fn donchian_channels(
    highs: &[f64],
    lows: &[f64],
    period: usize,
) -> Result<DonchianChannelsResult> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len {
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

    let mut upper = vec![None; len];
    let mut middle = vec![None; len];
    let mut lower = vec![None; len];

    for i in (period - 1)..len {
        let start_idx = i + 1 - period;
        let slice_highs = &highs[start_idx..=i];
        let slice_lows = &lows[start_idx..=i];

        let highest = slice_highs.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = slice_lows.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let mid = (highest + lowest) / 2.0;

        upper[i] = Some(highest);
        lower[i] = Some(lowest);
        middle[i] = Some(mid);
    }

    Ok(DonchianChannelsResult {
        upper,
        middle,
        lower,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_channels() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let result = donchian_channels(&highs, &lows, 3).unwrap();

        assert_eq!(result.upper.len(), 5);
        assert!(result.upper[0].is_none());
        assert!(result.upper[1].is_none());
        assert!(result.upper[2].is_some());
    }
}
