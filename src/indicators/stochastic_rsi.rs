//! Stochastic RSI indicator.

use super::{IndicatorError, Result, rsi::rsi};

/// Calculate Stochastic RSI.
///
/// Applies Stochastic formula to RSI values.
/// Returns value between 0-100.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `rsi_period` - Period for RSI
/// * `stoch_period` - Period for Stochastic calculation on RSI
///
/// # Example
///
/// ```
/// use finance_query::indicators::stochastic_rsi;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
/// let result = stochastic_rsi(&prices, 3, 3).unwrap();
/// ```
pub fn stochastic_rsi(
    data: &[f64],
    rsi_period: usize,
    stoch_period: usize,
) -> Result<Vec<Option<f64>>> {
    if rsi_period == 0 || stoch_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Periods must be greater than 0".to_string(),
        ));
    }
    if data.len() < rsi_period + stoch_period {
        return Err(IndicatorError::InsufficientData {
            need: rsi_period + stoch_period,
            got: data.len(),
        });
    }

    let rsi_values = rsi(data, rsi_period)?;

    let mut result = vec![None; data.len()];

    for (i, item) in result
        .iter_mut()
        .enumerate()
        .skip(rsi_period + stoch_period - 1)
    {
        let start_idx = i + 1 - stoch_period;
        let end_idx = i;

        let mut min_rsi = f64::INFINITY;
        let mut max_rsi = f64::NEG_INFINITY;
        let mut current_rsi = 0.0;
        let mut valid = true;

        for (j, rsi_val) in rsi_values
            .iter()
            .enumerate()
            .skip(start_idx)
            .take(stoch_period)
        {
            if let Some(val) = rsi_val {
                if *val < min_rsi {
                    min_rsi = *val;
                }
                if *val > max_rsi {
                    max_rsi = *val;
                }
                if j == end_idx {
                    current_rsi = *val;
                }
            } else {
                valid = false;
                break;
            }
        }

        if valid {
            let range = max_rsi - min_rsi;
            let stoch_rsi = if range == 0.0 {
                50.0
            } else {
                ((current_rsi - min_rsi) / range) * 100.0
            };
            *item = Some(stoch_rsi);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stochastic_rsi() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
        let result = stochastic_rsi(&prices, 3, 3).unwrap();

        assert_eq!(result.len(), 9);
        // RSI valid from index 3
        // StochRSI valid from index 3 + 3 - 1 = 5

        assert!(result[0].is_none());
        assert!(result[4].is_none());
        assert!(result[5].is_some());
    }
}
