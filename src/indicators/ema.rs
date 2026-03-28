//! Exponential Moving Average (EMA) indicator.

/// Internal EMA returning only the valid values as plain `f64` (no `Option` wrapping,
/// no leading `None` padding). The returned `Vec<f64>` has length `data.len() - period + 1`,
/// where index 0 corresponds to original index `period - 1`.
///
/// Use this inside other indicator computations to avoid the `Option<f64>` overhead.
pub(crate) fn ema_raw(data: &[f64], period: usize) -> Vec<f64> {
    if period == 0 || data.is_empty() || data.len() < period {
        return Vec::new();
    }
    let multiplier = 2.0 / (period as f64 + 1.0);
    let initial: f64 = data[..period].iter().sum::<f64>() / period as f64;
    let mut result = Vec::with_capacity(data.len() - period + 1);
    result.push(initial);
    let mut prev = initial;
    for &price in &data[period..] {
        let val = (price - prev) * multiplier + prev;
        result.push(val);
        prev = val;
    }
    result
}

/// Calculate Exponential Moving Average (EMA).
///
/// EMA gives more weight to recent prices, making it more responsive than SMA.
/// The first value is calculated as an SMA, then subsequent values use the EMA formula.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods for the moving average
///
/// # Formula
///
/// - First EMA = SMA(period)
/// - Multiplier = 2 / (period + 1)
/// - EMA = (Close - Previous EMA) × Multiplier + Previous EMA
///
/// # Example
///
/// ```
/// use finance_query::indicators::ema;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let result = ema(&prices, 3);
///
/// // First 2 values are None (insufficient data)
/// assert!(result[0].is_none());
/// assert!(result[1].is_none());
/// // Subsequent values are calculated using EMA formula
/// assert!(result[2].is_some());
/// ```
pub fn ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    if period == 0 || data.is_empty() || data.len() < period {
        return vec![None; data.len()];
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut result = Vec::with_capacity(data.len());

    // First EMA value is the SMA of the first `period` elements — computed inline
    // (avoids calling sma() which would allocate a Vec<Option<f64>> of size N)
    let initial: f64 = data[..period].iter().sum::<f64>() / period as f64;

    result.extend(std::iter::repeat_n(None, period - 1));
    result.push(Some(initial));

    let mut prev = initial;
    for &price in &data[period..] {
        let val = (price - prev) * multiplier + prev;
        result.push(Some(val));
        prev = val;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema(&data, 3);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert!(result[2].is_some());
        assert!(result[3].is_some());
        assert!(result[4].is_some());

        // EMA should be more responsive to recent price changes
        // Later values should be closer to actual price than SMA
    }

    #[test]
    fn test_ema_period_1() {
        let data = vec![10.0, 20.0, 30.0];
        let result = ema(&data, 1);

        // Period 1 EMA should equal the price itself
        assert_eq!(result[0], Some(10.0));
        assert_eq!(result[1], Some(20.0));
        assert_eq!(result[2], Some(30.0));
    }

    #[test]
    fn test_ema_insufficient_data() {
        let data = vec![1.0, 2.0];
        let result = ema(&data, 5);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
    }
}
