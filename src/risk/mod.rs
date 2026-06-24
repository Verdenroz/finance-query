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
#[cfg(feature = "python")]
use finance_query_derive::PyModel;
use serde::{Deserialize, Serialize};

/// Comprehensive risk summary for a symbol.
///
/// Obtain via [`Ticker::risk`](crate::Ticker::risk).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", derive(PyModel))]
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

/// Trading calendar used to annualise risk ratios — different asset classes
/// have different period counts per year.
// Which variants are constructed depends on enabled provider features (each
// domain handle picks one), so some are unused under a given feature set.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TradingCalendar {
    /// Exchange-traded (equities, indices, futures, commodities): 252 trading
    /// days, ~6.5h sessions.
    Exchange,
    /// Foreign exchange: 24h trading, 5 days/week (~260 days/year).
    Forex,
    /// Crypto: continuous 24/7 trading (365 days/year).
    Crypto,
}

impl TradingCalendar {
    fn trading_days(self) -> f64 {
        match self {
            TradingCalendar::Exchange => 252.0,
            TradingCalendar::Forex => 260.0,
            TradingCalendar::Crypto => 365.0,
        }
    }

    fn session_hours(self) -> f64 {
        match self {
            TradingCalendar::Exchange => 6.5,
            TradingCalendar::Forex | TradingCalendar::Crypto => 24.0,
        }
    }
}

/// Number of `interval` periods in a trading year for the given calendar — the
/// annualisation factor for Sharpe/Sortino/Calmar. Day/week/month/quarter are
/// exact; intraday scales the daily count by the session length.
pub(crate) fn periods_per_year(interval: crate::Interval, cal: TradingCalendar) -> f64 {
    use crate::Interval;
    let days = cal.trading_days();
    match interval {
        Interval::OneDay => days,
        Interval::OneWeek => 52.0,
        Interval::OneMonth => 12.0,
        Interval::ThreeMonths => 4.0,
        Interval::OneHour => days * cal.session_hours(),
        Interval::ThirtyMinutes => days * cal.session_hours() * 2.0,
        Interval::FifteenMinutes => days * cal.session_hours() * 4.0,
        Interval::FiveMinutes => days * cal.session_hours() * 12.0,
        Interval::OneMinute => days * cal.session_hours() * 60.0,
    }
}

/// Build a [`RiskSummary`] from candle data and an optional benchmark return
/// series, using the default daily exchange calendar (252 periods/year).
pub(crate) fn compute_risk_summary(
    candles: &[Candle],
    benchmark_returns: Option<&[f64]>,
) -> RiskSummary {
    compute_risk_summary_with_periods(candles, benchmark_returns, 252.0)
}

/// Build a [`RiskSummary`] with an explicit annualisation factor
/// (`periods_per_year`), so non-daily intervals and non-equity asset classes
/// annualise correctly. See [`periods_per_year`].
pub(crate) fn compute_risk_summary_with_periods(
    candles: &[Candle],
    benchmark_returns: Option<&[f64]>,
    periods_per_year: f64,
) -> RiskSummary {
    let returns = candles_to_returns(candles);

    let var_95 = historical_var(&returns, 0.95).unwrap_or(0.0);
    let var_99 = historical_var(&returns, 0.99).unwrap_or(0.0);
    let parametric_var_95 = parametric_var(&returns, 0.95).unwrap_or(0.0);

    let sharpe = sharpe_ratio(&returns, 0.0, periods_per_year);
    let sortino = sortino_ratio(&returns, 0.0, periods_per_year);

    let dd = max_drawdown(&returns);
    let total_return = returns.iter().fold(1.0_f64, |acc, r| acc * (1.0 + r)) - 1.0;
    let years = returns.len() as f64 / periods_per_year;
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
            provider_id: None,
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

    #[test]
    fn test_periods_per_year_by_calendar() {
        use crate::Interval;
        // Daily differs by asset class; calendar periodicity is shared.
        assert_eq!(
            periods_per_year(Interval::OneDay, TradingCalendar::Exchange),
            252.0
        );
        assert_eq!(
            periods_per_year(Interval::OneDay, TradingCalendar::Forex),
            260.0
        );
        assert_eq!(
            periods_per_year(Interval::OneDay, TradingCalendar::Crypto),
            365.0
        );
        assert_eq!(
            periods_per_year(Interval::OneWeek, TradingCalendar::Crypto),
            52.0
        );
        // Intraday scales daily count by session length (24h crypto > 6.5h exchange).
        assert!(
            periods_per_year(Interval::OneHour, TradingCalendar::Crypto)
                > periods_per_year(Interval::OneHour, TradingCalendar::Exchange)
        );
    }

    #[test]
    fn test_annualization_factor_changes_sharpe() {
        // Same returns, different periods_per_year → different annualised Sharpe.
        let candles: Vec<Candle> = (0..50).map(|i| make_candle(100.0 + i as f64)).collect();
        let daily = compute_risk_summary_with_periods(&candles, None, 252.0);
        let crypto = compute_risk_summary_with_periods(&candles, None, 365.0);
        assert!(daily.sharpe.is_some() && crypto.sharpe.is_some());
        assert!(crypto.sharpe.unwrap() > daily.sharpe.unwrap());
    }
}
