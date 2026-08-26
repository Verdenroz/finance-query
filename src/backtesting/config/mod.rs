//! Backtest configuration.

mod builder;
mod costs;
mod sizing;

use serde::{Deserialize, Serialize};

use super::error::{BacktestError, Result};

pub use builder::BacktestConfigBuilder;
pub use costs::CommissionFn;
pub use sizing::{PositionSizing, SizingContext};

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

    // ── Phase 5: Enhanced Broker Simulation ──────────────────────────────────
    /// Symmetric bid-ask spread as a fraction of price (0.0 – 1.0).
    ///
    /// On each fill, **half** the spread widens the entry price adversely and
    /// **half** widens the exit price adversely (independent of [`slippage_pct`],
    /// which models directional market impact). For example, a `0.0002` spread
    /// (2 bps) costs 1 bp on entry and 1 bp on exit.
    ///
    /// Defaults to `0.0`.
    ///
    /// [`slippage_pct`]: Self::slippage_pct
    pub spread_pct: f64,

    /// Transaction tax as a fraction of trade value, applied on **buy** orders
    /// only (0.0 – 1.0).
    ///
    /// Models jurisdiction-specific purchase taxes such as the UK Stamp Duty
    /// Reserve Tax (0.5 %). Applied on:
    /// - Long entries (buying shares)
    /// - Short exits (covering the short — i.e. buying to close)
    ///
    /// Defaults to `0.0`.
    pub transaction_tax_pct: f64,

    /// Custom commission function `f(size, price) -> commission`.
    ///
    /// When `Some`, **replaces** the flat [`commission`] + percentage
    /// [`commission_pct`] fields. The function receives the fill quantity
    /// (`size`) and the fill price (`price`) and must return the total
    /// commission amount in the same currency as [`initial_capital`].
    ///
    /// **Not serialized** — reconstruct after deserialization if needed.
    ///
    /// [`commission`]: Self::commission
    /// [`commission_pct`]: Self::commission_pct
    /// [`initial_capital`]: Self::initial_capital
    #[serde(skip)]
    pub commission_fn: Option<CommissionFn>,

    /// Maximum gross exposure as a multiple of equity.
    ///
    /// `1.0` (the default) is a cash account: an entry can commit at most the
    /// available equity. Above `1.0` the shortfall is a margin loan, charged at
    /// [`margin_interest_rate`] and subject to [`maintenance_margin_pct`].
    ///
    /// [`margin_interest_rate`]: Self::margin_interest_rate
    /// [`maintenance_margin_pct`]: Self::maintenance_margin_pct
    #[serde(default = "default_max_leverage")]
    pub max_leverage: f64,

    /// Equity floor as a fraction of gross exposure, below which the broker
    /// liquidates the position (0.0 - 1.0).
    ///
    /// Only consulted when [`max_leverage`] exceeds `1.0`, since an unlevered
    /// account cannot fall below it.
    ///
    /// [`max_leverage`]: Self::max_leverage
    #[serde(default = "default_maintenance_margin_pct")]
    pub maintenance_margin_pct: f64,

    /// Annual rate charged on the value of borrowed shares while a short
    /// position is open (0.0 - 1.0).
    ///
    /// Prorated per bar by [`bars_per_year`]. Defaults to `0.0`.
    ///
    /// [`bars_per_year`]: Self::bars_per_year
    #[serde(default)]
    pub short_borrow_rate: f64,

    /// Annual rate charged on a debit cash balance (0.0 - 1.0).
    ///
    /// A leveraged long drives cash negative; that shortfall is the margin
    /// loan. Prorated per bar by [`bars_per_year`]. Defaults to `0.0`, which
    /// makes leverage free and will flatter any leveraged strategy.
    ///
    /// [`bars_per_year`]: Self::bars_per_year
    #[serde(default)]
    pub margin_interest_rate: f64,

    /// Scheme used to size each entry.
    ///
    /// Defaults to [`PositionSizing::FixedFraction`], which commits
    /// [`position_size_pct`] of equity. Every other scheme treats that field as
    /// a ceiling and sizes at or below it.
    ///
    /// [`position_size_pct`]: Self::position_size_pct
    #[serde(default)]
    pub position_sizing: PositionSizing,
}

