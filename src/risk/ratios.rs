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
    let (mean, std_dev) = mean_and_std(returns)?;
    sharpe_with_stats(mean, std_dev, risk_free_rate, periods_per_year)
}

/// Sample mean and standard deviation (n-1 denominator).
pub(crate) fn mean_and_std(returns: &[f64]) -> Option<(f64, f64)> {
    if returns.len() < 2 {
        return None;
    }
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
    Some((mean, variance.sqrt()))
}

/// `sharpe_ratio` given a precomputed mean and standard deviation.
pub(crate) fn sharpe_with_stats(
    mean: f64,
    std_dev: f64,
    risk_free_rate: f64,
    periods_per_year: f64,
) -> Option<f64> {
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

/// Compute the Omega Ratio at a `0.0` threshold: probability-weighted ratio
/// of gains to losses over the full return distribution.
///
/// `Σ max(r, 0) / Σ max(-r, 0)`. More general than Sharpe — considers the
/// full return distribution rather than only mean and standard deviation.
/// Returns `f64::MAX` when there are no negative returns, `0.0` when there
/// are also no positive returns.
pub fn omega_ratio(returns: &[f64]) -> f64 {
    crate::perf_metrics::omega_ratio(returns)
}

/// Compute the Kelly Criterion: optimal fraction of capital to risk, given a
/// win rate and average win/loss magnitudes (in percent).
///
/// `W - (1 - W) / R` where `R = avg_win_pct / abs(avg_loss_pct)`. Returns
/// `f64::MAX` when there are no losses and wins are positive (unbounded
/// edge), `0.0` for other degenerate inputs.
///
/// Use [`win_loss_stats`] to derive `win_rate`/`avg_win_pct`/`avg_loss_pct`
/// from a plain return series (treating each positive-return period as a
/// "win" and each negative-return period as a "loss").
pub fn kelly_criterion(win_rate: f64, avg_win_pct: f64, avg_loss_pct: f64) -> f64 {
    crate::perf_metrics::kelly_criterion(win_rate, avg_win_pct, avg_loss_pct)
}

/// Compute the Ulcer Index: root-mean-square of drawdown depth across a
/// return series, expressed as a percentage (0–100).
///
/// Unlike [`max_drawdown`](super::max_drawdown), penalises both depth and
/// duration of drawdowns — a long shallow drawdown scores higher than a
/// brief deep one.
pub fn ulcer_index(returns: &[f64]) -> f64 {
    crate::perf_metrics::ulcer_index(&super::drawdown::drawdown_series(returns))
}

/// Compute the Information Ratio vs a benchmark: annualised mean excess
/// return divided by tracking error.
///
/// Returns `None` when the series differ in length, fewer than 2 aligned
/// observations are available, or tracking error is zero.
pub fn information_ratio(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<f64> {
    crate::perf_metrics::information_ratio(asset_returns, benchmark_returns, periods_per_year)
}

/// Compute the tracking error vs a benchmark: annualised standard deviation
/// of (asset − benchmark) periodic returns.
///
/// Returns `None` when the series differ in length or fewer than 2 aligned
/// observations are available.
pub fn tracking_error(
    asset_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<f64> {
    crate::perf_metrics::tracking_error(asset_returns, benchmark_returns, periods_per_year)
}

/// Derive win-rate and average win/loss percentages from a return series,
/// treating each period as if it were a discrete "trade" (a positive-return
/// period is a win, a negative-return period is a loss) — the natural
/// analogue of backtesting trade statistics for a plain returns series with
/// no explicit trade log. Feeds [`kelly_criterion`].
///
/// Returns `(win_rate, avg_win_pct, avg_loss_pct)`, all `0.0` for an empty
/// series.
pub fn win_loss_stats(returns: &[f64]) -> (f64, f64, f64) {
    let total = returns.len();
    if total == 0 {
        return (0.0, 0.0, 0.0);
    }

    let wins: Vec<f64> = returns.iter().copied().filter(|&r| r > 0.0).collect();
    let losses: Vec<f64> = returns.iter().copied().filter(|&r| r < 0.0).collect();

    let win_rate = wins.len() as f64 / total as f64;
    let avg_win_pct = if wins.is_empty() {
        0.0
    } else {
        wins.iter().sum::<f64>() / wins.len() as f64 * 100.0
    };
    let avg_loss_pct = if losses.is_empty() {
        0.0
    } else {
        losses.iter().sum::<f64>() / losses.len() as f64 * 100.0
    };

    (win_rate, avg_win_pct, avg_loss_pct)
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
