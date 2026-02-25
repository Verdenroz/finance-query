//! Dividend analytics computed from historical dividend data.

use serde::{Deserialize, Serialize};

use super::events::Dividend;

/// Computed analytics derived from a symbol's dividend history.
///
/// Obtain via [`Ticker::dividend_analytics`](crate::Ticker::dividend_analytics).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendAnalytics {
    /// Total dividends paid in the requested range
    pub total_paid: f64,
    /// Number of dividend payments in the requested range
    pub payment_count: usize,
    /// Average dividend per payment
    pub average_payment: f64,
    /// Compound Annual Growth Rate of the dividend amount.
    ///
    /// `None` when fewer than two payments spanning at least one year are available.
    pub cagr: Option<f64>,
    /// Most recent dividend payment
    pub last_payment: Option<Dividend>,
    /// Earliest dividend payment in the requested range
    pub first_payment: Option<Dividend>,
}

impl DividendAnalytics {
    /// Compute analytics from a pre-filtered, chronologically sorted slice of dividends.
    pub(crate) fn from_dividends(dividends: &[Dividend]) -> Self {
        if dividends.is_empty() {
            return Self {
                total_paid: 0.0,
                payment_count: 0,
                average_payment: 0.0,
                cagr: None,
                last_payment: None,
                first_payment: None,
            };
        }

        let total_paid: f64 = dividends.iter().map(|d| d.amount).sum();
        let payment_count = dividends.len();
        let average_payment = total_paid / payment_count as f64;

        let first = dividends.first().cloned();
        let last = dividends.last().cloned();

        let cagr = compute_cagr(&first, &last);

        Self {
            total_paid,
            payment_count,
            average_payment,
            cagr,
            last_payment: last,
            first_payment: first,
        }
    }
}

/// Compute dividend CAGR between two payments.
///
/// Requires at least one full year between payments and both amounts > 0.
fn compute_cagr(first: &Option<Dividend>, last: &Option<Dividend>) -> Option<f64> {
    let first = first.as_ref()?;
    let last = last.as_ref()?;

    if first.amount <= 0.0 || last.amount <= 0.0 {
        return None;
    }

    let years = (last.timestamp - first.timestamp) as f64 / 31_557_600.0; // seconds per Julian year
    if years < 1.0 {
        return None;
    }

    Some((last.amount / first.amount).powf(1.0 / years) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn div(timestamp: i64, amount: f64) -> Dividend {
        Dividend { timestamp, amount }
    }

    #[test]
    fn test_empty_dividends() {
        let a = DividendAnalytics::from_dividends(&[]);
        assert_eq!(a.payment_count, 0);
        assert_eq!(a.total_paid, 0.0);
        assert!(a.cagr.is_none());
    }

    #[test]
    fn test_single_dividend() {
        let a = DividendAnalytics::from_dividends(&[div(1_000_000_000, 0.50)]);
        assert_eq!(a.payment_count, 1);
        assert!((a.total_paid - 0.50).abs() < 1e-9);
        assert!(a.cagr.is_none()); // only one payment, can't compute CAGR
    }

    #[test]
    fn test_cagr_two_years() {
        // Two payments ~2 years apart: 0.50 → 0.605 ≈ 10% CAGR
        let secs_per_year = 31_557_600_i64;
        let t0 = 1_600_000_000_i64;
        let t1 = t0 + 2 * secs_per_year;
        let a = DividendAnalytics::from_dividends(&[div(t0, 0.50), div(t1, 0.605)]);
        assert!(a.cagr.is_some());
        let cagr = a.cagr.unwrap();
        assert!(
            (cagr - 0.10).abs() < 0.01,
            "expected ~10% CAGR, got {cagr:.4}"
        );
    }

    #[test]
    fn test_totals() {
        let divs = [
            div(1_000_000, 0.25),
            div(2_000_000, 0.25),
            div(3_000_000, 0.25),
        ];
        let a = DividendAnalytics::from_dividends(&divs);
        assert_eq!(a.payment_count, 3);
        assert!((a.total_paid - 0.75).abs() < 1e-9);
        assert!((a.average_payment - 0.25).abs() < 1e-9);
    }
}
