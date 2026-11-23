//! Money Flow Index (MFI) indicator.

use super::{IndicatorError, Result};

/// Calculate Money Flow Index (MFI).
///
/// Volume-weighted RSI. Returns value between 0-100.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `volumes` - Volume data
/// * `period` - Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::mfi;
///
/// let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
/// let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
/// let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];
/// let volumes = vec![100.0, 200.0, 150.0, 100.0, 50.0];
/// let result = mfi(&highs, &lows, &closes, &volumes, 3).unwrap();
/// ```
pub fn mfi(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    volumes: &[f64],
    period: usize,
) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len || volumes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < period + 1 {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: len,
        });
    }

    let mut typical_prices = Vec::with_capacity(len);
    for i in 0..len {
        typical_prices.push((highs[i] + lows[i] + closes[i]) / 3.0);
    }

    let mut result = vec![None; len];

    let mut raw_money_flow = Vec::with_capacity(len);
    raw_money_flow.push(0.0);

    for i in 1..len {
        raw_money_flow.push(typical_prices[i] * volumes[i]);
    }

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    for i in 1..=period {
        if typical_prices[i] > typical_prices[i - 1] {
            positive_flow += raw_money_flow[i];
        } else if typical_prices[i] < typical_prices[i - 1] {
            negative_flow += raw_money_flow[i];
        }
    }

    if negative_flow == 0.0 {
        result[period] = Some(100.0);
    } else {
        let money_ratio = positive_flow / negative_flow;
        result[period] = Some(100.0 - (100.0 / (1.0 + money_ratio)));
    }

    for i in (period + 1)..len {
        let old_idx = i - period;
        if typical_prices[old_idx] > typical_prices[old_idx - 1] {
            positive_flow -= raw_money_flow[old_idx];
        } else if typical_prices[old_idx] < typical_prices[old_idx - 1] {
            negative_flow -= raw_money_flow[old_idx];
        }

        if typical_prices[i] > typical_prices[i - 1] {
            positive_flow += raw_money_flow[i];
        } else if typical_prices[i] < typical_prices[i - 1] {
            negative_flow += raw_money_flow[i];
        }

        if negative_flow == 0.0 {
            result[i] = Some(100.0);
        } else {
            let money_ratio = positive_flow / negative_flow;
            result[i] = Some(100.0 - (100.0 / (1.0 + money_ratio)));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfi() {
        let highs = vec![10.0, 11.0, 12.0, 11.0, 10.0];
        let lows = vec![8.0, 9.0, 10.0, 9.0, 8.0];
        let closes = vec![9.0, 10.0, 11.0, 10.0, 9.0];
        let volumes = vec![100.0, 200.0, 150.0, 100.0, 50.0];
        let result = mfi(&highs, &lows, &closes, &volumes, 3).unwrap();

        assert_eq!(result.len(), 5);
        assert!(result[2].is_none());
        assert!(result[3].is_some());
    }
}
