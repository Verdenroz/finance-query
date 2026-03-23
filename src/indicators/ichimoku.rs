//! Ichimoku Cloud indicator.

use std::collections::VecDeque;

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

    // Pass 1: conv + base + lagging in a single loop — 4 inline deques, no closure overhead.
    // Eliminates 2 intermediate Vec<f64> allocations vs the 3-pass closure approach.
    {
        let conv_off = conversion - 1;
        let base_off = base - 1;
        let mut conv_max: VecDeque<usize> = VecDeque::new();
        let mut conv_min: VecDeque<usize> = VecDeque::new();
        let mut base_max: VecDeque<usize> = VecDeque::new();
        let mut base_min: VecDeque<usize> = VecDeque::new();

        for i in 0..len {
            while conv_max.front().is_some_and(|&j| j + conversion <= i) {
                conv_max.pop_front();
            }
            while conv_min.front().is_some_and(|&j| j + conversion <= i) {
                conv_min.pop_front();
            }
            while base_max.front().is_some_and(|&j| j + base <= i) {
                base_max.pop_front();
            }
            while base_min.front().is_some_and(|&j| j + base <= i) {
                base_min.pop_front();
            }

            while conv_max.back().is_some_and(|&j| highs[j] <= highs[i]) {
                conv_max.pop_back();
            }
            while conv_min.back().is_some_and(|&j| lows[j] >= lows[i]) {
                conv_min.pop_back();
            }
            while base_max.back().is_some_and(|&j| highs[j] <= highs[i]) {
                base_max.pop_back();
            }
            while base_min.back().is_some_and(|&j| lows[j] >= lows[i]) {
                base_min.pop_back();
            }

            conv_max.push_back(i);
            conv_min.push_back(i);
            base_max.push_back(i);
            base_min.push_back(i);

            let conv_val = if i >= conv_off {
                let cv =
                    (highs[*conv_max.front().unwrap()] + lows[*conv_min.front().unwrap()]) / 2.0;
                conversion_line[i] = Some(cv);
                Some(cv)
            } else {
                None
            };

            if i >= base_off {
                let bv =
                    (highs[*base_max.front().unwrap()] + lows[*base_min.front().unwrap()]) / 2.0;
                base_line[i] = Some(bv);
                if let Some(cv) = conv_val
                    && i + displacement < len
                {
                    leading_span_a[i + displacement] = Some((cv + bv) / 2.0);
                }
            }

            if i >= lagging {
                lagging_span[i - lagging] = Some(closes[i]);
            }
        }
    }

    // Pass 2: span_b via 2 inline deques — writes directly to leading_span_b
    {
        let span_b_off = span_b_period - 1;
        let mut max_deque: VecDeque<usize> = VecDeque::new();
        let mut min_deque: VecDeque<usize> = VecDeque::new();

        for i in 0..len {
            while max_deque.front().is_some_and(|&j| j + span_b_period <= i) {
                max_deque.pop_front();
            }
            while min_deque.front().is_some_and(|&j| j + span_b_period <= i) {
                min_deque.pop_front();
            }
            while max_deque.back().is_some_and(|&j| highs[j] <= highs[i]) {
                max_deque.pop_back();
            }
            while min_deque.back().is_some_and(|&j| lows[j] >= lows[i]) {
                min_deque.pop_back();
            }
            max_deque.push_back(i);
            min_deque.push_back(i);
            if i >= span_b_off && i + displacement < len {
                let bv =
                    (highs[*max_deque.front().unwrap()] + lows[*min_deque.front().unwrap()]) / 2.0;
                leading_span_b[i + displacement] = Some(bv);
            }
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
