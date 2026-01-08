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
        }
    }
}

impl BacktestConfig {
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

        if !(0.0..=1.0).contains(&self.position_size_pct) {
            return Err(BacktestError::invalid_param(
                "position_size_pct",
                "must be between 0.0 and 1.0",
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

    /// Calculate position size based on available capital
    pub fn calculate_position_size(&self, available_capital: f64, price: f64) -> f64 {
        let capital_to_use = available_capital * self.position_size_pct;

        // Account for commission to ensure we don't exceed available capital
        // Commission is commission_pct of the total entry value
        // If we use capital C, commission is commission_pct * C
        // Total needed: C + (commission_pct * C) = C * (1 + commission_pct)
        // So we should use: capital_to_use / (1 + commission_pct)
        let adjusted_capital = capital_to_use / (1.0 + self.commission_pct);

        adjusted_capital / price
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
    fn test_position_sizing_with_commission() {
        let config = BacktestConfig::builder()
            .position_size_pct(0.5) // Use 50% of capital
            .commission_pct(0.001) // 0.1% commission
            .build()
            .unwrap();

        // With $10,000 and price $100, use $5,000
        // But adjusted for commission: 5000 / 1.001 = 4995.004995
        // So shares = 4995.004995 / 100 = 49.95
        let size = config.calculate_position_size(10_000.0, 100.0);
        let expected = 5000.0 / 1.001 / 100.0;
        assert!((size - expected).abs() < 0.01);
    }
}
