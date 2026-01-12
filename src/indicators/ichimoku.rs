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
/// Returns all five Ichimoku lines.
///
/// # Arguments
///
/// * `highs` - High prices
/// * `lows` - Low prices
/// * `closes` - Close prices
///
/// # Example
///
/// ```
/// use finance_query::indicators::ichimoku;
///
/// let highs = vec![10.0; 100];
/// let lows = vec![8.0; 100];
/// let closes = vec![9.0; 100];
/// let result = ichimoku(&highs, &lows, &closes).unwrap();
/// ```
pub fn ichimoku(highs: &[f64], lows: &[f64], closes: &[f64]) -> Result<IchimokuResult> {
    let len = highs.len();
    if lows.len() != len || closes.len() != len {
        return Err(IndicatorError::InvalidPeriod(
            "Data lengths must match".to_string(),
        ));
    }
    if len < 52 {
        return Err(IndicatorError::InsufficientData { need: 52, got: len });
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
        if i >= 8 {
            let start = i - 8;
            conversion_line[i] = Some(midpoint(&highs[start..=i], &lows[start..=i]));
        }

        if i >= 25 {
            let start = i - 25;
            base_line[i] = Some(midpoint(&highs[start..=i], &lows[start..=i]));
        }

        if i >= 25
            && let (Some(conv), Some(base)) = (conversion_line[i], base_line[i])
        {
            let val = (conv + base) / 2.0;
            if i + 26 < len {
                leading_span_a[i + 26] = Some(val);
            }
        }

        if i >= 51 {
            let start = i - 51;
            let val = midpoint(&highs[start..=i], &lows[start..=i]);
            if i + 26 < len {
                leading_span_b[i + 26] = Some(val);
            }
        }

        if i >= 26 {
            lagging_span[i - 26] = Some(closes[i]);
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
    fn test_ichimoku() {
        let highs = vec![10.0; 100];
        let lows = vec![8.0; 100];
        let closes = vec![9.0; 100];
        let result = ichimoku(&highs, &lows, &closes).unwrap();

        assert_eq!(result.conversion_line.len(), 100);
        assert!(result.conversion_line[8].is_some());
        assert!(result.base_line[25].is_some());
        assert!(result.leading_span_a[51].is_some()); // 25 + 26
        assert!(result.leading_span_b[77].is_some()); // 51 + 26
        assert!(result.lagging_span[0].is_some()); // 26 - 26
    }
}
