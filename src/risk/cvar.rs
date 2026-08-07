//! Conditional Value at Risk (CVaR / Expected Shortfall) calculations.
//!
//! CVaR is the average loss in the tail beyond the VaR threshold — a more
//! conservative tail-risk measure than VaR, which only reports the
//! threshold loss itself rather than the average loss beyond it.

use super::var::normal_quantile;

/// Compute historical CVaR (Expected Shortfall) at the given confidence level.
///
/// # Arguments
///
/// * `returns` - Daily log-returns or simple returns (as fractions, e.g. 0.02 = 2%)
/// * `confidence` - Confidence level, e.g. 0.95 for 95% CVaR
///
/// Returns `None` when `returns` is empty.
pub fn historical_cvar(returns: &[f64], confidence: f64) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    historical_cvar_sorted(&sorted, confidence)
}

/// `historical_cvar` over an already-sorted slice.
pub(crate) fn historical_cvar_sorted(sorted: &[f64], confidence: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let n = sorted.len();
    let idx = (((1.0 - confidence) * n as f64) as usize).min(n - 1);
    // Average of the tail up to and including the VaR threshold index.
    let tail = &sorted[..=idx];
    let mean_tail = tail.iter().sum::<f64>() / tail.len() as f64;
    Some(-mean_tail)
}

/// Compute parametric CVaR assuming normally distributed returns.
///
/// Uses the closed-form Expected Shortfall for a normal distribution:
/// `ES = -(mean - std_dev * phi(z) / (1 - confidence))`, where `phi` is the
/// standard normal density and `z` is the confidence level's quantile.
///
/// # Arguments
///
/// * `returns` - Daily returns as fractions
/// * `confidence` - Confidence level (0.95 or 0.99 are common)
///
/// Returns `None` when fewer than 2 observations are provided.
pub fn parametric_cvar(returns: &[f64], confidence: f64) -> Option<f64> {
    let (mean, std_dev) = super::ratios::mean_and_std(returns)?;
    Some(parametric_cvar_with_stats(mean, std_dev, confidence))
}

/// `parametric_cvar` given a precomputed mean and standard deviation.
pub(crate) fn parametric_cvar_with_stats(mean: f64, std_dev: f64, confidence: f64) -> f64 {
    let z = normal_quantile(confidence);
    let phi_z = (-(z * z) / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let es_factor = phi_z / (1.0 - confidence);
    -(mean - es_factor * std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_cvar_empty() {
        assert!(historical_cvar(&[], 0.95).is_none());
    }

    #[test]
    fn test_historical_cvar_simple() {
        // sorted: [-0.10, -0.05, 0.0, 0.05, 0.10]; 95% VaR idx = 0 -> tail = [-0.10]
        let returns = [-0.05_f64, 0.0, 0.05, 0.10, -0.10];
        let cvar = historical_cvar(&returns, 0.95).unwrap();
        assert!((cvar - 0.10).abs() < 1e-9, "got {cvar}");
    }

    #[test]
    fn test_historical_cvar_worse_than_var() {
        // CVaR (average of the tail) should be at least as large as VaR
        // (the single threshold loss) for a fat left tail.
        let returns: Vec<f64> = vec![
            -0.30, -0.20, -0.15, -0.10, -0.05, 0.0, 0.02, 0.03, 0.04, 0.05,
        ];
        let cvar = historical_cvar(&returns, 0.90).unwrap();
        let var = super::super::var::historical_var(&returns, 0.90).unwrap();
        assert!(cvar >= var, "CVaR ({cvar}) should be >= VaR ({var})");
    }

    #[test]
    fn test_parametric_cvar_positive_for_volatile_returns() {
        let returns: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        let cvar = parametric_cvar(&returns, 0.95).unwrap();
        assert!(cvar > 0.0, "CVaR must be positive");
    }

    #[test]
    fn test_parametric_cvar_worse_than_var() {
        let returns: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        let cvar = parametric_cvar(&returns, 0.95).unwrap();
        let var = super::super::var::parametric_var(&returns, 0.95).unwrap();
        assert!(
            cvar >= var,
            "parametric CVaR ({cvar}) should be >= VaR ({var})"
        );
    }
}
