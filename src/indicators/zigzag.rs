//! ZigZag indicator: filters out price moves smaller than a percentage
//! threshold, connecting only the significant swing highs and lows.

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// A single confirmed ZigZag swing point.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ZigZagPoint {
    /// Index into the original `highs`/`lows` slices where this swing occurred.
    pub index: usize,
    /// Price at the pivot (the high or low that qualified as a swing point).
    pub price: f64,
    /// `true` for a swing high, `false` for a swing low.
    pub is_high: bool,
}

/// Calculate ZigZag swing points from high/low series using a percentage
/// reversal threshold.
///
/// Starting from the first bar, price must move by at least `deviation_pct`
/// (e.g. `5.0` for 5%) away from the running extreme before a reversal is
/// confirmed and a pivot recorded. Consecutive pivots always alternate
/// between highs and lows. The final unconfirmed extreme (the most recent
/// swing-in-progress) is included as the last point.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `deviation_pct` - Minimum reversal size as a percentage (e.g. `5.0` = 5%)
///
/// # Example
///
/// ```
/// use finance_query::indicators::zigzag;
///
/// let highs = vec![100.0, 110.0, 90.0, 120.0, 80.0];
/// let lows = vec![100.0, 110.0, 90.0, 120.0, 80.0];
/// let pivots = zigzag(&highs, &lows, 5.0).unwrap();
///
/// assert_eq!(pivots.len(), 4);
/// assert!(pivots[0].is_high);
/// assert!(!pivots[1].is_high);
/// ```
pub fn zigzag(highs: &[f64], lows: &[f64], deviation_pct: f64) -> Result<Vec<ZigZagPoint>> {
    if deviation_pct <= 0.0 {
        return Err(IndicatorError::InvalidPeriod(
            "deviation_pct must be greater than 0".to_string(),
        ));
    }
    if highs.len() != lows.len() {
        return Err(IndicatorError::InvalidPeriod(
            "highs and lows must have the same length".to_string(),
        ));
    }
    if highs.is_empty() {
        return Err(IndicatorError::InsufficientData {
            need: 2,
            got: highs.len(),
        });
    }

    let threshold = deviation_pct / 100.0;
    let mut pivots = Vec::new();

    let start_price = (highs[0] + lows[0]) / 2.0;
    let mut trend: Option<bool> = None; // Some(true) = uptrend (tracking a high), Some(false) = downtrend
    let mut extreme_idx = 0usize;
    let mut extreme_price = start_price;

    for i in 1..highs.len() {
        match trend {
            None => {
                if highs[i] >= start_price * (1.0 + threshold) {
                    trend = Some(true);
                    extreme_idx = i;
                    extreme_price = highs[i];
                } else if lows[i] <= start_price * (1.0 - threshold) {
                    trend = Some(false);
                    extreme_idx = i;
                    extreme_price = lows[i];
                }
            }
            Some(true) => {
                if highs[i] > extreme_price {
                    extreme_price = highs[i];
                    extreme_idx = i;
                } else if lows[i] <= extreme_price * (1.0 - threshold) {
                    pivots.push(ZigZagPoint {
                        index: extreme_idx,
                        price: extreme_price,
                        is_high: true,
                    });
                    trend = Some(false);
                    extreme_price = lows[i];
                    extreme_idx = i;
                }
            }
            Some(false) => {
                if lows[i] < extreme_price {
                    extreme_price = lows[i];
                    extreme_idx = i;
                } else if highs[i] >= extreme_price * (1.0 + threshold) {
                    pivots.push(ZigZagPoint {
                        index: extreme_idx,
                        price: extreme_price,
                        is_high: false,
                    });
                    trend = Some(true);
                    extreme_price = highs[i];
                    extreme_idx = i;
                }
            }
        }
    }

    if let Some(is_high) = trend {
        pivots.push(ZigZagPoint {
            index: extreme_idx,
            price: extreme_price,
            is_high,
        });
    }

    Ok(pivots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_basic() {
        let highs = vec![100.0, 110.0, 90.0, 120.0, 80.0];
        let lows = vec![100.0, 110.0, 90.0, 120.0, 80.0];
        let pivots = zigzag(&highs, &lows, 5.0).unwrap();

        assert_eq!(pivots.len(), 4);
        assert_eq!(
            pivots[0],
            ZigZagPoint {
                index: 1,
                price: 110.0,
                is_high: true
            }
        );
        assert_eq!(
            pivots[1],
            ZigZagPoint {
                index: 2,
                price: 90.0,
                is_high: false
            }
        );
        assert_eq!(
            pivots[2],
            ZigZagPoint {
                index: 3,
                price: 120.0,
                is_high: true
            }
        );
        assert_eq!(
            pivots[3],
            ZigZagPoint {
                index: 4,
                price: 80.0,
                is_high: false
            }
        );
    }

    #[test]
    fn test_zigzag_alternates() {
        let highs = vec![100.0, 110.0, 90.0, 120.0, 80.0];
        let lows = vec![100.0, 110.0, 90.0, 120.0, 80.0];
        let pivots = zigzag(&highs, &lows, 5.0).unwrap();
        for w in pivots.windows(2) {
            assert_ne!(w[0].is_high, w[1].is_high, "pivots must alternate");
        }
    }

    #[test]
    fn test_zigzag_small_moves_filtered() {
        // Moves within the threshold shouldn't produce intermediate pivots.
        let highs = vec![100.0, 101.0, 100.5, 101.5, 130.0];
        let lows = vec![100.0, 100.5, 100.0, 101.0, 129.0];
        let pivots = zigzag(&highs, &lows, 10.0).unwrap();
        // Only the final large move to 130 should register (plus possibly the trend start).
        assert!(pivots.len() <= 2);
    }

    #[test]
    fn test_zigzag_invalid_deviation() {
        assert!(zigzag(&[1.0, 2.0], &[1.0, 2.0], 0.0).is_err());
        assert!(zigzag(&[1.0, 2.0], &[1.0, 2.0], -1.0).is_err());
    }

    #[test]
    fn test_zigzag_mismatched_lengths() {
        assert!(zigzag(&[1.0, 2.0], &[1.0], 5.0).is_err());
    }

    #[test]
    fn test_zigzag_empty() {
        assert!(zigzag(&[], &[], 5.0).is_err());
    }
}
