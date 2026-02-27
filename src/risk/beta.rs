//! Beta calculation against a benchmark.

/// Compute the beta of an asset relative to a benchmark.
///
/// `β = Cov(asset, benchmark) / Var(benchmark)`
///
/// Both slices must have the same length and at least 2 observations.
/// Returns `None` on insufficient data or zero benchmark variance.
pub fn beta(asset_returns: &[f64], benchmark_returns: &[f64]) -> Option<f64> {
    let n = asset_returns.len();
    if n < 2 || n != benchmark_returns.len() {
        return None;
    }

    let asset_mean = asset_returns.iter().sum::<f64>() / n as f64;
    let bench_mean = benchmark_returns.iter().sum::<f64>() / n as f64;

    let covariance: f64 = asset_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(a, b)| (a - asset_mean) * (b - bench_mean))
        .sum::<f64>()
        / (n - 1) as f64;

    let bench_variance: f64 = benchmark_returns
        .iter()
        .map(|b| (b - bench_mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;

    if bench_variance == 0.0 {
        return None;
    }

    Some(covariance / bench_variance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beta_identical() {
        // Asset = benchmark → beta = 1.0
        let r = vec![0.01, -0.02, 0.03, -0.01, 0.02];
        let b = beta(&r, &r).unwrap();
        assert!((b - 1.0).abs() < 1e-9, "expected 1.0, got {b}");
    }

    #[test]
    fn test_beta_zero_benchmark_variance() {
        let asset = vec![0.01, 0.02, 0.03];
        let bench = vec![0.01, 0.01, 0.01]; // constant → variance = 0
        assert!(beta(&asset, &bench).is_none());
    }

    #[test]
    fn test_beta_length_mismatch() {
        assert!(beta(&[0.01], &[0.01, 0.02]).is_none());
    }

    #[test]
    fn test_beta_inverse() {
        // Asset = -benchmark → beta = -1.0
        let bench = vec![0.01, -0.02, 0.03, -0.01];
        let asset: Vec<f64> = bench.iter().map(|x| -x).collect();
        let b = beta(&asset, &bench).unwrap();
        assert!((b + 1.0).abs() < 1e-9, "expected -1.0, got {b}");
    }
}
