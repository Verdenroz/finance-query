//! Elder Ray Index indicator.

pub use super::bull_bear_power::BullBearPowerResult as ElderRayResult;
use super::{Result, bull_bear_power::bull_bear_power};

/// Calculate Elder Ray Index.
///
/// Similar to Bull Bear Power.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
///
/// # Example
///
/// ```
/// use finance_query::indicators::elder_ray;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = elder_ray(&highs, &lows, &closes).unwrap();
/// ```
pub fn elder_ray(highs: &[f64], lows: &[f64], closes: &[f64]) -> Result<ElderRayResult> {
    bull_bear_power(highs, lows, closes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elder_ray() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = elder_ray(&highs, &lows, &closes).unwrap();

        assert_eq!(result.bull_power.len(), 20);
        assert!(result.bull_power[11].is_none());
        assert!(result.bull_power[12].is_some());
    }
}
