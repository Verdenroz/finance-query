//! Shared performance-metric math for `risk` and `backtesting`.
//!
//! `risk` and `backtesting` are independent optional features — `backtesting`
//! does not (and should not) depend on `risk` — but both already imply
//! `indicators`. Formulas that both modules need (Omega Ratio, Kelly
//! Criterion, Ulcer Index, Information Ratio, tracking error) live here,
//! gated on `any(risk, backtesting)`, so extracting the duplication doesn't
//! saddle either feature with a dependency on the other.
//!
//! Everything here is pure math over plain `f64` slices — no `Candle`,
//! `EquityPoint`, or `Trade` types — so both call sites can adapt their own
//! domain types to these signatures without this module depending on either.

/// Kelly Criterion: optimal fraction of capital to risk, given a win rate and
/// average win/loss magnitudes (in percent).
///
/// `W - (1 - W) / R` where `R = avg_win_pct / abs(avg_loss_pct)`.
///
/// Returns `f64::MAX` when there are no losses and wins are positive
/// (unbounded edge), `0.0` for other degenerate inputs (no wins, or a zero
/// win rate).
pub(crate) fn kelly_criterion(win_rate: f64, avg_win_pct: f64, avg_loss_pct: f64) -> f64 {
    let abs_loss = avg_loss_pct.abs();
    if abs_loss == 0.0 {
        return if avg_win_pct > 0.0 { f64::MAX } else { 0.0 };
    }
    if avg_win_pct == 0.0 {
        return 0.0;
    }
    let r = avg_win_pct / abs_loss;
    win_rate - (1.0 - win_rate) / r
}

/// Omega Ratio at a `0.0` threshold: `Σ max(r, 0) / Σ max(-r, 0)`.
///
/// More general than Sharpe — considers the full return distribution rather
/// than only mean and standard deviation. Returns `f64::MAX` when there are
/// no negative returns, `0.0` when there are also no positive returns.
pub(crate) fn omega_ratio(returns: &[f64]) -> f64 {
    let gains: f64 = returns.iter().map(|&r| r.max(0.0)).sum();
    let losses: f64 = returns.iter().map(|&r| (-r).max(0.0)).sum();
    if losses == 0.0 {
        if gains > 0.0 { f64::MAX } else { 0.0 }
    } else {
        gains / losses
    }
}

/// Ulcer Index: `sqrt(mean((drawdown_pct × 100)²))` given per-period drawdown
/// fractions (0.0–1.0), returned in **percentage** units (0–100) to match
/// backtesting.py and Peter Martin's original 1987 definition.
///
/// Unlike max drawdown, penalises both depth and duration — a long shallow
/// drawdown scores higher than a brief deep one.
pub(crate) fn ulcer_index(drawdown_pcts: &[f64]) -> f64 {
    if drawdown_pcts.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = drawdown_pcts.iter().map(|d| (d * 100.0).powi(2)).sum();
    (sum_sq / drawdown_pcts.len() as f64).sqrt()
}

/// Mean and sample standard deviation (n-1) of the excess-return series
/// (`strategy - benchmark`, aligned pairwise). `None` when the series differ
/// in length or fewer than 2 observations are available.
fn excess_stats(strategy_returns: &[f64], benchmark_returns: &[f64]) -> Option<(f64, f64)> {
    let n = strategy_returns.len();
    if n < 2 || n != benchmark_returns.len() {
        return None;
    }
    let excess: Vec<f64> = strategy_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(s, b)| s - b)
        .collect();
    let mean = excess.iter().sum::<f64>() / n as f64;
    let variance = excess.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    Some((mean, variance.sqrt()))
}

/// Tracking error: annualised standard deviation of (strategy − benchmark)
/// periodic returns.
///
/// `None` when the series differ in length or fewer than 2 aligned
/// observations are available.
pub(crate) fn tracking_error(
    strategy_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<f64> {
    excess_stats(strategy_returns, benchmark_returns).map(|(_, std)| std * periods_per_year.sqrt())
}

/// Information Ratio: annualised mean excess return divided by tracking
/// error (annualised standard deviation of excess returns).
///
/// `None` when the series differ in length, fewer than 2 aligned
/// observations are available, or tracking error is zero.
pub(crate) fn information_ratio(
    strategy_returns: &[f64],
    benchmark_returns: &[f64],
    periods_per_year: f64,
) -> Option<f64> {
    let (mean, std) = excess_stats(strategy_returns, benchmark_returns)?;
    if std > 0.0 {
        Some((mean / std) * periods_per_year.sqrt())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelly_criterion() {
        // W=0.6, avg_win=10%, avg_loss=5% => R=2.0 => Kelly=0.6 - 0.4/2 = 0.4
        let kelly = kelly_criterion(0.6, 10.0, -5.0);
        assert!(
            (kelly - 0.4).abs() < 1e-9,
            "Kelly should be 0.4, got {kelly}"
        );

        // No losses, positive wins -> unbounded edge
        assert_eq!(kelly_criterion(1.0, 10.0, 0.0), f64::MAX);
        // No losses, no wins -> degenerate
        assert_eq!(kelly_criterion(0.0, 0.0, 0.0), 0.0);
    }

    #[test]
    fn test_omega_ratio() {
        assert_eq!(omega_ratio(&[1.0, 2.0, 3.0]), f64::MAX);
        assert_eq!(omega_ratio(&[-1.0, -2.0, -3.0]), 0.0);
        let omega = omega_ratio(&[2.0, -1.0, 3.0, -2.0]);
        assert!((omega - 5.0 / 3.0).abs() < 1e-9, "got {omega}");
    }

    #[test]
    fn test_ulcer_index() {
        assert_eq!(ulcer_index(&[]), 0.0);
        assert_eq!(ulcer_index(&[0.0, 0.0, 0.0]), 0.0);
        // drawdowns of 10% and 20% -> sqrt(mean(10^2, 20^2)) = sqrt(250) ≈ 15.81
        let ui = ulcer_index(&[0.10, 0.20]);
        assert!((ui - 250f64.sqrt()).abs() < 1e-9, "got {ui}");
    }

    #[test]
    fn test_tracking_error_and_information_ratio() {
        let strategy = vec![0.01, 0.02, -0.01, 0.03, 0.00];
        let benchmark = vec![0.005, 0.01, -0.02, 0.02, 0.01];
        let te = tracking_error(&strategy, &benchmark, 252.0).unwrap();
        assert!(te > 0.0);
        let ir = information_ratio(&strategy, &benchmark, 252.0).unwrap();
        assert!(ir.is_finite());
    }

    #[test]
    fn test_information_ratio_insufficient_data() {
        assert!(information_ratio(&[0.01], &[0.01], 252.0).is_none());
        assert!(information_ratio(&[0.01, 0.02], &[0.01], 252.0).is_none());
    }

    #[test]
    fn test_information_ratio_zero_tracking_error() {
        // Identical series -> zero excess std dev -> None
        let r = vec![0.01, 0.02, 0.03, -0.01];
        assert!(information_ratio(&r, &r, 252.0).is_none());
        assert!(tracking_error(&r, &r, 252.0).unwrap() == 0.0);
    }
}
