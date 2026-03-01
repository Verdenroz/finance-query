//! Ichimoku Cloud indicator.

use super::{IndicatorError, Result};
use serde::{Deserialize, Serialize};

/// Result of Ichimoku Cloud calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IchimokuResult {
    /// Conversion Line (Tenkan-sen)
    pub conversion_line: Vec<Option<f64>>,
    /// Base Line (Kijun-sen)
    pub base_line: Vec<Option<f64>>,
    /// Leading Span A (Senkou Span A)
    pub leading_span_a: Vec<Option<f64>>,
    /// Leading Span B (Senkou Span B)
    pub leading_span_b: Vec<Option<f64>>,
    /// Lagging Span (Chikou Span)
    pub lagging_span: Vec<Option<f64>>,
}

/// Calculate Ichimoku Cloud.
///
/// Returns all five Ichimoku lines. Leading Span B uses `2 * base` bars.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
/// * `conversion` - Conversion line (Tenkan-sen) period (default: 9)
/// * `base` - Base line (Kijun-sen) period; also controls cloud displacement (default: 26)
/// * `lagging` - Lagging span (Chikou Span) back-displacement in bars (default: 26)
/// * `displacement` - Cloud forward displacement in bars (default: 26)
///
/// # Example
///
/// ```
/// use finance_query::indicators::ichimoku;
///
/// let highs = vec![10.0; 100];
/// let lows = vec![8.0; 100];
/// let closes = vec![9.0; 100];
/// let result = ichimoku(&highs, &lows, &closes, 9, 26, 26, 26).unwrap();
/// ```
pub fn ichimoku(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    conversion: usize,
    base: usize,
    lagging: usize,
    displacement: usize,
) -> Result<IchimokuResult> {
    if conversion == 0 || base == 0 || lagging == 0 || displacement == 0 {
        return Err(IndicatorError::InvalidPeriod(
            "All periods must be greater than 0".to_string(),
        ));
    }
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    let span_b_period = 2 * base;
    let need = span_b_period.max(lagging);
    if len < need {
        return Err(IndicatorError::InsufficientData { need, got: len });
    }

    let mut conversion_line = vec![None; len];
    let mut base_line = vec![None; len];
    let mut leading_span_a = vec![None; len];
    let mut leading_span_b = vec![None; len];
    let mut lagging_span = vec![None; len];

    let midpoint = |h: &[f64], l: &[f64]| -> f64 {
        let highest = h.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = l.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        (highest + lowest) / 2.0
    };

    for i in 0..len {
        if i >= conversion - 1 {
            let start = i + 1 - conversion;
            conversion_line[i] = Some(midpoint(&highs[start..=i], &lows[start..=i]));
        }

        if i >= base - 1 {
            let start = i + 1 - base;
            base_line[i] = Some(midpoint(&highs[start..=i], &lows[start..=i]));
        }

        if i >= base - 1
            && let (Some(conv), Some(base_val)) = (conversion_line[i], base_line[i])
        {
            let val = (conv + base_val) / 2.0;
            if i + displacement < len {
                leading_span_a[i + displacement] = Some(val);
            }
        }

        if i >= span_b_period - 1 {
            let start = i + 1 - span_b_period;
            let val = midpoint(&highs[start..=i], &lows[start..=i]);
            if i + displacement < len {
                leading_span_b[i + displacement] = Some(val);
            }
        }

        if i >= lagging {
            lagging_span[i - lagging] = Some(closes[i]);
        }
    }

    Ok(IchimokuResult {
        conversion_line,
        base_line,
        leading_span_a,
        leading_span_b,
        lagging_span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ichimoku_defaults() {
        let highs = vec![10.0; 100];
        let lows = vec![8.0; 100];
        let closes = vec![9.0; 100];
        let result = ichimoku(&highs, &lows, &closes, 9, 26, 26, 26).unwrap();

        assert_eq!(result.conversion_line.len(), 100);
        assert!(result.conversion_line[8].is_some());
        assert!(result.base_line[25].is_some());
        assert!(result.leading_span_a[51].is_some()); // 25 + 26
        assert!(result.leading_span_b[77].is_some()); // 51 + 26
        assert!(result.lagging_span[0].is_some()); // 26 - 26
    }

    #[test]
    fn test_ichimoku_custom_periods() {
        let highs = vec![10.0; 100];
        let lows = vec![8.0; 100];
        let closes = vec![9.0; 100];
        // Custom: conversion=5, base=13, lagging=13, displacement=13
        let result = ichimoku(&highs, &lows, &closes, 5, 13, 13, 13).unwrap();
        assert!(result.conversion_line[4].is_some());
        assert!(result.base_line[12].is_some());
    }

    #[test]
    fn test_ichimoku_custom_produces_different_output() {
        let highs: Vec<f64> = (1..=100).map(|i| i as f64 + 1.0).collect();
        let lows: Vec<f64> = (1..=100).map(|i| i as f64 - 1.0).collect();
        let closes: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let default = ichimoku(&highs, &lows, &closes, 9, 26, 26, 26).unwrap();
        let custom = ichimoku(&highs, &lows, &closes, 5, 13, 13, 13).unwrap();
        let idx = 30;
        assert!(default.conversion_line[idx].is_some());
        assert!(custom.conversion_line[idx].is_some());
        assert_ne!(default.conversion_line[idx], custom.conversion_line[idx]);
    }
}
