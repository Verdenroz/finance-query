//! Backtest configuration and builder.

use serde::{Deserialize, Serialize};

use super::error::{BacktestError, Result};

/// Configuration for backtest execution.
///
/// Use `BacktestConfig::builder()` to construct with the builder pattern.
///
/// # Example
///
/// ```
/// use finance_query::backtesting::BacktestConfig;
///
/// let config = BacktestConfig::builder()
///     .initial_capital(50_000.0)
///     .commission_pct(0.001)
///     .slippage_pct(0.0005)
///     .allow_short(true)
///     .stop_loss_pct(0.05)
///     .take_profit_pct(0.10)
///     .build()
///     .unwrap();
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial portfolio capital in base currency
    pub initial_capital: f64,

    /// Commission per trade (flat fee)
    pub commission: f64,

    /// Commission as percentage of trade value (0.0 - 1.0)
    pub commission_pct: f64,

    /// Slippage as percentage of price (0.0 - 1.0)
    pub slippage_pct: f64,

    /// Position sizing: fraction of equity per trade (0.0 - 1.0)
    pub position_size_pct: f64,

    /// Maximum number of concurrent positions (None = unlimited)
    pub max_positions: Option<usize>,

    /// Allow short selling
    pub allow_short: bool,

    /// Require signal strength threshold to trigger trades (0.0 - 1.0)
    pub min_signal_strength: f64,

    /// Stop-loss percentage (0.0 - 1.0). Auto-exit if loss exceeds this.
    pub stop_loss_pct: Option<f64>,

    /// Take-profit percentage (0.0 - 1.0). Auto-exit if profit exceeds this.
    pub take_profit_pct: Option<f64>,

    /// Close any open position at end of backtest
    pub close_at_end: bool,

    /// Annual risk-free rate for Sharpe/Sortino/Calmar ratio calculations (0.0 - 1.0).
    ///
    /// Defaults to `0.0`. Use the current T-bill rate for accurate ratios
    /// (e.g. `0.05` for 5% annual). Converted to a per-period rate internally.
    pub risk_free_rate: f64,

    /// Trailing stop percentage (0.0 - 1.0).
    ///
    /// For **long** positions: tracks the peak (highest) price since entry and
    /// triggers an exit when the price drops this fraction below the peak.
    ///
    /// For **short** positions: tracks the trough (lowest) price since entry and
    /// triggers an exit when the price rises this fraction above the trough.
    ///
    /// Checked before strategy signals each bar, same as `stop_loss_pct` and
    /// `take_profit_pct`. Exit slippage is applied.
    pub trailing_stop_pct: Option<f64>,

    /// When `true`, dividend income received during a holding period is
    /// notionally reinvested: the income is included in the trade's P&L as
    /// if additional shares were purchased at the dividend ex-date close price.
    ///
    /// When `false` (default), dividend income is simply added to P&L at close.
    /// In both cases the dividend amount is recorded on the `Trade` for reporting.
    pub reinvest_dividends: bool,

    /// Number of bars per calendar year, used for annualising returns and ratios.
    ///
    /// Defaults to `252.0` (US equity daily bars). Set to `52.0` for weekly
    /// bars, `12.0` for monthly, or `252.0 * 6.5` (≈ 1638) for hourly bars.
    /// This affects annualised return, Sharpe, Sortino, Calmar, and all
    /// benchmark metrics.
    pub bars_per_year: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            commission: 0.0,
            commission_pct: 0.001,  // 0.1% per trade
            slippage_pct: 0.001,    // 0.1% slippage
            position_size_pct: 1.0, // Use 100% of available capital
            max_positions: Some(1), // Single position at a time
            allow_short: false,
            min_signal_strength: 0.0,
            stop_loss_pct: None,
            take_profit_pct: None,
            close_at_end: true,
            risk_free_rate: 0.0,
            trailing_stop_pct: None,
            reinvest_dividends: false,
            bars_per_year: 252.0,
        }
    }
}

impl BacktestConfig {
    /// Create a zero-cost configuration with no commission or slippage.
    ///
    /// Useful for unit tests and frictionless benchmark comparisons.
    /// All other fields use the same defaults as [`BacktestConfig::default()`].
    pub fn zero_cost() -> Self {
        Self {
            commission: 0.0,
            commission_pct: 0.0,
            slippage_pct: 0.0,
            ..Default::default()
        }
    }

