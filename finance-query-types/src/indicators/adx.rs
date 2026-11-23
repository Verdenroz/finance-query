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

    // Single-pass: compute TR/+DM/-DM inline, apply Wilder smoothing, accumulate DX,
    // then compute ADX — eliminates 7 intermediate Vec allocations.
    let period_f = period as f64;
    let period_m1 = (period - 1) as f64;

    // Seed smoothed values from the first `period` bars (indices 1..=period).
    let mut s_tr = 0.0_f64;
    let mut s_plus = 0.0_f64;
    let mut s_minus = 0.0_f64;
    for i in 1..=period {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        s_tr += high_low.max(high_close).max(low_close);

        let up = highs[i] - highs[i - 1];
        let dn = lows[i - 1] - lows[i];
        if up > dn && up > 0.0 {
            s_plus += up;
        }
        if dn > up && dn > 0.0 {
            s_minus += dn;
        }
    }
    s_tr /= period_f;
    s_plus /= period_f;
    s_minus /= period_f;

    let tr_dm = |i: usize| -> (f64, f64, f64) {
        let high_low = highs[i] - lows[i];
        let high_close = (highs[i] - closes[i - 1]).abs();
        let low_close = (lows[i] - closes[i - 1]).abs();
        let tr = high_low.max(high_close).max(low_close);
        let up = highs[i] - highs[i - 1];
        let dn = lows[i - 1] - lows[i];
        let plus = if up > dn && up > 0.0 { up } else { 0.0 };
        let minus = if dn > up && dn > 0.0 { dn } else { 0.0 };
        (tr, plus, minus)
    };
    let dx_at = |str: f64, sp: f64, sm: f64| -> f64 {
        let p_di = if str != 0.0 { 100.0 * sp / str } else { 0.0 };
        let m_di = if str != 0.0 { 100.0 * sm / str } else { 0.0 };
        let di_sum = p_di + m_di;
        if di_sum != 0.0 {
            100.0 * (p_di - m_di).abs() / di_sum
        } else {
            0.0
        }
    };

    // Accumulate DX values from index `period` to `2*period - 1` for the first ADX seed.
    let mut dx_sum = dx_at(s_tr, s_plus, s_minus);
    for i in (period + 1)..=(2 * period - 1).min(len - 1) {
        let (tr, plus, minus) = tr_dm(i);
        s_tr = (s_tr * period_m1 + tr) / period_f;
        s_plus = (s_plus * period_m1 + plus) / period_f;
        s_minus = (s_minus * period_m1 + minus) / period_f;
        dx_sum += dx_at(s_tr, s_plus, s_minus);
    }

    let mut result = vec![None; len];
    let first_adx_idx = 2 * period - 1;

    if first_adx_idx < len {
        let mut adx = dx_sum / period_f;
        result[first_adx_idx] = Some(adx);

        for (i, slot) in result.iter_mut().enumerate().skip(first_adx_idx + 1) {
            let (tr, plus, minus) = tr_dm(i);
            s_tr = (s_tr * period_m1 + tr) / period_f;
            s_plus = (s_plus * period_m1 + plus) / period_f;
            s_minus = (s_minus * period_m1 + minus) / period_f;
            adx = (adx * period_m1 + dx_at(s_tr, s_plus, s_minus)) / period_f;
            *slot = Some(adx);
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
