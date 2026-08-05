//! Pivot Points indicator (Standard and Fibonacci variants).
//!
//! Pivot points are classic intraday/swing support-resistance levels derived
//! from the *previous* bar's high/low/close and held for the current bar —
//! the same convention floor traders have used since well before charting
//! software existed.

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Pivot point support/resistance levels for a single bar.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PivotPoints {
    /// Central pivot point: `(high + low + close) / 3`
    pub pivot: f64,
    /// First resistance level
    pub r1: f64,
    /// Second resistance level
    pub r2: f64,
    /// Third resistance level
    pub r3: f64,
    /// First support level
    pub s1: f64,
    /// Second support level
    pub s2: f64,
    /// Third support level
    pub s3: f64,
}

fn validate(highs: &[f64], lows: &[f64], closes: &[f64]) -> Result<()> {
    if highs.len() != lows.len() || highs.len() != closes.len() {
        return Err(IndicatorError::InvalidPeriod(
            "highs, lows, and closes must have the same length".to_string(),
        ));
    }
    if highs.len() < 2 {
        return Err(IndicatorError::InsufficientData {
            need: 2,
            got: highs.len(),
        });
    }
    Ok(())
}

/// Calculate classic (standard) pivot points.
///
/// Each bar's levels are derived from the **previous** bar's high/low/close;
/// the first bar has no prior bar and is therefore `None`.
///
/// # Formula
///
/// - Pivot = (High + Low + Close) / 3
/// - R1 = 2×Pivot − Low, S1 = 2×Pivot − High
/// - R2 = Pivot + (High − Low), S2 = Pivot − (High − Low)
/// - R3 = High + 2×(Pivot − Low), S3 = Low − 2×(High − Pivot)
///
/// # Example
///
/// ```
/// use finance_query::indicators::pivot_points;
///
/// let highs = vec![10.0, 12.0, 11.0];
/// let lows = vec![8.0, 9.0, 8.5];
/// let closes = vec![9.0, 11.0, 10.0];
/// let result = pivot_points(&highs, &lows, &closes).unwrap();
///
/// assert!(result[0].is_none());
/// assert!(result[1].is_some());
/// ```
pub fn pivot_points(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
) -> Result<Vec<Option<PivotPoints>>> {
    validate(highs, lows, closes)?;
    let mut result = vec![None; highs.len()];
    for i in 1..highs.len() {
        let (h, l, c) = (highs[i - 1], lows[i - 1], closes[i - 1]);
        let pivot = (h + l + c) / 3.0;
        let range = h - l;
        result[i] = Some(PivotPoints {
            pivot,
            r1: 2.0 * pivot - l,
            s1: 2.0 * pivot - h,
            r2: pivot + range,
            s2: pivot - range,
            r3: h + 2.0 * (pivot - l),
            s3: l - 2.0 * (h - pivot),
        });
    }
    Ok(result)
}

/// Calculate Fibonacci pivot points.
///
/// Uses the same central pivot as the standard variant, but Fibonacci
/// retracement ratios (38.2%, 61.8%, 100%) of the previous bar's range for
/// the support/resistance levels instead of the classic multiples.
///
/// # Formula
///
/// - Pivot = (High + Low + Close) / 3
/// - R1/S1 = Pivot ± 0.382×Range, R2/S2 = Pivot ± 0.618×Range, R3/S3 = Pivot ± 1.000×Range
///
/// # Example
///
/// ```
/// use finance_query::indicators::fibonacci_pivot_points;
///
/// let highs = vec![10.0, 12.0, 11.0];
/// let lows = vec![8.0, 9.0, 8.5];
/// let closes = vec![9.0, 11.0, 10.0];
/// let result = fibonacci_pivot_points(&highs, &lows, &closes).unwrap();
///
/// assert!(result[0].is_none());
/// assert!(result[1].is_some());
/// ```
pub fn fibonacci_pivot_points(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
) -> Result<Vec<Option<PivotPoints>>> {
    validate(highs, lows, closes)?;
    let mut result = vec![None; highs.len()];
    for i in 1..highs.len() {
        let (h, l, c) = (highs[i - 1], lows[i - 1], closes[i - 1]);
        let pivot = (h + l + c) / 3.0;
        let range = h - l;
        result[i] = Some(PivotPoints {
            pivot,
            r1: pivot + 0.382 * range,
            r2: pivot + 0.618 * range,
            r3: pivot + 1.000 * range,
            s1: pivot - 0.382 * range,
            s2: pivot - 0.618 * range,
            s3: pivot - 1.000 * range,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pivot_points_basic() {
        // Prior bar: H=12, L=8, C=10 -> pivot = 30/3 = 10.0
        let highs = vec![12.0, 15.0];
        let lows = vec![8.0, 9.0];
        let closes = vec![10.0, 13.0];
        let result = pivot_points(&highs, &lows, &closes).unwrap();

        assert!(result[0].is_none());
        let p = result[1].unwrap();
        assert!((p.pivot - 10.0).abs() < 1e-9);
        assert!((p.r1 - 12.0).abs() < 1e-9); // 2*10 - 8
        assert!((p.s1 - 8.0).abs() < 1e-9); // 2*10 - 12
        assert!((p.r2 - 14.0).abs() < 1e-9); // 10 + 4
        assert!((p.s2 - 6.0).abs() < 1e-9); // 10 - 4
        assert!(p.r3 > p.r2);
        assert!(p.s3 < p.s2);
    }

    #[test]
    fn test_fibonacci_pivot_points_basic() {
        let highs = vec![12.0, 15.0];
        let lows = vec![8.0, 9.0];
        let closes = vec![10.0, 13.0];
        let result = fibonacci_pivot_points(&highs, &lows, &closes).unwrap();

        assert!(result[0].is_none());
        let p = result[1].unwrap();
        assert!((p.pivot - 10.0).abs() < 1e-9);
        // range = 4.0
        assert!((p.r1 - (10.0 + 0.382 * 4.0)).abs() < 1e-9);
        assert!((p.s1 - (10.0 - 0.382 * 4.0)).abs() < 1e-9);
        assert!((p.r3 - 14.0).abs() < 1e-9);
        assert!((p.s3 - 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_pivot_points_insufficient_data() {
        assert!(pivot_points(&[1.0], &[1.0], &[1.0]).is_err());
    }

    #[test]
    fn test_pivot_points_mismatched_lengths() {
        assert!(pivot_points(&[1.0, 2.0], &[1.0], &[1.0, 2.0]).is_err());
    }
}
