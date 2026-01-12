//! Keltner Channels indicator.

use super::{IndicatorError, Result, atr::atr, ema::ema};
use serde::{Deserialize, Serialize};

/// Result of Keltner Channels calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeltnerChannelsResult {
    /// Upper channel
    pub upper: Vec<Option<f64>>,
    /// Middle channel (EMA)
    pub middle: Vec<Option<f64>>,
    /// Lower channel
    pub lower: Vec<Option<f64>>,
}

/// Calculate Keltner Channels.
///
/// Middle Line = EMA(period)
/// Upper Channel = EMA + (multiplier * ATR)
/// Lower Channel = EMA - (multiplier * ATR)
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - EMA period
/// * `atr_period` - ATR period
/// * `multiplier` - ATR multiplier
///
/// # Example
///
/// ```
/// use finance_query::indicators::keltner_channels;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = keltner_channels(&highs, &lows, &closes, 10, 10, 2.0).unwrap();
/// ```
pub fn keltner_channels(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
    atr_period: usize,
    multiplier: f64,
) -> Result<KeltnerChannelsResult> {
    if period == 0 || atr_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
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
    let atr_values = atr(highs, lows, closes, atr_period)?;

    let mut upper = vec![None; len];
    let mut middle = vec![None; len];
    let mut lower = vec![None; len];

    for i in 0..len {
        if let (Some(ema_val), Some(atr_val)) = (ema_values[i], atr_values[i]) {
            middle[i] = Some(ema_val);
            upper[i] = Some(ema_val + (multiplier * atr_val));
            lower[i] = Some(ema_val - (multiplier * atr_val));
        } else if let Some(ema_val) = ema_values[i] {
            middle[i] = Some(ema_val);
        }
    }

    Ok(KeltnerChannelsResult {
        upper,
        middle,
        lower,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_channels() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = keltner_channels(&highs, &lows, &closes, 10, 10, 2.0).unwrap();

        assert_eq!(result.upper.len(), 20);
        assert!(result.upper[8].is_none());
        assert!(result.upper[9].is_some());
    }
}