fn default_max_leverage() -> f64 {
    1.0
}

fn default_maintenance_margin_pct() -> f64 {
    0.25
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
            spread_pct: 0.0,
            transaction_tax_pct: 0.0,
            commission_fn: None,
            max_leverage: default_max_leverage(),
            maintenance_margin_pct: default_maintenance_margin_pct(),
            short_borrow_rate: 0.0,
            margin_interest_rate: 0.0,
            position_sizing: PositionSizing::default(),
        }
    }
}

impl BacktestConfig {
    /// Create a zero-cost configuration with no commission, slippage, spread, or tax.
    ///
    /// Useful for unit tests and frictionless benchmark comparisons.
    /// All other fields use the same defaults as [`BacktestConfig::default()`].
    pub fn zero_cost() -> Self {
        Self {
            commission: 0.0,
            commission_pct: 0.0,
            slippage_pct: 0.0,
            spread_pct: 0.0,
            transaction_tax_pct: 0.0,
            commission_fn: None,
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

        if !(0.0..=1.0).contains(&self.spread_pct) {
            return Err(BacktestError::invalid_param(
                "spread_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.transaction_tax_pct) {
            return Err(BacktestError::invalid_param(
                "transaction_tax_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if !self.max_leverage.is_finite() || self.max_leverage < 1.0 {
            return Err(BacktestError::invalid_param(
                "max_leverage",
                "must be finite and at least 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.maintenance_margin_pct) {
            return Err(BacktestError::invalid_param(
                "maintenance_margin_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if self.max_leverage > 1.0 && self.max_leverage * self.maintenance_margin_pct >= 1.0 {
            return Err(BacktestError::invalid_param(
                "max_leverage",
                "must be below 1.0 / maintenance_margin_pct, or a full-size entry \
                 is liquidated on the bar after it opens",
            ));
        }

        if !(0.0..=1.0).contains(&self.short_borrow_rate) {
            return Err(BacktestError::invalid_param(
                "short_borrow_rate",
                "must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.margin_interest_rate) {
            return Err(BacktestError::invalid_param(
                "margin_interest_rate",
                "must be between 0.0 and 1.0",
            ));
        }

        self.validate_position_sizing()?;

        Ok(())
    }

    fn validate_position_sizing(&self) -> Result<()> {
        match self.position_sizing {
            PositionSizing::FixedFraction => {}
            PositionSizing::Atr {
                risk_pct,
                atr_period,
                atr_multiple,
            } => {
                if !(0.0..=1.0).contains(&risk_pct) {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.risk_pct",
                        "must be between 0.0 and 1.0",
                    ));
                }
                if atr_period == 0 {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.atr_period",
                        "must be at least 1",
                    ));
                }
                if !atr_multiple.is_finite() || atr_multiple <= 0.0 {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.atr_multiple",
                        "must be finite and positive",
                    ));
                }
            }
            PositionSizing::VolatilityTarget {
                target_vol_pct,
                lookback,
            } => {
                if !(0.0..=1.0).contains(&target_vol_pct) {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.target_vol_pct",
                        "must be between 0.0 and 1.0",
                    ));
                }
                if lookback < 2 {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.lookback",
                        "must be at least 2",
                    ));
                }
            }
            PositionSizing::FractionalKelly {
                kelly_fraction,
                lookback_trades,
            } => {
                if !(0.0..=1.0).contains(&kelly_fraction) {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.kelly_fraction",
                        "must be between 0.0 and 1.0",
                    ));
                }
                if lookback_trades == 0 {
                    return Err(BacktestError::invalid_param(
                        "position_sizing.lookback_trades",
                        "must be at least 1",
                    ));
                }
            }
        }

