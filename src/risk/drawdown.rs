//! Maximum drawdown analysis.

use crate::models::chart::Candle;

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

/// Where the maximum drawdown happened in a candle series.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct DrawdownSpan {
    /// Maximum drawdown as a positive fraction (e.g., 0.30 = 30% loss from peak)
    pub max_drawdown: f64,
    /// Index into the candle slice of the peak the drawdown fell from
    pub peak_index: usize,
    /// Index into the candle slice of the trough
    pub trough_index: usize,
    /// Unix timestamp of the peak candle
    pub peak_timestamp: i64,
    /// Unix timestamp of the trough candle
    pub trough_timestamp: i64,
}

/// Locate the maximum drawdown of a candle series by close price.
///
/// Returns `None` when the closes never fall below a prior peak. `close` is
/// not dividend-adjusted. The depth matches [`max_drawdown`] over
/// [`returns`](crate::risk::returns) of the same candles.
pub fn max_drawdown_span(candles: &[Candle]) -> Option<DrawdownSpan> {
    let mut peak_idx = 0_usize;
    let mut best: Option<(f64, usize, usize)> = None;

    for (i, c) in candles.iter().enumerate() {
        let peak = candles[peak_idx].close;
        if c.close > peak {
            peak_idx = i;
            continue;
        }
        let dd = (peak - c.close) / peak;
        if dd > best.map_or(0.0, |(d, ..)| d) {
            best = Some((dd, peak_idx, i));
        }
    }

    best.map(|(max_drawdown, peak_index, trough_index)| DrawdownSpan {
        max_drawdown,
        peak_index,
        trough_index,
        peak_timestamp: candles[peak_index].timestamp,
        trough_timestamp: candles[trough_index].timestamp,
    })
}

/// Per-period drawdown fraction (peak-to-current), built from the same
/// synthetic cumulative-equity curve `max_drawdown` uses internally.
///
/// Returned length is `returns.len() + 1` (includes the starting point,
/// always `0.0`). Used to feed `perf_metrics::ulcer_index` — kept separate
/// from `max_drawdown` itself since that function also tracks peak/trough
/// indices for recovery-period detection that ulcer index doesn't need.
pub(crate) fn drawdown_series(returns: &[f64]) -> Vec<f64> {
    if returns.is_empty() {
        return Vec::new();
    }
    let mut equity = Vec::with_capacity(returns.len() + 1);
    equity.push(1.0_f64);
    for r in returns {
        let last = *equity.last().unwrap();
        equity.push(last * (1.0 + r));
    }

    let mut peak = equity[0];
    let mut out = Vec::with_capacity(equity.len());
    for &val in &equity {
        if val > peak {
            peak = val;
        }
        out.push((peak - val) / peak);
    }
    out
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

    fn candle(timestamp: i64, close: f64) -> Candle {
        Candle::__new(timestamp, close, close, close, close, 1, None, None)
    }

    #[test]
    fn span_locates_peak_and_trough() {
        let candles = [
            candle(10, 100.0),
            candle(20, 110.0),
            candle(30, 88.0),
            candle(40, 95.0),
            candle(50, 120.0),
        ];
        let span = max_drawdown_span(&candles).unwrap();
        assert_eq!((span.peak_index, span.trough_index), (1, 2));
        assert_eq!((span.peak_timestamp, span.trough_timestamp), (20, 30));
        let from_returns = max_drawdown(&crate::risk::returns(&candles));
        assert!((span.max_drawdown - from_returns.max_drawdown).abs() < 1e-12);
    }

    #[test]
    fn span_is_none_without_drawdown() {
        assert!(max_drawdown_span(&[]).is_none());
        assert!(max_drawdown_span(&[candle(1, 1.0), candle(2, 2.0)]).is_none());
    }
}
