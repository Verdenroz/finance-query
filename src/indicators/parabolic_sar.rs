//! Parabolic SAR indicator.

use super::{IndicatorError, Result};

/// Calculate Parabolic SAR.
///
/// Stop and Reverse indicator for trend-following.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `acceleration` - Acceleration factor (e.g., 0.02)
/// * `maximum` - Maximum acceleration (e.g., 0.2)
///
/// # Example
///
/// ```
/// use finance_query::indicators::parabolic_sar;
///
/// let highs = vec![10.0, 11.0, 12.0, 13.0];
/// let lows = vec![8.0, 9.0, 10.0, 11.0];
/// let closes = vec![9.0, 10.0, 11.0, 12.0];
/// let result = parabolic_sar(&highs, &lows, &closes, 0.02, 0.2).unwrap();
/// ```
pub fn parabolic_sar(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    acceleration: f64,
    maximum: f64,
) -> Result<Vec<Option<f64>>> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 2 {
        return Err(IndicatorError::InsufficientData { need: 2, got: len });
    }

    let mut result = vec![None; len];

    let mut is_long = closes[1] > closes[0];
    let mut sar = if is_long { lows[0] } else { highs[0] };
    let mut ep = if is_long { highs[1] } else { lows[1] };
    let mut af = acceleration;

    for i in 2..len {
        sar = sar + af * (ep - sar);

        let reversed = if is_long {
            closes[i] < sar
        } else {
            closes[i] > sar
        };

        if reversed {
            is_long = !is_long;
            sar = ep;
            ep = if is_long { highs[i] } else { lows[i] };
            af = acceleration;
        } else if is_long && highs[i] > ep {
            ep = highs[i];
            af = (af + acceleration).min(maximum);
        } else if !is_long && lows[i] < ep {
            ep = lows[i];
            af = (af + acceleration).min(maximum);
        }

        result[i] = Some(sar);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parabolic_sar() {
        let highs = vec![10.0, 11.0, 12.0, 13.0];
        let lows = vec![8.0, 9.0, 10.0, 11.0];
        let closes = vec![9.0, 10.0, 11.0, 12.0];
        let result = parabolic_sar(&highs, &lows, &closes, 0.02, 0.2).unwrap();

        assert_eq!(result.len(), 4);
        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
        assert!(result[3].is_some());
    }
}
