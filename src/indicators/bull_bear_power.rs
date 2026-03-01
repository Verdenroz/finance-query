//! Bull Bear Power indicator.

use super::{IndicatorError, Result, ema::ema};
use serde::{Deserialize, Serialize};

/// Result of Bull Bear Power calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BullBearPowerResult {
    /// Bull Power
    pub bull_power: Vec<Option<f64>>,
    /// Bear Power
    pub bear_power: Vec<Option<f64>>,
}

/// Calculate Bull Bear Power.
///
/// Bull Power = High - EMA(period)
/// Bear Power = Low - EMA(period)
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
/// use finance_query::indicators::bull_bear_power;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = bull_bear_power(&highs, &lows, &closes, 13).unwrap();
/// ```
pub fn bull_bear_power(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
) -> Result<BullBearPowerResult> {
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
    if len < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: len,
        });
    }

    let ema_values = ema(closes, period);

    let mut bull_power = vec![None; len];
    let mut bear_power = vec![None; len];

    for i in 0..len {
        if let Some(ema_val) = ema_values[i] {
            bull_power[i] = Some(highs[i] - ema_val);
            bear_power[i] = Some(lows[i] - ema_val);
        }
    }

    Ok(BullBearPowerResult {
        bull_power,
        bear_power,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bull_bear_power_default_period() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = bull_bear_power(&highs, &lows, &closes, 13).unwrap();

        assert_eq!(result.bull_power.len(), 20);
        assert!(result.bull_power[11].is_none());
        assert!(result.bull_power[12].is_some());
    }

    #[test]
    fn test_bull_bear_power_custom_period() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = bull_bear_power(&highs, &lows, &closes, 5).unwrap();

        assert_eq!(result.bull_power.len(), 20);
        assert!(result.bull_power[3].is_none());
        assert!(result.bull_power[4].is_some());
    }

    #[test]
    fn test_bull_bear_power_custom_produces_different_output() {
        let highs: Vec<f64> = (1..=30).map(|i| i as f64 + 1.0).collect();
        let lows: Vec<f64> = (1..=30).map(|i| i as f64 - 1.0).collect();
        let closes: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let default = bull_bear_power(&highs, &lows, &closes, 13).unwrap();
        let custom = bull_bear_power(&highs, &lows, &closes, 5).unwrap();
        let idx = 14;
        assert!(default.bull_power[idx].is_some());
        assert!(custom.bull_power[idx].is_some());
        assert_ne!(default.bull_power[idx], custom.bull_power[idx]);
    }
}
