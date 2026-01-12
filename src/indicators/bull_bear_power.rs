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
/// Bull Power = High - EMA(13)
/// Bear Power = Low - EMA(13)
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
/// use finance_query::indicators::bull_bear_power;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = bull_bear_power(&highs, &lows, &closes).unwrap();
/// ```
pub fn bull_bear_power(highs: &[f64], lows: &[f64], closes: &[f64]) -> Result<BullBearPowerResult> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 13 {
        return Err(IndicatorError::InsufficientData { need: 13, got: len });
    }

    let ema13 = ema(closes, 13);

    let mut bull_power = vec![None; len];
    let mut bear_power = vec![None; len];

    for i in 0..len {
        if let Some(ema_val) = ema13[i] {
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
    fn test_bull_bear_power() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = bull_bear_power(&highs, &lows, &closes).unwrap();

        assert_eq!(result.bull_power.len(), 20);
        assert!(result.bull_power[11].is_none());
        assert!(result.bull_power[12].is_some());
    }
}