    /// Create a new builder
    pub fn builder() -> BacktestConfigBuilder {
        BacktestConfigBuilder::default()
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> Result<()> {
        if self.initial_capital <= 0.0 {
            return Err(BacktestError::invalid_param(
                "initial_capital",
                "must be positive",
            ));
        }

        if self.commission < 0.0 {
            return Err(BacktestError::invalid_param(
                "commission",
                "cannot be negative",
            ));
        }

        if !(0.0..=1.0).contains(&self.commission_pct) {
            return Err(BacktestError::invalid_param(
                "commission_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.slippage_pct) {
            return Err(BacktestError::invalid_param(
                "slippage_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if self.position_size_pct <= 0.0 || self.position_size_pct > 1.0 {
            return Err(BacktestError::invalid_param(
                "position_size_pct",
                "must be between 0.0 (exclusive) and 1.0 (inclusive)",
            ));
        }

        if !(0.0..=1.0).contains(&self.min_signal_strength) {
            return Err(BacktestError::invalid_param(
                "min_signal_strength",
                "must be between 0.0 and 1.0",
            ));
        }

        if let Some(sl) = self.stop_loss_pct
            && !(0.0..=1.0).contains(&sl)
        {
            return Err(BacktestError::invalid_param(
                "stop_loss_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if let Some(tp) = self.take_profit_pct
            && !(0.0..=1.0).contains(&tp)
        {
            return Err(BacktestError::invalid_param(
                "take_profit_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.risk_free_rate) {
            return Err(BacktestError::invalid_param(
                "risk_free_rate",
                "must be between 0.0 and 1.0",
            ));
        }

        if let Some(trail) = self.trailing_stop_pct
            && !(0.0..=1.0).contains(&trail)
        {
            return Err(BacktestError::invalid_param(
                "trailing_stop_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if self.bars_per_year <= 0.0 {
            return Err(BacktestError::invalid_param(
                "bars_per_year",
                "must be positive (e.g. 252 for daily, 52 for weekly)",
            ));
        }

        Ok(())
    }

    /// Calculate commission for a trade value
    pub fn calculate_commission(&self, trade_value: f64) -> f64 {
        self.commission + (trade_value * self.commission_pct)
    }

    /// Apply slippage to a price (for entry)
    pub fn apply_entry_slippage(&self, price: f64, is_long: bool) -> f64 {
        if is_long {
            // Buying: price goes up slightly
            price * (1.0 + self.slippage_pct)
        } else {
            // Shorting: price goes down slightly (less favorable entry)
            price * (1.0 - self.slippage_pct)
        }
    }

    /// Apply slippage to a price (for exit)
    pub fn apply_exit_slippage(&self, price: f64, is_long: bool) -> f64 {
        if is_long {
            // Selling long: price goes down slightly
            price * (1.0 - self.slippage_pct)
        } else {
            // Covering short: price goes up slightly
            price * (1.0 + self.slippage_pct)
        }
    }

    /// Calculate position size based on available capital.
    ///
    /// `price` **must** be the slippage-adjusted entry price (i.e. the value
    /// returned by [`apply_entry_slippage`] for the current direction). Passing
    /// the raw close price when slippage is non-zero would over-allocate capital
    /// and cause the subsequent `entry_value + commission > cash` guard to reject
    /// the entry.
    ///
    /// [`apply_entry_slippage`]: Self::apply_entry_slippage
    pub fn calculate_position_size(&self, available_capital: f64, price: f64) -> f64 {
        let capital_to_use = available_capital * self.position_size_pct;

        // Account for both entry and exit commission so we don't exceed available capital.
        // For percentage commission: Total cost = C * (1 + 2 * commission_pct)
        // For flat commission: reserve 2 * flat_commission (entry + exit)
        // Combined: adjusted = capital_to_use / (1 + 2 * commission_pct) - 2 * commission
        let adjusted_capital =
            capital_to_use / (1.0 + 2.0 * self.commission_pct) - 2.0 * self.commission;

        (adjusted_capital / price).max(0.0)
    }
}

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
    fn test_default_config() {
        let config = BacktestConfig::default();
        assert_eq!(config.initial_capital, 10_000.0);
        assert!(config.validate().is_ok());
    }

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
    fn test_validation_failures() {
        assert!(
            BacktestConfig::builder()
                .initial_capital(-100.0)
                .build()
                .is_err()
        );

        assert!(
            BacktestConfig::builder()
                .commission_pct(1.5)
                .build()
                .is_err()
        );

        assert!(
            BacktestConfig::builder()
                .stop_loss_pct(2.0)
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_commission_calculation() {
        let config = BacktestConfig::builder()
            .commission(5.0)
            .commission_pct(0.01)
            .build()
            .unwrap();

        // For $1000 trade: $5 flat + 1% = $5 + $10 = $15
        let commission = config.calculate_commission(1000.0);
        assert!((commission - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_slippage() {
        let config = BacktestConfig::builder()
            .slippage_pct(0.01) // 1%
            .build()
            .unwrap();

        // Long entry: price goes up
        let entry_price = config.apply_entry_slippage(100.0, true);
        assert!((entry_price - 101.0).abs() < 0.01);

        // Long exit: price goes down
        let exit_price = config.apply_exit_slippage(100.0, true);
        assert!((exit_price - 99.0).abs() < 0.01);

        // Short entry: price goes down (less favorable)
        let short_entry = config.apply_entry_slippage(100.0, false);
        assert!((short_entry - 99.0).abs() < 0.01);

        // Short exit: price goes up
        let short_exit = config.apply_exit_slippage(100.0, false);
        assert!((short_exit - 101.0).abs() < 0.01);
    }

    #[test]
    fn test_position_sizing() {
        let config = BacktestConfig::builder()
            .position_size_pct(0.5) // Use 50% of capital
            .commission_pct(0.0) // No commission for simpler test
            .build()
            .unwrap();

        // With $10,000 and price $100, use $5,000 -> 50 shares
        let size = config.calculate_position_size(10_000.0, 100.0);
        assert!((size - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_risk_free_rate() {
        let config = BacktestConfig::builder()
            .risk_free_rate(0.05)
            .build()
            .unwrap();
        assert!((config.risk_free_rate - 0.05).abs() < f64::EPSILON);

        // Out-of-range should fail
        assert!(
            BacktestConfig::builder()
                .risk_free_rate(1.5)
                .build()
                .is_err()
        );
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

    #[test]
    fn test_position_sizing_with_commission() {
        let config = BacktestConfig::builder()
            .position_size_pct(0.5) // Use 50% of capital
            .commission_pct(0.001) // 0.1% commission
            .build()
            .unwrap();

        // With $10,000 and price $100, use $5,000
        // But adjusted for entry + exit commission: 5000 / 1.002 = 4990.019960...
        // So shares = 4990.019960 / 100 = 49.90...
        let size = config.calculate_position_size(10_000.0, 100.0);
        let expected = 5000.0 / 1.002 / 100.0;
        assert!((size - expected).abs() < 0.01);
    }

    #[test]
    fn test_position_size_zero_rejected() {
        assert!(
            BacktestConfig::builder()
                .position_size_pct(0.0)
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_bars_per_year_validation() {
        // Default is 252
        let config = BacktestConfig::default();
        assert!((config.bars_per_year - 252.0).abs() < f64::EPSILON);
        assert!(config.validate().is_ok());

        // Valid custom value
        let config = BacktestConfig::builder()
            .bars_per_year(52.0)
            .build()
            .unwrap();
        assert!((config.bars_per_year - 52.0).abs() < f64::EPSILON);

        // Zero must be rejected
        assert!(
            BacktestConfig::builder()
                .bars_per_year(0.0)
                .build()
                .is_err()
        );

        // Negative must be rejected
        assert!(
            BacktestConfig::builder()
                .bars_per_year(-1.0)
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_position_sizing_accounts_for_exit_commission() {
        // Verify the denominator is 1 + 2*comm (entry + exit)
        let comm = 0.01; // 1%
        let config = BacktestConfig::builder()
            .commission_pct(comm)
            .position_size_pct(1.0)
            .build()
            .unwrap();
        let size = config.calculate_position_size(10_000.0, 100.0);
        let expected = 10_000.0 / (1.0 + 2.0 * comm) / 100.0;
        assert!((size - expected).abs() < 0.001);
    }

    #[test]
    fn test_position_sizing_flat_commission_reduces_size() {
        // With $10 flat commission per side, $20 total must be reserved
        let config = BacktestConfig::builder()
            .commission(10.0)
            .commission_pct(0.0)
            .position_size_pct(1.0)
            .build()
            .unwrap();
        let size_with_flat = config.calculate_position_size(10_000.0, 100.0);

        let config_no_flat = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(1.0)
            .build()
            .unwrap();
        let size_no_flat = config_no_flat.calculate_position_size(10_000.0, 100.0);

        // Flat commission should reduce position size
        assert!(size_with_flat < size_no_flat);
        // Expected: (10_000 - 20) / 100 = 99.8
        let expected = (10_000.0 - 20.0) / 100.0;
        assert!((size_with_flat - expected).abs() < 0.001);
    }

    #[test]
    fn test_position_sizing_flat_commission_exceeds_capital_returns_zero() {
        // If flat commission alone exceeds available capital, quantity should be 0
        let config = BacktestConfig::builder()
            .commission(6_000.0) // $6k/side → $12k total > $10k capital
            .position_size_pct(1.0)
            .build()
            .unwrap();
        let size = config.calculate_position_size(10_000.0, 100.0);
        assert_eq!(size, 0.0);
    }
}
