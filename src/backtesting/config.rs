//! Backtest configuration and builder.

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::error::{BacktestError, Result};

// ── CommissionFn ──────────────────────────────────────────────────────────────

/// A custom commission function: `f(size, price) -> commission_amount`.
///
/// When set on [`BacktestConfig`] via [`BacktestConfigBuilder::commission_fn`],
/// it **replaces** the flat `commission` + percentage `commission_pct` fields.
/// Use it to model broker-specific fee schedules such as per-share fees with
/// a minimum, tiered rates, or Robinhood-style zero-commission structures.
///
/// # Example
///
/// ```
/// use finance_query::backtesting::BacktestConfig;
///
/// // IB-style: $0.005 per share, minimum $1.00 per order
/// let config = BacktestConfig::builder()
///     .commission_fn(|size, price| (size * 0.005_f64).max(1.00))
///     .build()
///     .unwrap();
/// ```
#[derive(Clone)]
pub struct CommissionFn(Arc<dyn Fn(f64, f64) -> f64 + Send + Sync>);

impl CommissionFn {
    /// Create from any closure or function pointer matching `Fn(f64, f64) -> f64`.
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(f64, f64) -> f64 + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }

    /// Call the underlying function with `(size, price)`.
    #[inline]
    pub(crate) fn call(&self, size: f64, price: f64) -> f64 {
        (self.0)(size, price)
    }
}

impl fmt::Debug for CommissionFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommissionFn(<closure>)")
    }
}

// ── Position sizing schemes ───────────────────────────────────────────────────

/// Position sizing scheme selector.
///
/// Defaults to [`PositionSizing::FixedFraction`], which delegates entirely to
/// [`BacktestConfig::position_size_pct`] — existing configurations that never
/// touch this field see byte-for-byte identical sizing behavior.
///
/// The non-default variants compute their own fraction-of-equity per entry
/// using bar-level context (ATR, realized volatility, or trailing trade
/// history) supplied by the engine via [`SizingContext`]; when that context is
/// unavailable (e.g. still in the ATR warmup, or too few closed trades for a
/// reliable Kelly estimate) every variant falls back to
/// [`BacktestConfig::position_size_pct`] so a scheme never returns an
/// undefined position size.
///
/// **Scope note:** these schemes govern the fraction used to size a *new*
/// entry (`Signal::long` / `Signal::short`). Scale-in adds
/// (`Signal::scale_in`) continue to use the signal's own explicit
/// `scale_fraction` regardless of the configured scheme.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PositionSizing {
    /// Fixed fraction of equity per trade — uses [`BacktestConfig::position_size_pct`].
    ///
    /// This is the default and preserves pre-existing sizing behavior exactly.
    #[default]
    FixedFraction,

    /// Size inversely proportional to Average True Range: a larger ATR (more
    /// volatility) produces a smaller position, normalising risk-per-trade
    /// across regimes.
    ///
    /// The fraction of equity allocated is derived so that a stop-out at
    /// `atr_multiple` ATRs away from entry would lose approximately
    /// `risk_pct` of equity:
    /// `fraction = risk_pct * entry_price / (atr_multiple * atr)`.
    Atr {
        /// Fraction of equity to risk if price moves `atr_multiple` ATRs against the entry (0.0 - 1.0).
        risk_pct: f64,
        /// Lookback period for the ATR computation.
        atr_period: usize,
        /// Stop distance expressed as a multiple of ATR.
        atr_multiple: f64,
    },

    /// Size to target a specific portfolio volatility contribution.
    ///
    /// The fraction of equity allocated is `target_vol_pct / recent_volatility`,
    /// where `recent_volatility` is the trailing per-bar return standard
    /// deviation over `lookback` bars — smaller positions when the asset has
    /// been more volatile than the target, larger when calmer.
    VolatilityTarget {
        /// Target per-bar portfolio volatility contribution (fractional, e.g. `0.01` for 1%).
        target_vol_pct: f64,
        /// Number of trailing bars used to estimate realized volatility.
        lookback: usize,
    },

    /// Size as a fraction of the Kelly-optimal bet size, derived from the
    /// strategy's own trailing win-rate and payoff ratio.
    ///
    /// `kelly_star = win_rate - (1 - win_rate) / payoff_ratio`, then
    /// `fraction = kelly_fraction * max(kelly_star, 0.0)`. Using a fraction
    /// (e.g. `0.5` for "half-Kelly") rather than full Kelly is standard
    /// practice — full Kelly is highly sensitive to estimation error in
    /// `win_rate`/`payoff_ratio` and can produce large drawdowns.
    FractionalKelly {
        /// Multiplier applied to the full Kelly fraction (e.g. `0.5` = half-Kelly).
        kelly_fraction: f64,
        /// Number of trailing closed trades used to estimate win-rate/payoff ratio.
        lookback_trades: usize,
    },
}

