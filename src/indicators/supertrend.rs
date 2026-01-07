//! SuperTrend indicator.

use super::{IndicatorError, Result, atr::atr};
use serde::{Deserialize, Serialize};

/// Result of SuperTrend calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuperTrendResult {
    /// SuperTrend line
    pub value: Vec<Option<f64>>,
    /// Trend direction (true = up, false = down)
    pub is_uptrend: Vec<Option<bool>>,
}

/// Calculate SuperTrend.
///
/// Trend-following indicator based on ATR.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - ATR period
/// * `multiplier` - ATR multiplier
///
/// # Example
///
/// ```
/// use finance_query::indicators::supertrend;
///
/// let highs = vec![10.0; 20];
/// let lows = vec![8.0; 20];
/// let closes = vec![9.0; 20];
/// let result = supertrend(&highs, &lows, &closes, 10, 3.0).unwrap();
/// ```
pub fn supertrend(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: usize,
    multiplier: f64,
) -> Result<SuperTrendResult> {
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

    let atr_values = atr(highs, lows, closes, period)?;

    let mut supertrend = vec![None; len];
    let mut is_uptrend = vec![None; len];

    let start_idx = period - 1;

    let mut prev_final_upper = 0.0;
    let mut prev_final_lower = 0.0;
    let mut prev_trend = true;

    for i in start_idx..len {
        if let Some(atr_val) = atr_values[i] {
            let hl2 = (highs[i] + lows[i]) / 2.0;
            let basic_upper = hl2 + (multiplier * atr_val);
            let basic_lower = hl2 - (multiplier * atr_val);

            let current_close = closes[i];
            let prev_close = if i > 0 { closes[i - 1] } else { current_close };

            let final_upper = if i == start_idx
                || basic_upper < prev_final_upper
                || prev_close > prev_final_upper
            {
                basic_upper
            } else {
                prev_final_upper
            };

            let final_lower = if i == start_idx
                || basic_lower > prev_final_lower
                || prev_close < prev_final_lower
            {
                basic_lower
            } else {
                prev_final_lower
            };

            let trend = if i == start_idx {
                true
            } else if prev_trend && current_close <= final_lower {
                false
            } else if !prev_trend && current_close >= final_upper {
                true
            } else {
                prev_trend
            };

            let st_val = if trend { final_lower } else { final_upper };

            supertrend[i] = Some(st_val);
            is_uptrend[i] = Some(trend);

            prev_final_upper = final_upper;
            prev_final_lower = final_lower;
            prev_trend = trend;
        }
    }

    Ok(SuperTrendResult {
        value: supertrend,
        is_uptrend,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supertrend() {
        let highs = vec![10.0; 20];
        let lows = vec![8.0; 20];
        let closes = vec![9.0; 20];
        let result = supertrend(&highs, &lows, &closes, 10, 3.0).unwrap();

        assert_eq!(result.value.len(), 20);
        assert!(result.value[8].is_none());
        assert!(result.value[9].is_some());
    }
}
