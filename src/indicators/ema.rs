//! Exponential Moving Average (EMA) indicator.

use super::sma::sma;

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

    // Calculate initial SMA for the first EMA value
    let sma_values = sma(data, period);

    for (i, &sma_val) in sma_values.iter().enumerate() {
        match (sma_val, i) {
            (Some(sma), idx) if idx == period - 1 => {
                // First EMA value is the SMA
                result.push(Some(sma));
            }
            (_, idx) if idx < period - 1 => {
                // Not enough data yet
                result.push(None);
            }
            _ => {
                // Calculate EMA using previous EMA
                if let Some(prev_ema) = result.last().and_then(|&v| v) {
                    let ema_val = (data[i] - prev_ema) * multiplier + prev_ema;
                    result.push(Some(ema_val));
                } else {
                    result.push(None);
                }
            }
        }
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
