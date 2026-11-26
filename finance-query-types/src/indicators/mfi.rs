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

    let typical_prices: Vec<f64> = highs
        .iter()
        .zip(lows)
        .zip(closes)
        .map(|((h, l), c)| (h + l + c) / 3.0)
        .collect();

    let flows = || {
        typical_prices
            .windows(2)
            .zip(&volumes[1..])
            .map(|(w, &v)| (w[1] > w[0], w[1] < w[0], w[1] * v))
    };

    let mut result = vec![None; len];

    let mut positive_flow = 0.0;
    let mut negative_flow = 0.0;

    for (up, down, raw) in flows().take(period) {
        if up {
            positive_flow += raw;
        } else if down {
            negative_flow += raw;
        }
    }
    result[period] = Some(money_flow_index(positive_flow, negative_flow));

    for ((out, (old_up, old_down, old_raw)), (up, down, raw)) in result[period + 1..]
        .iter_mut()
        .zip(flows())
        .zip(flows().skip(period))
    {
        if old_up {
            positive_flow -= old_raw;
        } else if old_down {
            negative_flow -= old_raw;
        }

        if up {
            positive_flow += raw;
        } else if down {
            negative_flow += raw;
        }

        *out = Some(money_flow_index(positive_flow, negative_flow));
    }

    Ok(result)
}

fn money_flow_index(positive_flow: f64, negative_flow: f64) -> f64 {
    if negative_flow == 0.0 {
        100.0
    } else {
        let money_ratio = positive_flow / negative_flow;
        100.0 - (100.0 / (1.0 + money_ratio))
    }
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
