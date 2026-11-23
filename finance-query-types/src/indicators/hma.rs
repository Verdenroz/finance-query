//! Hull Moving Average (HMA) indicator.

use super::{IndicatorError, Result, wma::wma_raw};

/// Calculate Hull Moving Average (HMA).
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
/// Responsive moving average with reduced lag.
///
/// # Arguments
///
/// * `data` - Price data (typically close prices)
/// * `period` - Number of periods
///
/// # Formula
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
///
/// # Example
///
/// ```
/// use finance_query::indicators::hma;
///
/// let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
/// let result = hma(&prices, 4).unwrap();
/// ```
pub fn hma(data: &[f64], period: usize) -> Result<Vec<Option<f64>>> {
    if period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Period must be greater than 0".to_string(),
        ));
    }
    if data.len() < period {
        return Err(IndicatorError::InsufficientData {
            need: period,
            got: data.len(),
        });
    }

    let half_period = period / 2;
    let sqrt_period = (period as f64).sqrt() as usize;

    if sqrt_period == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "Sqrt period is 0".to_string(),
        ));
    }

    // wma_raw eliminates 2 × Vec<Option<f64>>(N) and the Option-check collection loop
    let half_raw = wma_raw(data, half_period); // len = N - (half_period - 1)
    let full_raw = wma_raw(data, period); // len = N - (period - 1)

    // Align: full_raw[k] → orig k + period-1; half_raw[k + shift] → same orig index
    let shift = period - half_period;
    let diff: Vec<f64> = full_raw
        .iter()
        .enumerate()
        .map(|(k, &fv)| 2.0 * half_raw[k + shift] - fv)
        .collect();

    if diff.len() < sqrt_period {
        return Err(IndicatorError::InsufficientData {
            need: period + sqrt_period,
            got: data.len(),
        });
    }

    let hma_raw = wma_raw(&diff, sqrt_period);

    // hma_raw[k] → diff[k + sqrt_period - 1] → orig k + period - 1 + sqrt_period - 1
    let mut result = vec![None; data.len()];
    let base = period + sqrt_period - 2;
    for (k, &v) in hma_raw.iter().enumerate() {
        let orig = k + base;
        if orig < data.len() {
            result[orig] = Some(v);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hma() {
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0];
        let result = hma(&prices, 4).unwrap();

        assert_eq!(result.len(), prices.len());
        // period=4, half=2, sqrt=2
        // wma_half valid from index 1
        // wma_full valid from index 3
        // diff valid from index 3
        // hma (wma of diff, period 2) valid from index 3 + (2-1) = 4

        assert!(result[0].is_none());
        assert!(result[3].is_none());
        assert!(result[4].is_some());
    }
}
