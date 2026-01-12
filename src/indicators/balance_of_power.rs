//! Balance of Power (BOP) indicator.

use super::{IndicatorError, Result, sma::sma};

/// Calculate Balance of Power (BOP).
///
/// (Close - Open) / (High - Low)
///
/// # Arguments
///
/// * `opens` - Open prices
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `period` - Smoothing period (optional, default 14)
///
/// # Example
///
/// ```
/// use finance_query::indicators::balance_of_power;
///
/// let opens = vec![9.0; 10];
/// let highs = vec![10.0; 10];
/// let lows = vec![8.0; 10];
/// let closes = vec![9.5; 10];
/// let result = balance_of_power(&opens, &highs, &lows, &closes, Some(3)).unwrap();
/// ```
pub fn balance_of_power(
    opens: &[f64],
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    period: Option<usize>,
) -> Result<Vec<Option<f64>>> {
    let len = opens.len();
    if highs.len() != len || lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }

    let mut bop_raw = Vec::with_capacity(len);

    for i in 0..len {
        let high_low = highs[i] - lows[i];
        if high_low != 0.0 {
            bop_raw.push((closes[i] - opens[i]) / high_low);
        } else {
            bop_raw.push(0.0);
        }
    }

    if let Some(p) = period {
        let smoothed = sma(&bop_raw, p);
        Ok(smoothed)
    } else {
        Ok(bop_raw.into_iter().map(Some).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_of_power() {
        let opens = vec![9.0; 10];
        let highs = vec![10.0; 10];
        let lows = vec![8.0; 10];
        let closes = vec![9.5; 10];
        let result = balance_of_power(&opens, &highs, &lows, &closes, Some(3)).unwrap();

        assert_eq!(result.len(), 10);
        assert!(result[1].is_none());
        assert!(result[2].is_some());
    }
}
