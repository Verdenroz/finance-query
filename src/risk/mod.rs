//! Standalone risk analytics.
//!
//! Requires the **`risk`** feature flag (which implies **`indicators`**).
//!
//! Provides Value at Risk, Sharpe/Sortino/Calmar ratios, beta, and max drawdown
//! as standalone metrics — independent of the backtesting engine.
//!
//! # Quick Start
//!
//! ```no_run
//! use finance_query::{Ticker, Interval, TimeRange};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ticker = Ticker::new("AAPL").await?;
//! let summary = ticker.risk(Interval::OneDay, TimeRange::OneYear, None).await?;
//!
//! println!("VaR (95%):      {:.2}%", summary.var_95 * 100.0);
//! println!("Max drawdown:   {:.2}%", summary.max_drawdown * 100.0);
//! if let Some(sharpe) = summary.sharpe {
//!     println!("Sharpe ratio:   {sharpe:.2}");
//! }
//! # Ok(())
//! # }
//! ```

mod beta;
mod drawdown;
mod ratios;
mod var;

pub use self::beta::beta;
pub use self::drawdown::max_drawdown;
pub use self::ratios::{calmar_ratio, sharpe_ratio, sortino_ratio};
pub use self::var::{historical_var, parametric_var};

use crate::models::chart::Candle;
use serde::{Deserialize, Serialize};

/// Comprehensive risk summary for a symbol.
///
/// Obtain via [`Ticker::risk`](crate::Ticker::risk).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RiskSummary {
    /// 1-day historical Value at Risk at 95% confidence (expressed as positive loss fraction)
    pub var_95: f64,
    /// 1-day historical Value at Risk at 99% confidence
    pub var_99: f64,
    /// 1-day parametric VaR at 95% confidence (assumes normally distributed returns)
    pub parametric_var_95: f64,
    /// Annualised Sharpe Ratio (risk-free rate = 0, 252 trading days/year).
    /// `None` when fewer than 2 periods or zero volatility.
    pub sharpe: Option<f64>,
    /// Annualised Sortino Ratio (penalises only downside volatility).
    /// `None` when fewer than 2 periods or zero downside deviation.
    pub sortino: Option<f64>,
    /// Calmar Ratio (annualised return / max drawdown).
    /// `None` when max drawdown is zero.
    pub calmar: Option<f64>,
    /// Beta vs benchmark. `None` when no benchmark is provided or data is insufficient.
    pub beta: Option<f64>,
    /// Maximum drawdown as a positive fraction (e.g., 0.30 = 30%)
    pub max_drawdown: f64,
    /// Number of trading periods to recover from the maximum drawdown.
    /// `None` when no recovery occurred within the data window.
    pub max_drawdown_recovery_periods: Option<u64>,
}

/// Compute returns from a slice of candles (simple daily returns: close-to-close).
pub(crate) fn candles_to_returns(candles: &[Candle]) -> Vec<f64> {
    candles
        .windows(2)
        .map(|w| (w[1].close - w[0].close) / w[0].close)
        .collect()
}

/// Build a [`RiskSummary`] from candle data and an optional benchmark return series.
pub(crate) fn compute_risk_summary(
    candles: &[Candle],
    benchmark_returns: Option<&[f64]>,
) -> RiskSummary {
    let returns = candles_to_returns(candles);

    let var_95 = historical_var(&returns, 0.95).unwrap_or(0.0);
    let var_99 = historical_var(&returns, 0.99).unwrap_or(0.0);
    let parametric_var_95 = parametric_var(&returns, 0.95).unwrap_or(0.0);

    let sharpe = sharpe_ratio(&returns, 0.0, 252.0);
    let sortino = sortino_ratio(&returns, 0.0, 252.0);

    let dd = max_drawdown(&returns);
    let total_return = returns.iter().fold(1.0_f64, |acc, r| acc * (1.0 + r)) - 1.0;
    let years = returns.len() as f64 / 252.0;
    let calmar = calmar_ratio(total_return, years, dd.max_drawdown);

    let beta_val = benchmark_returns.and_then(|br| beta(&returns, br));

    RiskSummary {
        var_95,
        var_99,
        parametric_var_95,
        sharpe,
        sortino,
        calmar,
        beta: beta_val,
        max_drawdown: dd.max_drawdown,
        max_drawdown_recovery_periods: dd.recovery_periods,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(close: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: close,
            high: close,
            low: close,
            close,
            volume: 1_000_000,
            adj_close: None,
        }
    }

    #[test]
    fn test_compute_risk_summary_flat() {
        // Constant prices → zero returns → zero VaR, no ratios
        let candles: Vec<Candle> = (0..=252).map(|_| make_candle(100.0)).collect();
        let summary = compute_risk_summary(&candles, None);
        assert_eq!(summary.var_95, 0.0);
        assert_eq!(summary.max_drawdown, 0.0);
        assert!(summary.sharpe.is_none());
    }

    #[test]
    fn test_candles_to_returns_basic() {
        let candles = vec![make_candle(100.0), make_candle(110.0), make_candle(99.0)];
        let returns = candles_to_returns(&candles);
        assert_eq!(returns.len(), 2);
        assert!((returns[0] - 0.10).abs() < 1e-9);
        assert!((returns[1] - (-0.1)).abs() < 0.01);
    }
}
