//! Simple Moving Average (SMA) indicator.

/// Calculate Simple Moving Average (SMA).
///
/// Returns a vector where each element is the average of the previous `period` values.
/// The first `period - 1` elements will be `None` since there's insufficient data.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods for the moving average
///
/// # Formula
///
/// SMA = (P1 + P2 + ... + Pn) / n
///
/// Where:
/// - P = Price at each period
/// - n = Number of periods
///
/// # Example
///
/// ```
/// use finance_query::indicators::sma;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];
/// let result = sma(&prices, 3);
///
/// // First 2 values are None (insufficient data)
/// assert!(result[0].is_none());
/// assert!(result[1].is_none());
/// // Third value: (10 + 11 + 12) / 3 = 11.0
/// assert_eq!(result[2], Some(11.0));
/// ```
pub fn sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    if period == 0 || data.is_empty() {
        return vec![None; data.len()];
    }

    let mut result = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let sum: f64 = data[i + 1 - period..=i].iter().sum();
            result.push(Some(sum / period as f64));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(2.0)); // (1+2+3)/3 = 2
        assert_eq!(result[3], Some(3.0)); // (2+3+4)/3 = 3
        assert_eq!(result[4], Some(4.0)); // (3+4+5)/3 = 4
    }

    #[test]
    fn test_sma_period_1() {
        let data = vec![10.0, 20.0, 30.0];
        let result = sma(&data, 1);

        assert_eq!(result[0], Some(10.0));
        assert_eq!(result[1], Some(20.0));
        assert_eq!(result[2], Some(30.0));
    }

    #[test]
    fn test_sma_empty_data() {
        let data: Vec<f64> = vec![];
        let result = sma(&data, 5);

        assert!(result.is_empty());
    }
}
