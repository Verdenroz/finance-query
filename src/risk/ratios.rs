//! Standalone risk-adjusted return ratios.
//!
//! These complement the Sharpe ratio already computed inside the backtesting engine,
//! providing access to these metrics without running a full backtest.

/// Compute the annualised Sharpe Ratio.
///
/// `Sharpe = (mean_return - risk_free_rate) / std_dev`, annualised by `sqrt(periods_per_year)`.
///
/// # Arguments
///
/// * `returns` - Per-period returns as fractions (e.g., daily returns)
/// * `risk_free_rate` - Risk-free rate **per period** (e.g., 0.0001 for daily ≈ 2.5% annual)
/// * `periods_per_year` - Trading periods in a year (252 for daily, 52 for weekly)
///
/// Returns `None` when fewer than 2 observations or standard deviation is zero.
pub fn sharpe_ratio(returns: &[f64], risk_free_rate: f64, periods_per_year: f64) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return None;
    }

    Some((mean - risk_free_rate) / std_dev * periods_per_year.sqrt())
}

/// Compute the annualised Sortino Ratio (penalises only downside volatility).
///
/// `Sortino = (mean_return - risk_free_rate) / downside_std`, annualised.
///
/// Returns `None` when fewer than 2 observations or downside deviation is zero.
pub fn sortino_ratio(returns: &[f64], risk_free_rate: f64, periods_per_year: f64) -> Option<f64> {
    if returns.len() < 2 {
        return None;
    }

    let mean = returns.iter().sum::<f64>() / returns.len() as f64;

    let downside_variance = returns
        .iter()
        .map(|r| {
            let diff = r - risk_free_rate;
            if diff < 0.0 { diff.powi(2) } else { 0.0 }
        })
        .sum::<f64>()
        / (returns.len() - 1) as f64;

    let downside_std = downside_variance.sqrt();

    if downside_std == 0.0 {
        return None;
    }

    Some((mean - risk_free_rate) / downside_std * periods_per_year.sqrt())
}

/// Compute the Calmar Ratio: annualised return divided by maximum drawdown.
///
/// # Arguments
///
/// * `total_return` - Cumulative return over the entire period (fraction)
/// * `years` - Length of the period in years
/// * `max_drawdown` - Maximum drawdown as a positive fraction (e.g., 0.30 = 30%)
///
/// Returns `None` when `max_drawdown` is zero.
pub fn calmar_ratio(total_return: f64, years: f64, max_drawdown: f64) -> Option<f64> {
    if max_drawdown == 0.0 || years <= 0.0 {
        return None;
    }
    let annualised = (1.0 + total_return).powf(1.0 / years) - 1.0;
    Some(annualised / max_drawdown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharpe_positive_returns() {
        let returns = vec![0.001_f64; 252];
        let s = sharpe_ratio(&returns, 0.0, 252.0).unwrap();
        assert!(s > 0.0, "Expected positive Sharpe, got {s}");
    }

    #[test]
    fn test_sortino_only_positive() {
        // All positive returns → downside std = 0 → None
        let returns = vec![0.01_f64; 252];
        assert!(sortino_ratio(&returns, 0.0, 252.0).is_none());
    }

    #[test]
    fn test_calmar_zero_drawdown() {
        assert!(calmar_ratio(0.20, 2.0, 0.0).is_none());
    }

    #[test]
    fn test_calmar_simple() {
        // 20% total over 2 years, 10% max drawdown
        // annualised ≈ 9.54%, Calmar ≈ 0.954
        let c = calmar_ratio(0.20, 2.0, 0.10).unwrap();
        assert!((c - 0.954).abs() < 0.01, "got {c}");
    }
}
