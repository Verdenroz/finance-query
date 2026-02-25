//! Maximum drawdown analysis.

/// Maximum drawdown result.
#[derive(Debug, Clone)]
pub struct DrawdownResult {
    /// Maximum drawdown as a positive fraction (e.g., 0.30 = 30% loss from peak)
    pub max_drawdown: f64,
    /// Number of periods from trough to next peak (recovery). `None` if no recovery.
    pub recovery_periods: Option<u64>,
}

/// Compute the maximum drawdown from a return series.
///
/// # Arguments
///
/// * `returns` - Per-period returns as fractions (e.g., daily returns)
///
/// Returns `DrawdownResult` with `max_drawdown = 0.0` when `returns` is empty.
pub fn max_drawdown(returns: &[f64]) -> DrawdownResult {
    if returns.is_empty() {
        return DrawdownResult {
            max_drawdown: 0.0,
            recovery_periods: None,
        };
    }

    // Build cumulative equity curve (starting at 1.0)
    let mut equity = Vec::with_capacity(returns.len() + 1);
    equity.push(1.0_f64);
    for r in returns {
        let last = *equity.last().unwrap();
        equity.push(last * (1.0 + r));
    }

    let mut peak = equity[0];
    let mut peak_idx = 0_usize;
    let mut max_dd = 0.0_f64;
    let mut trough_idx = 0_usize;

    for (i, &val) in equity.iter().enumerate() {
        if val > peak {
            peak = val;
            peak_idx = i;
        }
        let dd = (peak - val) / peak;
        if dd > max_dd {
            max_dd = dd;
            trough_idx = i;
        }
    }

    // Find recovery: first index after trough where equity returns to peak level
    let recovery_periods = if max_dd > 0.0 {
        let peak_val = equity[peak_idx];
        equity[trough_idx..]
            .iter()
            .enumerate()
            .skip(1)
            .find(|&(_, &v)| v >= peak_val)
            .map(|(offset, _)| offset as u64)
    } else {
        None
    };

    DrawdownResult {
        max_drawdown: max_dd,
        recovery_periods,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let r = max_drawdown(&[]);
        assert_eq!(r.max_drawdown, 0.0);
        assert!(r.recovery_periods.is_none());
    }

    #[test]
    fn test_all_positive() {
        let returns = vec![0.01_f64; 100];
        let r = max_drawdown(&returns);
        assert_eq!(r.max_drawdown, 0.0);
    }

    #[test]
    fn test_single_drop_recovery() {
        // Up 10%, down 20%, up 15%
        let returns = vec![0.10_f64, -0.20, 0.15];
        let r = max_drawdown(&returns);
        // After +10%: 1.10; after -20%: 0.88; drawdown = (1.10 - 0.88)/1.10 ≈ 0.20
        assert!(
            (r.max_drawdown - 0.20).abs() < 0.01,
            "got {}",
            r.max_drawdown
        );
    }

    #[test]
    fn test_no_recovery() {
        // Steady decline — never recovers
        let returns = vec![-0.05_f64; 10];
        let r = max_drawdown(&returns);
        assert!(r.max_drawdown > 0.0);
        assert!(r.recovery_periods.is_none());
    }
}
