//! Elder Ray Index indicator.

pub use super::bull_bear_power::BullBearPowerResult as ElderRayResult;
use super::{Result, bull_bear_power::bull_bear_power};

/// Calculate Elder Ray Index.
///
/// Similar to Bull Bear Power; uses EMA of the given period.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - EMA period (default: 13)
///
/// # Example
///
/// ```
/// use finance_query::indicators::elder_ray;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = elder_ray(&highs, &lows, &closes, 13).unwrap();
/// ```
pub fn elder_ray(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Result<ElderRayResult> {
    bull_bear_power(highs, lows, closes, period)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elder_ray_default_period() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = elder_ray(&highs, &lows, &closes, 13).unwrap();

        assert_eq!(result.bull_power.len(), 20);
        assert!(result.bull_power[11].is_none());
        assert!(result.bull_power[12].is_some());
    }

    #[test]
    fn test_elder_ray_custom_period() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = elder_ray(&highs, &lows, &closes, 5).unwrap();
        assert!(result.bull_power[3].is_none());
        assert!(result.bull_power[4].is_some());
    }
}