/// Bar-level context supplied to non-default [`PositionSizing`] schemes.
///
/// Populated by the engine from precomputed indicator/statistics series and
/// the trade log accumulated so far in the simulation. All fields are
/// `None` when the corresponding scheme isn't active or when there isn't yet
/// enough history to compute a value (e.g. before the ATR warmup period, or
/// before enough trades have closed for a Kelly estimate) — in which case the
/// scheme falls back to [`BacktestConfig::position_size_pct`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SizingContext {
    /// Current Average True Range value, for [`PositionSizing::Atr`].
    pub atr: Option<f64>,
    /// Trailing per-bar return standard deviation, for [`PositionSizing::VolatilityTarget`].
    pub recent_volatility: Option<f64>,
    /// Trailing win rate (0.0 - 1.0) from closed trades, for [`PositionSizing::FractionalKelly`].
    pub win_rate: Option<f64>,
    /// Trailing average-win / average-loss ratio, for [`PositionSizing::FractionalKelly`].
    pub payoff_ratio: Option<f64>,
}

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

    // ── Margin / leverage / borrow cost ──────────────────────────────────────
    /// Maximum notional exposure allowed as a multiple of equity (`1.0` = no
    /// leverage, buying power equals current equity).
    ///
    /// Defaults to `1.0`, matching pre-leverage behavior exactly: position
    /// sizing and entry buying-power checks scale by this factor, so a
    /// default config never allocates more than 100% of equity. Set above
    /// `1.0` to model a leveraged (margin) account.
    ///
    /// [`maintenance_margin_pct`]: Self::maintenance_margin_pct
    #[serde(default = "default_max_leverage")]
    pub max_leverage: f64,

    /// Minimum ratio of equity to notional exposure before a margin call
    /// force-closes the position (0.0 - 1.0). Only enforced when
    /// [`max_leverage`] is greater than `1.0` — at the default leverage of
    /// `1.0` this field has no effect, preserving backward compatibility.
    ///
    /// Defaults to `0.25` (a typical Reg-T maintenance margin). When
    /// `equity < notional_exposure * maintenance_margin_pct`, the engine
    /// force-closes the full position at the current bar's close.
    ///
    /// [`max_leverage`]: Self::max_leverage
    #[serde(default = "default_maintenance_margin_pct")]
    pub maintenance_margin_pct: f64,

    /// Annualized borrow rate charged on short positions' notional value
    /// (0.0 - 1.0), accrued each bar and prorated by [`bars_per_year`].
    ///
    /// Defaults to `0.0` (no borrow cost — pre-existing behavior for short
    /// strategies is unchanged). Models the stock-loan fee a broker charges
    /// to maintain a short position.
    ///
    /// [`bars_per_year`]: Self::bars_per_year
    #[serde(default)]
    pub short_borrow_rate: f64,

    /// Position sizing scheme. Defaults to [`PositionSizing::FixedFraction`],
    /// which uses [`position_size_pct`] exactly as before this field existed.
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

        if self.max_leverage < 1.0 || !self.max_leverage.is_finite() {
            return Err(BacktestError::invalid_param(
                "max_leverage",
                "must be finite and >= 1.0 (1.0 = no leverage)",
            ));
        }

        if !(0.0..=1.0).contains(&self.maintenance_margin_pct) {
            return Err(BacktestError::invalid_param(
                "maintenance_margin_pct",
                "must be between 0.0 and 1.0",
            ));
        }

        if !(0.0..=1.0).contains(&self.short_borrow_rate) {
            return Err(BacktestError::invalid_param(
                "short_borrow_rate",
                "must be between 0.0 and 1.0 (annualized fraction)",
            ));
        }

        Ok(())
    }

    /// Calculate commission for a fill.
    ///
    /// When [`commission_fn`] is set it takes precedence over the flat
    /// [`commission`] + percentage [`commission_pct`] fields.
    ///
    /// [`commission_fn`]: Self::commission_fn
    /// [`commission`]: Self::commission
    /// [`commission_pct`]: Self::commission_pct
    pub fn calculate_commission(&self, size: f64, price: f64) -> f64 {
        if let Some(ref f) = self.commission_fn {
            f.call(size, price)
        } else {
            self.commission + (size * price * self.commission_pct)
        }
    }

    /// Apply slippage to a price (for entry).
    pub fn apply_entry_slippage(&self, price: f64, is_long: bool) -> f64 {
        if is_long {
            price * (1.0 + self.slippage_pct)
        } else {
            price * (1.0 - self.slippage_pct)
        }
    }

    /// Apply slippage to a price (for exit).
    pub fn apply_exit_slippage(&self, price: f64, is_long: bool) -> f64 {
        if is_long {
            price * (1.0 - self.slippage_pct)
        } else {
            price * (1.0 + self.slippage_pct)
        }
    }

    /// Apply the bid-ask spread to an entry fill price (half-spread adverse).
    ///
    /// Long entries pay the ask (price rises by `spread_pct / 2`);
    /// short entries receive the bid (price falls by `spread_pct / 2`).
    pub fn apply_entry_spread(&self, price: f64, is_long: bool) -> f64 {
        let half = self.spread_pct / 2.0;
        if is_long {
            price * (1.0 + half)
        } else {
            price * (1.0 - half)
        }
    }

    /// Apply the bid-ask spread to an exit fill price (half-spread adverse).
    ///
    /// Long exits receive the bid (price falls by `spread_pct / 2`);
    /// short exits pay the ask (price rises by `spread_pct / 2`).
    pub fn apply_exit_spread(&self, price: f64, is_long: bool) -> f64 {
        let half = self.spread_pct / 2.0;
        if is_long {
            price * (1.0 - half)
        } else {
            price * (1.0 + half)
        }
    }

    /// Calculate the transaction tax on a fill.
    ///
    /// Tax applies only to **buy** orders (`is_buy = true`):
    /// - Long entries (opening a long position)
    /// - Short exits (covering a short position)
    ///
    /// Returns `0.0` for all sell orders.
    pub fn calculate_transaction_tax(&self, trade_value: f64, is_buy: bool) -> f64 {
        if is_buy {
            trade_value * self.transaction_tax_pct
        } else {
            0.0
        }
    }

    /// Calculate position size based on available capital.
    ///
    /// `price` **must** be the fully-adjusted entry price (after slippage and
    /// spread) so that subsequent fill guards (`entry_value + costs > cash`)
    /// do not over-allocate capital.
    ///
    /// When [`commission_fn`] is set the commission component cannot be
    /// analytically solved for, so only spread and transaction-tax fractions
    /// are deducted from the denominator; the fill-rejection guard catches any
    /// remaining over-allocation.
    ///
    /// Scales by [`max_leverage`] (default `1.0`, i.e. no change from
    /// pre-leverage behavior). Always uses [`position_size_pct`] as the
    /// fraction of equity — for [`PositionSizing`]-aware sizing use
    /// [`calculate_position_size_with_context`].
    ///
    /// [`commission_fn`]: Self::commission_fn
    /// [`max_leverage`]: Self::max_leverage
    /// [`position_size_pct`]: Self::position_size_pct
    /// [`calculate_position_size_with_context`]: Self::calculate_position_size_with_context
    pub fn calculate_position_size(&self, available_capital: f64, price: f64) -> f64 {
        self.size_from_fraction(
            available_capital,
            price,
            self.position_size_pct * self.max_leverage,
        )
    }

    /// Calculate position size using the configured [`PositionSizing`] scheme.
    ///
    /// `ctx` supplies the bar-level ATR / realized-volatility / trailing
    /// win-rate-and-payoff-ratio values the non-default schemes need. When the
    /// relevant field is `None` (scheme inactive, or not enough history yet),
    /// the scheme falls back to [`position_size_pct`]. The resulting fraction
    /// is then scaled by [`max_leverage`] exactly as in
    /// [`calculate_position_size`], and clamped so it never exceeds
    /// `max_leverage` regardless of how the scheme's formula evaluates.
    ///
    /// With the default [`PositionSizing::FixedFraction`] scheme this produces
    /// results **identical** to [`calculate_position_size`] — the sizing
    /// scheme is purely additive.
    ///
    /// [`position_size_pct`]: Self::position_size_pct
    /// [`max_leverage`]: Self::max_leverage
    /// [`calculate_position_size`]: Self::calculate_position_size
    pub fn calculate_position_size_with_context(
        &self,
        available_capital: f64,
        price: f64,
        ctx: &SizingContext,
    ) -> f64 {
        let base_fraction = match self.position_sizing {
            PositionSizing::FixedFraction => self.position_size_pct,
            PositionSizing::Atr {
                risk_pct,
                atr_multiple,
                ..
            } => match ctx.atr {
                Some(atr) if atr > 0.0 && atr_multiple > 0.0 && price > 0.0 => {
                    (risk_pct * price) / (atr_multiple * atr)
                }
                _ => self.position_size_pct,
            },
            PositionSizing::VolatilityTarget { target_vol_pct, .. } => {
                match ctx.recent_volatility {
                    Some(vol) if vol > 0.0 => target_vol_pct / vol,
                    _ => self.position_size_pct,
                }
            }
            PositionSizing::FractionalKelly { kelly_fraction, .. } => {
                match (ctx.win_rate, ctx.payoff_ratio) {
                    (Some(w), Some(r)) if r > 0.0 => {
                        let kelly_star = w - (1.0 - w) / r;
                        (kelly_fraction * kelly_star).max(0.0)
                    }
                    _ => self.position_size_pct,
                }
            }
        };

        // Scale by leverage, then cap so no scheme's formula can allocate more
        // than the configured leverage ceiling (e.g. a very low ATR/vol input).
        let leveraged_max = self.max_leverage.max(1.0);
        let fraction = (base_fraction.max(0.0) * self.max_leverage).min(leveraged_max);

        self.size_from_fraction(available_capital, price, fraction)
    }

    /// Shared position-size arithmetic: convert a fraction-of-equity into a
    /// quantity, after deducting round-trip friction costs.
    fn size_from_fraction(&self, available_capital: f64, price: f64, fraction: f64) -> f64 {
        let capital_to_use = available_capital * fraction;

        let adjusted_capital = if self.commission_fn.is_some() {
            // Can't analytically invert commission_fn; use spread + tax only.
            // The fill-rejection guard will catch any over-allocation.
            capital_to_use / (1.0 + self.spread_pct + self.transaction_tax_pct)
        } else {
            // Round-trip costs (fraction of trade value):
            //   - Commission: 2 × commission_pct  (entry + exit)
            //   - Spread:     spread_pct           (half each way)
            //   - Tax:        transaction_tax_pct  (buy only — conservative for shorts)
            let friction =
                1.0 + 2.0 * self.commission_pct + self.spread_pct + self.transaction_tax_pct;
            capital_to_use / friction - 2.0 * self.commission
        };

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

    /// Set the maximum notional exposure allowed, as a multiple of equity.
    ///
    /// `1.0` (the default) means no leverage — buying power equals equity.
    /// Values above `1.0` allow leveraged long/short entries; combine with
    /// [`maintenance_margin_pct`](Self::maintenance_margin_pct) to model
    /// margin-call liquidation.
    pub fn max_leverage(mut self, leverage: f64) -> Self {
        self.config.max_leverage = leverage;
        self
    }

    /// Set the maintenance margin ratio (equity / notional exposure) below
    /// which the engine force-closes the position (0.0 - 1.0).
    ///
    /// Only enforced when [`max_leverage`](Self::max_leverage) is greater
    /// than `1.0`. Defaults to `0.25`.
    pub fn maintenance_margin_pct(mut self, pct: f64) -> Self {
        self.config.maintenance_margin_pct = pct;
        self
    }

    /// Set the annualized borrow rate charged on short positions' notional
    /// value (0.0 - 1.0). Defaults to `0.0` (no borrow cost).
    pub fn short_borrow_rate(mut self, rate: f64) -> Self {
        self.config.short_borrow_rate = rate;
        self
    }

    /// Set the position sizing scheme. Defaults to
    /// [`PositionSizing::FixedFraction`], which uses
    /// [`position_size_pct`](Self::position_size_pct).
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

        // For $1000 trade (10 units @ $100): $5 flat + 1% = $5 + $10 = $15
        let commission = config.calculate_commission(10.0, 100.0);
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

    // ── Phase 5: Enhanced Broker Simulation ──────────────────────────────────

    #[test]
    fn test_spread_entry_long() {
        let config = BacktestConfig::builder()
            .spread_pct(0.0004) // 4 bps
            .build()
            .unwrap();
        // Long entry pays the ask: price rises by half-spread (2 bps)
        let price = config.apply_entry_spread(100.0, true);
        assert!((price - 100.02).abs() < 1e-10);
    }

    #[test]
    fn test_spread_exit_long() {
        let config = BacktestConfig::builder()
            .spread_pct(0.0004)
            .build()
            .unwrap();
        // Long exit receives the bid: price falls by half-spread
        let price = config.apply_exit_spread(100.0, true);
        assert!((price - 99.98).abs() < 1e-10);
    }

    #[test]
    fn test_spread_entry_short() {
        let config = BacktestConfig::builder()
            .spread_pct(0.0004)
            .build()
            .unwrap();
        // Short entry receives the bid: price falls by half-spread
        let price = config.apply_entry_spread(100.0, false);
        assert!((price - 99.98).abs() < 1e-10);
    }

    #[test]
    fn test_spread_exit_short() {
        let config = BacktestConfig::builder()
            .spread_pct(0.0004)
            .build()
            .unwrap();
        // Short exit pays the ask: price rises by half-spread
        let price = config.apply_exit_spread(100.0, false);
        assert!((price - 100.02).abs() < 1e-10);
    }

    #[test]
    fn test_spread_zero_is_noop() {
        let config = BacktestConfig::default(); // spread_pct = 0.0
        assert!((config.apply_entry_spread(123.45, true) - 123.45).abs() < 1e-10);
        assert!((config.apply_exit_spread(123.45, false) - 123.45).abs() < 1e-10);
    }

    #[test]
    fn test_spread_validation() {
        assert!(BacktestConfig::builder().spread_pct(1.5).build().is_err());
        assert!(BacktestConfig::builder().spread_pct(-0.01).build().is_err());
        assert!(BacktestConfig::builder().spread_pct(0.0).build().is_ok());
        assert!(BacktestConfig::builder().spread_pct(1.0).build().is_ok());
    }

    #[test]
    fn test_transaction_tax_on_buy() {
        let config = BacktestConfig::builder()
            .transaction_tax_pct(0.005) // UK stamp duty 0.5%
            .build()
            .unwrap();
        let tax = config.calculate_transaction_tax(10_000.0, true);
        assert!((tax - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_transaction_tax_not_on_sell() {
        let config = BacktestConfig::builder()
            .transaction_tax_pct(0.005)
            .build()
            .unwrap();
        let tax = config.calculate_transaction_tax(10_000.0, false);
        assert_eq!(tax, 0.0);
    }

    #[test]
    fn test_transaction_tax_zero_default() {
        let config = BacktestConfig::default();
        assert_eq!(config.calculate_transaction_tax(100_000.0, true), 0.0);
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
    fn test_commission_fn_replaces_flat_and_pct() {
        // Custom fn: $0.005/share minimum $1.00
        let config = BacktestConfig::builder()
            .commission_fn(|size, _price| (size * 0.005_f64).max(1.00))
            .build()
            .unwrap();
        // 100 shares: 100 * 0.005 = $0.50 → minimum kicks in → $1.00
        let comm = config.calculate_commission(100.0, 50.0);
        assert!((comm - 1.00).abs() < 1e-10);
        // 500 shares: 500 * 0.005 = $2.50 → above minimum
        let comm = config.calculate_commission(500.0, 50.0);
        assert!((comm - 2.50).abs() < 1e-10);
    }

    #[test]
    fn test_commission_fn_ignores_flat_and_pct_fields() {
        // Even with flat=5 and pct=0.01 set, commission_fn should override
        let config = BacktestConfig::builder()
            .commission(5.0)
            .commission_pct(0.01)
            .commission_fn(|size, price| size * price * 0.0005)
            .build()
            .unwrap();
        // 10 shares @ $100: fn gives 10*100*0.0005 = $0.50
        let comm = config.calculate_commission(10.0, 100.0);
        assert!((comm - 0.50).abs() < 1e-10);
    }

    #[test]
    fn test_commission_fn_fallback_when_none() {
        // Without commission_fn, standard flat+pct applies
        let config = BacktestConfig::builder()
            .commission(1.0)
            .commission_pct(0.002)
            .build()
            .unwrap();
        // 10 shares @ $100 = $1000 trade: $1 + $2 = $3
        let comm = config.calculate_commission(10.0, 100.0);
        assert!((comm - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_position_sizing_includes_spread_and_tax() {
        let spread = 0.0004; // 4 bps round-trip
        let tax = 0.005; // 0.5% stamp duty
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .spread_pct(spread)
            .transaction_tax_pct(tax)
            .position_size_pct(1.0)
            .build()
            .unwrap();

        let size = config.calculate_position_size(10_000.0, 100.0);
        let expected = 10_000.0 / (1.0 + spread + tax) / 100.0;
        assert!((size - expected).abs() < 0.01);
    }

    #[test]
    fn test_zero_cost_clears_new_fields() {
        let config = BacktestConfig::zero_cost();
        assert_eq!(config.spread_pct, 0.0);
        assert_eq!(config.transaction_tax_pct, 0.0);
        assert!(config.commission_fn.is_none());
    }

    // ── Margin / leverage / sizing scheme defaults ─────────────────────────────

    #[test]
    fn test_default_config_has_no_leverage() {
        let config = BacktestConfig::default();
        assert_eq!(config.max_leverage, 1.0);
        assert_eq!(config.maintenance_margin_pct, 0.25);
        assert_eq!(config.short_borrow_rate, 0.0);
        assert_eq!(config.position_sizing, PositionSizing::FixedFraction);
    }

    #[test]
    fn test_max_leverage_validation() {
        assert!(BacktestConfig::builder().max_leverage(0.5).build().is_err());
        assert!(BacktestConfig::builder().max_leverage(1.0).build().is_ok());
        assert!(BacktestConfig::builder().max_leverage(3.0).build().is_ok());
    }

    #[test]
    fn test_maintenance_margin_pct_validation() {
        assert!(
            BacktestConfig::builder()
                .maintenance_margin_pct(-0.1)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .maintenance_margin_pct(1.5)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .maintenance_margin_pct(0.25)
                .build()
                .is_ok()
        );
    }

    #[test]
    fn test_short_borrow_rate_validation() {
        assert!(
            BacktestConfig::builder()
                .short_borrow_rate(-0.01)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .short_borrow_rate(1.5)
                .build()
                .is_err()
        );
        assert!(
            BacktestConfig::builder()
                .short_borrow_rate(0.03)
                .build()
                .is_ok()
        );
    }

    #[test]
    fn test_position_sizing_builder() {
        let config = BacktestConfig::builder()
            .position_sizing(PositionSizing::Atr {
                risk_pct: 0.01,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .build()
            .unwrap();
        assert_eq!(
            config.position_sizing,
            PositionSizing::Atr {
                risk_pct: 0.01,
                atr_period: 14,
                atr_multiple: 2.0,
            }
        );
    }

    // ── calculate_position_size_with_context ──────────────────────────────────

    #[test]
    fn test_sizing_with_context_fixed_fraction_matches_plain() {
        // Default scheme must produce byte-identical output to the old method.
        let config = BacktestConfig::builder()
            .position_size_pct(0.6)
            .commission_pct(0.0)
            .build()
            .unwrap();
        let ctx = SizingContext::default();
        let with_ctx = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let plain = config.calculate_position_size(10_000.0, 100.0);
        assert!((with_ctx - plain).abs() < 1e-12);
    }

    #[test]
    fn test_sizing_atr_scheme_matches_formula() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .build()
            .unwrap();
        let ctx = SizingContext {
            atr: Some(2.0),
            ..Default::default()
        };
        // fraction = risk_pct * price / (atr_multiple * atr) = 0.02*100/(2.0*2.0) = 0.5
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected_fraction = 0.02 * 100.0 / (2.0 * 2.0);
        let expected = 10_000.0 * expected_fraction / 100.0;
        assert!((size - expected).abs() < 0.01);
    }

    #[test]
    fn test_sizing_atr_scheme_falls_back_without_atr() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.4)
            .position_sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .build()
            .unwrap();
        let ctx = SizingContext::default(); // atr: None
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected = config.calculate_position_size(10_000.0, 100.0);
        assert!((size - expected).abs() < 1e-9);
    }

    #[test]
    fn test_sizing_volatility_target_scheme_matches_formula() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_sizing(PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            })
            .build()
            .unwrap();
        let ctx = SizingContext {
            recent_volatility: Some(0.02),
            ..Default::default()
        };
        // fraction = target_vol_pct / recent_volatility = 0.01/0.02 = 0.5
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected = 10_000.0 * 0.5 / 100.0;
        assert!((size - expected).abs() < 0.01);
    }

    #[test]
    fn test_sizing_volatility_target_falls_back_without_data() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.3)
            .position_sizing(PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            })
            .build()
            .unwrap();
        let ctx = SizingContext::default();
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected = config.calculate_position_size(10_000.0, 100.0);
        assert!((size - expected).abs() < 1e-9);
    }

    #[test]
    fn test_sizing_fractional_kelly_matches_formula() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_sizing(PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 20,
            })
            .build()
            .unwrap();
        // win_rate=0.6, payoff_ratio=2.0 → kelly_star = 0.6 - 0.4/2.0 = 0.4
        // fraction = 0.5 * 0.4 = 0.2
        let ctx = SizingContext {
            win_rate: Some(0.6),
            payoff_ratio: Some(2.0),
            ..Default::default()
        };
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected = 10_000.0 * 0.2 / 100.0;
        assert!((size - expected).abs() < 0.01);
    }

    #[test]
    fn test_sizing_fractional_kelly_negative_edge_clamps_to_zero() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_sizing(PositionSizing::FractionalKelly {
                kelly_fraction: 1.0,
                lookback_trades: 20,
            })
            .build()
            .unwrap();
        // win_rate=0.2, payoff_ratio=1.0 → kelly_star = 0.2 - 0.8/1.0 = -0.6 → clamp to 0
        let ctx = SizingContext {
            win_rate: Some(0.2),
            payoff_ratio: Some(1.0),
            ..Default::default()
        };
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert_eq!(size, 0.0);
    }

    #[test]
    fn test_sizing_fractional_kelly_falls_back_without_history() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.35)
            .position_sizing(PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 20,
            })
            .build()
            .unwrap();
        let ctx = SizingContext::default();
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let expected = config.calculate_position_size(10_000.0, 100.0);
        assert!((size - expected).abs() < 1e-9);
    }

    #[test]
    fn test_sizing_with_context_scales_with_leverage() {
        let base = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.5)
            .build()
            .unwrap();
        let leveraged = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.5)
            .max_leverage(2.0)
            .build()
            .unwrap();

        let ctx = SizingContext::default();
        let base_size = base.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        let leveraged_size = leveraged.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert!((leveraged_size - base_size * 2.0).abs() < 1e-6);
    }
}
