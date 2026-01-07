//! Average Directional Index (ADX) indicator.

use super::{IndicatorError, Result};

/// Calculate Average Directional Index (ADX).
///
/// Measures trend strength (not direction).
/// Returns value between 0-100.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::adx;
///
/// let highs = vec![10.0; 30];
/// let lows = vec![8.0; 30];
/// let closes = vec![9.0; 30];
/// let result = adx(&highs, &lows, &closes, 14).unwrap();
/// ```
pub fn adx(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 2 * period {
        return Err(IndicatorError::InsufficientData {
            need: 2 * period,
            got: len,
        });
    }

    let mut tr_values = Vec::with_capacity(len);
    let mut plus_dm = Vec::with_capacity(len);
    let mut minus_dm = Vec::with_capacity(len);

    tr_values.push(0.0);
    plus_dm.push(0.0);
    minus_dm.push(0.0);

    for i in 1..len {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        tr_values.push(tr);

        let up_move = highs[i] - highs[i - 1];
        let down_move = lows[i - 1] - lows[i];

        let plus = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };

        plus_dm.push(plus);
        minus_dm.push(minus);
    }

    let mut smoothed_tr = vec![0.0; len];
    let mut smoothed_plus = vec![0.0; len];
    let mut smoothed_minus = vec![0.0; len];
    let mut dx_values = vec![0.0; len];

    let mut tr_sum = 0.0;
    let mut plus_sum = 0.0;
    let mut minus_sum = 0.0;

    for i in 1..=period {
        tr_sum += tr_values[i];
        plus_sum += plus_dm[i];
        minus_sum += minus_dm[i];
    }

    smoothed_tr[period] = tr_sum / period as f64;
    smoothed_plus[period] = plus_sum / period as f64;
    smoothed_minus[period] = minus_sum / period as f64;

    let plus_di = if smoothed_tr[period] != 0.0 {
        100.0 * smoothed_plus[period] / smoothed_tr[period]
    } else {
        0.0
    };
    let minus_di = if smoothed_tr[period] != 0.0 {
        100.0 * smoothed_minus[period] / smoothed_tr[period]
    } else {
        0.0
    };
    let di_sum = plus_di + minus_di;
    dx_values[period] = if di_sum != 0.0 {
        100.0 * (plus_di - minus_di).abs() / di_sum
    } else {
        0.0
    };

    for i in (period + 1)..len {
        smoothed_tr[i] =
            ((smoothed_tr[i - 1] * (period - 1) as f64) + tr_values[i]) / period as f64;
        smoothed_plus[i] =
            ((smoothed_plus[i - 1] * (period - 1) as f64) + plus_dm[i]) / period as f64;
        smoothed_minus[i] =
            ((smoothed_minus[i - 1] * (period - 1) as f64) + minus_dm[i]) / period as f64;

        let plus_di = if smoothed_tr[i] != 0.0 {
            100.0 * smoothed_plus[i] / smoothed_tr[i]
        } else {
            0.0
        };
        let minus_di = if smoothed_tr[i] != 0.0 {
            100.0 * smoothed_minus[i] / smoothed_tr[i]
        } else {
            0.0
        };
        let di_sum = plus_di + minus_di;
        dx_values[i] = if di_sum != 0.0 {
            100.0 * (plus_di - minus_di).abs() / di_sum
        } else {
            0.0
        };
    }

    let mut result = vec![None; len];

    let mut dx_sum = 0.0;
    for &dx in dx_values.iter().skip(period).take(period) {
        dx_sum += dx;
    }

    let first_adx_idx = 2 * period - 1;
    if first_adx_idx < len {
        let mut adx = dx_sum / period as f64;
        result[first_adx_idx] = Some(adx);

        for i in (first_adx_idx + 1)..len {
            adx = ((adx * (period - 1) as f64) + dx_values[i]) / period as f64;
            result[i] = Some(adx);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adx() {
        let highs = vec![10.0; 30];
        let lows = vec![8.0; 30];
        let closes = vec![9.0; 30];
        let result = adx(&highs, &lows, &closes, 14).unwrap();

        assert_eq!(result.len(), 30);
        // Valid from index 2*14 - 1 = 27
        assert!(result[26].is_none());
        assert!(result[27].is_some());
    }
}
