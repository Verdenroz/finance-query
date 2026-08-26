//! Builder for [`BacktestConfig`].

use super::{BacktestConfig, CommissionFn, PositionSizing};
use crate::backtesting::error::Result;

/// Builder for BacktestConfig
#[derive(Default)]
pub struct BacktestConfigBuilder {
    config: BacktestConfig,
}

impl BacktestConfigBuilder {
    /// Set initial capital
    pub fn initial_capital(mut self, capital: f64) -> Self {
        self.config.initial_capital = capital;
        self
    }

    /// Set flat commission per trade
    pub fn commission(mut self, fee: f64) -> Self {
        self.config.commission = fee;
        self
    }

    /// Set commission as percentage of trade value
    pub fn commission_pct(mut self, pct: f64) -> Self {
        self.config.commission_pct = pct;
        self
    }

    /// Set slippage as percentage of price
    pub fn slippage_pct(mut self, pct: f64) -> Self {
        self.config.slippage_pct = pct;
        self
    }

    /// Set position size as fraction of available equity
    pub fn position_size_pct(mut self, pct: f64) -> Self {
        self.config.position_size_pct = pct;
        self
    }

    /// Set maximum concurrent positions
    pub fn max_positions(mut self, max: usize) -> Self {
        self.config.max_positions = Some(max);
        self
    }

    /// Allow unlimited concurrent positions
    pub fn unlimited_positions(mut self) -> Self {
        self.config.max_positions = None;
        self
    }

    /// Allow or disallow short selling
    pub fn allow_short(mut self, allow: bool) -> Self {
        self.config.allow_short = allow;
        self
    }

    /// Set minimum signal strength threshold
    pub fn min_signal_strength(mut self, threshold: f64) -> Self {
        self.config.min_signal_strength = threshold;
        self
    }

    /// Set stop-loss percentage (auto-exit if loss exceeds this)
    pub fn stop_loss_pct(mut self, pct: f64) -> Self {
        self.config.stop_loss_pct = Some(pct);
        self
    }

    /// Set take-profit percentage (auto-exit if profit exceeds this)
    pub fn take_profit_pct(mut self, pct: f64) -> Self {
        self.config.take_profit_pct = Some(pct);
        self
    }

    /// Set whether to close open positions at end of backtest
    pub fn close_at_end(mut self, close: bool) -> Self {
        self.config.close_at_end = close;
        self
    }

    /// Set annual risk-free rate for Sharpe/Sortino/Calmar calculations (0.0 - 1.0)
    ///
    /// Use the current T-bill rate for accurate ratios (e.g. `0.05` for 5%).
    pub fn risk_free_rate(mut self, rate: f64) -> Self {
        self.config.risk_free_rate = rate;
        self
    }

    /// Set trailing stop percentage (0.0 - 1.0).
    ///
    /// For longs: exits when price drops this fraction below its peak since entry.
    /// For shorts: exits when price rises this fraction above its trough since entry.
    pub fn trailing_stop_pct(mut self, pct: f64) -> Self {
        self.config.trailing_stop_pct = Some(pct);
        self
    }

    /// Enable or disable dividend reinvestment
    ///
    /// When `true`, dividend income is reinvested (added to P&L as additional hypothetical shares).
    pub fn reinvest_dividends(mut self, reinvest: bool) -> Self {
        self.config.reinvest_dividends = reinvest;
        self
    }

    /// Set the number of bars per calendar year for annualisation.
    ///
    /// Defaults to `252.0` (US equity daily bars). Common values:
    /// - `252.0` — daily US equity
    /// - `52.0` — weekly
    /// - `12.0` — monthly
    /// - `252.0 * 6.5` (≈ 1638) — hourly (6.5-hour trading day)
    pub fn bars_per_year(mut self, n: f64) -> Self {
        self.config.bars_per_year = n;
        self
    }

    /// Set symmetric bid-ask spread as a fraction of price (0.0 – 1.0).
    ///
    /// Half the spread is applied adversely on entry and half on exit,
    /// independent of [`slippage_pct`](BacktestConfig::slippage_pct).
    /// For example, `0.0002` represents a 2-basis-point spread (1 bp per side).
    pub fn spread_pct(mut self, pct: f64) -> Self {
        self.config.spread_pct = pct;
        self
    }

    /// Set the transaction tax as a fraction of trade value, applied on buys only.
    ///
    /// Models purchase taxes such as UK Stamp Duty (0.005 = 0.5 %). Applied on
    /// long entries and short covers; not applied on sells.
    pub fn transaction_tax_pct(mut self, pct: f64) -> Self {
        self.config.transaction_tax_pct = pct;
        self
    }

    /// Set a custom commission function `f(size, price) -> commission`.
    ///
    /// Replaces the flat [`commission`](BacktestConfig::commission) and
    /// percentage [`commission_pct`](BacktestConfig::commission_pct) fields.
    /// Use this to model broker-specific fee schedules.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::backtesting::BacktestConfig;
    ///
    /// // $0.005 per share, minimum $1.00 per order
    /// let config = BacktestConfig::builder()
    ///     .commission_fn(|size, price| (size * 0.005_f64).max(1.00))
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn commission_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(f64, f64) -> f64 + Send + Sync + 'static,
    {
        self.config.commission_fn = Some(CommissionFn::new(f));
        self
    }

    /// Set the position sizing scheme.
    ///
    /// Schemes other than [`PositionSizing::FixedFraction`] size at or below
    /// [`position_size_pct`](BacktestConfig::position_size_pct), which stays the
    /// risk budget for the run.
    pub fn position_sizing(mut self, sizing: PositionSizing) -> Self {
        self.config.position_sizing = sizing;
        self
    }

    /// Build and validate the configuration
    pub fn build(self) -> Result<BacktestConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = BacktestConfig::builder()
            .initial_capital(50_000.0)
            .commission_pct(0.002)
            .allow_short(true)
            .stop_loss_pct(0.05)
            .take_profit_pct(0.10)
            .build()
            .unwrap();

        assert_eq!(config.initial_capital, 50_000.0);
        assert_eq!(config.commission_pct, 0.002);
        assert!(config.allow_short);
        assert_eq!(config.stop_loss_pct, Some(0.05));
        assert_eq!(config.take_profit_pct, Some(0.10));
    }

    #[test]
    fn test_trailing_stop() {
        let config = BacktestConfig::builder()
            .trailing_stop_pct(0.05)
            .build()
            .unwrap();
        assert_eq!(config.trailing_stop_pct, Some(0.05));

        // Out-of-range should fail
        assert!(
            BacktestConfig::builder()
                .trailing_stop_pct(1.5)
                .build()
                .is_err()
        );
    }
}
