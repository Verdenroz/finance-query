//! Relative Strength Index (RSI) indicator.

use super::{IndicatorError, Result};

/// Internal RSI returning only valid values as plain `f64` (no `Option` wrapping, no padding).
/// Length = `data.len() - period`. Index `k` corresponds to original index `k + period`.
///
/// Inlines EMA of gains/losses to eliminate 4 intermediate `Vec` allocations.
pub(crate) fn rsi_raw(data: &[f64], period: usize) -> Result<Vec<f64>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() <= period {
        return Err(IndicatorError::InsufficientData {
            need: period + 1,
            got: data.len(),
        });
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let period_f = period as f64;

    // Seed: SMA of first `period` gains/losses
    let mut avg_gain = 0.0f64;
    let mut avg_loss = 0.0f64;
    for i in 1..=period {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            avg_gain += change;
        } else {
            avg_loss += change.abs();
        }
    }
    avg_gain /= period_f;
    avg_loss /= period_f;

    let n_valid = data.len() - period;
    let mut result = Vec::with_capacity(n_valid);
    result.push(if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    });

    // EMA smoothing on inline gain/loss (no intermediate Vec)
    for i in (period + 1)..data.len() {
        let change = data[i] - data[i - 1];
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { change.abs() } else { 0.0 };
        avg_gain = (gain - avg_gain) * multiplier + avg_gain;
        avg_loss = (loss - avg_loss) * multiplier + avg_loss;
        result.push(if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        });
    }

    Ok(result)
}

/// Calculate Relative Strength Index (RSI).
///
/// RSI measures the magnitude of recent price changes to evaluate overbought or oversold conditions.
/// Values range from 0 to 100, with readings above 70 indicating overbought and below 30 indicating oversold.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods (typically 14)
///
/// # Formula
///
/// 1. Calculate price changes (current - previous)
/// 2. Separate into gains (positive changes) and losses (negative changes, absolute value)
/// 3. Calculate average gain and average loss using EMA
/// 4. RS = Average Gain / Average Loss
/// 5. RSI = 100 - (100 / (1 + RS))
///
/// # Example
///
/// ```
/// use finance_query::indicators::rsi;
///
/// let prices = vec![44.0, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42,
///                   45.84, 46.08, 45.89, 46.03, 45.61, 46.28, 46.28];
/// let result = rsi(&prices, 14).unwrap();
///
/// // First 14 values will be None (need period + 1 for calculation)
/// assert!(result[13].is_none());
/// // RSI values start from index 14
/// assert!(result[14].is_some());
/// ```
pub fn rsi(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    let raw = rsi_raw(data, period)?;
    let mut result = vec![None; data.len()];
    for (k, v) in raw.into_iter().enumerate() {
        result[k + period] = Some(v);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        // Test data with known RSI values
        let data = vec![
            44.0, 44.34, 44.09, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03,
            45.61, 46.28, 46.28, 46.0,
        ];

        let result = rsi(&data, 14).unwrap();

        assert_eq!(result.len(), data.len());

        // First period values should be None
        for (i, &item) in result.iter().enumerate().take(14) {
            assert_eq!(item, None, "Index {} should be None", i);
        }

        // RSI should be between 0 and 100
        for (i, &val) in result.iter().enumerate().skip(14) {
            if let Some(rsi_val) = val {
                assert!(
                    (0.0..=100.0).contains(&rsi_val),
                    "RSI at index {} = {} is out of range [0, 100]",
                    i,
                    rsi_val
                );
            }
        }
    }

    #[test]
    fn test_rsi_all_gains() {
        // Steadily increasing prices should give high RSI
        let data: Vec<f64> = (0..30).map(|x| x as f64).collect();
        let result = rsi(&data, 14).unwrap();

        // Later RSI values should be close to 100
        if let Some(rsi_val) = result.last().and_then(|&v| v) {
            assert!(rsi_val > 90.0, "RSI with all gains should be > 90");
        }
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let data = vec![1.0, 2.0, 3.0];
        let result = rsi(&data, 14);

        assert!(result.is_err());
    }
}
