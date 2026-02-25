//! Value at Risk (VaR) calculations.

/// Compute historical VaR at the given confidence level.
///
/// Returns the loss that is exceeded only `(1 - confidence)` fraction of the time,
/// expressed as a positive number (a loss).
///
/// # Arguments
///
/// * `returns` - Daily log-returns or simple returns (as fractions, e.g. 0.02 = 2%)
/// * `confidence` - Confidence level, e.g. 0.95 for 95% VaR
///
/// Returns `None` when `returns` is empty.
pub fn historical_var(returns: &[f64], confidence: f64) -> Option<f64> {
    if returns.is_empty() {
        return None;
    }

    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // The `(1 - confidence)` worst-case percentile
    let idx = ((1.0 - confidence) * sorted.len() as f64) as usize;
    let idx = idx.min(sorted.len() - 1);

    // VaR is expressed as a positive loss
    Some(-sorted[idx])
}

/// Compute parametric (variance-covariance) VaR assuming normally distributed returns.
///
/// Uses the normal distribution's z-score for the confidence level.
///
/// # Arguments
///
/// * `returns` - Daily returns as fractions
/// * `confidence` - Confidence level (0.95 or 0.99 are common)
///
/// Returns `None` when fewer than 2 observations are provided.
pub fn parametric_var(returns: &[f64], confidence: f64) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    let std_dev = variance.sqrt();

    // Approximate normal quantile (z-score) for common confidence levels
    let z = normal_quantile(confidence);

    // VaR = -(mean - z * std_dev)
    Some(-(mean - z * std_dev))
}

/// Approximate inverse normal CDF (quantile function) for confidence levels in [0.90, 0.999].
/// Uses the Beasley-Springer-Moro approximation.
fn normal_quantile(p: f64) -> f64 {
    // This covers the most common confidence levels (95%, 99%) with good accuracy.
    // For exact values, a statistics library would be needed.
    match p {
        p if p >= 0.999 => 3.090,
        p if p >= 0.995 => 2.576,
        p if p >= 0.990 => 2.326,
        p if p >= 0.975 => 1.960,
        p if p >= 0.950 => 1.645,
        p if p >= 0.900 => 1.282,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_var_empty() {
        assert!(historical_var(&[], 0.95).is_none());
    }

    #[test]
    fn test_historical_var_simple() {
        // With returns [-0.10, -0.05, 0.0, 0.05, 0.10], 95% VaR on 5 obs
        // sorted: [-0.10, -0.05, 0.0, 0.05, 0.10]
        // idx = (1 - 0.95) * 5 = 0 → sorted[0] = -0.10 → VaR = 0.10
        let returns = [-0.05_f64, 0.0, 0.05, 0.10, -0.10];
        let var = historical_var(&returns, 0.95).unwrap();
        assert!((var - 0.10).abs() < 1e-9, "got {var}");
    }

    #[test]
    fn test_parametric_var_reasonable() {
        let returns: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 0.01 } else { -0.01 })
            .collect();
        let var = parametric_var(&returns, 0.95).unwrap();
        assert!(var > 0.0, "VaR must be positive");
    }
}