        Ok(())
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
    fn test_leverage_rejected_when_it_cannot_survive_its_own_entry() {
        let levered = |leverage: f64, maintenance: f64| {
            BacktestConfig::builder()
                .max_leverage(leverage)
                .maintenance_margin_pct(maintenance)
                .build()
        };

        assert!(levered(5.0, 0.25).is_err());
        assert!(levered(4.0, 0.25).is_err());
        assert!(levered(3.0, 0.25).is_ok());
        assert!(levered(1.0, 1.0).is_ok());
    }

    #[test]
    fn test_position_sizing_validation_failures() {
        let sizing = |s: PositionSizing| BacktestConfig::builder().position_sizing(s).build();

        assert!(
            sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 0,
                atr_multiple: 2.0,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::Atr {
                risk_pct: -0.01,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 0.0,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 1,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 0,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::FractionalKelly {
                kelly_fraction: -0.5,
                lookback_trades: 20,
            })
            .is_err()
        );
        assert!(
            sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .is_ok()
        );
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
    fn test_spread_validation() {
        assert!(BacktestConfig::builder().spread_pct(1.5).build().is_err());
        assert!(BacktestConfig::builder().spread_pct(-0.01).build().is_err());
        assert!(BacktestConfig::builder().spread_pct(0.0).build().is_ok());
        assert!(BacktestConfig::builder().spread_pct(1.0).build().is_ok());
    }

    #[test]
    fn test_transaction_tax_validation() {
        assert!(
            BacktestConfig::builder()
                .transaction_tax_pct(1.5)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .transaction_tax_pct(-0.001)
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_margin_defaults_are_a_cash_account() {
        let config = BacktestConfig::default();
        assert_eq!(config.max_leverage, 1.0);
        assert_eq!(config.maintenance_margin_pct, 0.25);
        assert_eq!(config.short_borrow_rate, 0.0);
        assert_eq!(config.margin_interest_rate, 0.0);
        assert_eq!(config.position_sizing, PositionSizing::FixedFraction);
    }

    #[test]
    fn test_margin_field_validation() {
        assert!(BacktestConfig::builder().max_leverage(0.5).build().is_err());
        assert!(
            BacktestConfig::builder()
                .max_leverage(f64::NAN)
                .build()
                .is_err()
        );
        assert!(BacktestConfig::builder().max_leverage(3.0).build().is_ok());

        assert!(
            BacktestConfig::builder()
                .maintenance_margin_pct(1.5)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .short_borrow_rate(-0.01)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .margin_interest_rate(1.5)
                .build()
                .is_err()
        );
    }

    #[test]
    fn test_config_without_margin_fields_deserializes_to_defaults() {
        let json = serde_json::json!({
            "initial_capital": 10_000.0,
            "commission": 0.0,
            "commission_pct": 0.001,
            "slippage_pct": 0.001,
            "position_size_pct": 1.0,
            "max_positions": 1,
            "allow_short": false,
            "min_signal_strength": 0.0,
            "stop_loss_pct": null,
            "take_profit_pct": null,
            "close_at_end": true,
            "risk_free_rate": 0.0,
            "trailing_stop_pct": null,
            "reinvest_dividends": false,
            "bars_per_year": 252.0,
            "spread_pct": 0.0,
            "transaction_tax_pct": 0.0,
        });
        let config: BacktestConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.max_leverage, 1.0);
        assert_eq!(config.maintenance_margin_pct, 0.25);
        assert_eq!(config.short_borrow_rate, 0.0);
        assert_eq!(config.margin_interest_rate, 0.0);
        assert_eq!(config.position_sizing, PositionSizing::FixedFraction);
    }

    #[test]
    fn test_zero_cost_clears_new_fields() {
        let config = BacktestConfig::zero_cost();
        assert_eq!(config.spread_pct, 0.0);
        assert_eq!(config.transaction_tax_pct, 0.0);
        assert!(config.commission_fn.is_none());
    }
}
