//! Position and trade types for tracking open and closed positions.

use serde::{Deserialize, Serialize};

use super::signal::Signal;

/// Position direction
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    /// Long position (profit when price rises)
    Long,
    /// Short position (profit when price falls)
    Short,
}

impl std::fmt::Display for PositionSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Long => write!(f, "LONG"),
            Self::Short => write!(f, "SHORT"),
        }
    }
}

/// An open position
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Position direction
    pub side: PositionSide,

    /// Entry timestamp
    pub entry_timestamp: i64,

    /// Entry price (after slippage)
    pub entry_price: f64,

    /// Number of shares/units
    pub quantity: f64,

    /// Number of shares/units at entry (before any dividend reinvestment).
    #[serde(default)]
    pub entry_quantity: f64,

    /// Entry commission paid
    pub entry_commission: f64,

    /// Transaction tax paid on entry (long entries and short covers only).
    #[serde(default)]
    pub entry_transaction_tax: f64,

    /// Signal that triggered entry
    pub entry_signal: Signal,

    /// Accumulated dividend income received while this position was open.
    ///
    /// Added to trade P&L on close. Zero when dividends are not supplied to
    /// the engine or when the position receives no dividends.
    pub dividend_income: f64,

    /// Dividend income that was NOT reinvested (i.e. remains as cash).
    /// Used internally for correct cash-accounting.
    #[serde(default)]
    pub unreinvested_dividends: f64,

    /// Number of times this position has been scaled into (pyramid adds).
    ///
    /// Starts at `0` (initial entry). Incremented by
    /// [`Position::scale_in`] on each successful add.
    #[serde(default)]
    pub scale_in_count: usize,

    /// Number of partial closes executed so far.
    ///
    /// Used to assign a monotonically increasing [`Trade::scale_sequence`]
    /// to each [`Trade`] returned by [`Position::partial_close`].
    #[serde(default)]
    pub partial_close_count: usize,

    /// Per-trade stop-loss percentage override.
    ///
    /// Populated from [`Signal::bracket_stop_loss_pct`] when the position is
    /// opened. Takes precedence over [`BacktestConfig::stop_loss_pct`] when
    /// `Some`. `None` means fall back to the config-level default.
    ///
    /// [`Signal::bracket_stop_loss_pct`]: crate::backtesting::Signal::bracket_stop_loss_pct
    /// [`BacktestConfig::stop_loss_pct`]: crate::backtesting::BacktestConfig::stop_loss_pct
    #[serde(default)]
    pub bracket_stop_loss_pct: Option<f64>,

    /// Per-trade take-profit percentage override.
    ///
    /// Populated from [`Signal::bracket_take_profit_pct`] when the position is
    /// opened. Takes precedence over [`BacktestConfig::take_profit_pct`] when
    /// `Some`.
    ///
    /// [`Signal::bracket_take_profit_pct`]: crate::backtesting::Signal::bracket_take_profit_pct
    /// [`BacktestConfig::take_profit_pct`]: crate::backtesting::BacktestConfig::take_profit_pct
    #[serde(default)]
    pub bracket_take_profit_pct: Option<f64>,

    /// Per-trade trailing stop percentage override.
    ///
    /// Populated from [`Signal::bracket_trailing_stop_pct`] when the position
    /// is opened. Takes precedence over [`BacktestConfig::trailing_stop_pct`]
    /// when `Some`.
    ///
    /// [`Signal::bracket_trailing_stop_pct`]: crate::backtesting::Signal::bracket_trailing_stop_pct
    /// [`BacktestConfig::trailing_stop_pct`]: crate::backtesting::BacktestConfig::trailing_stop_pct
    #[serde(default)]
    pub bracket_trailing_stop_pct: Option<f64>,
}

impl Position {
    /// Create a new position.
    pub fn new(
        side: PositionSide,
        entry_timestamp: i64,
        entry_price: f64,
        quantity: f64,
        entry_commission: f64,
        entry_signal: Signal,
    ) -> Self {
        Self::new_with_tax(
            side,
            entry_timestamp,
            entry_price,
            quantity,
            entry_commission,
            0.0,
            entry_signal,
        )
    }

    /// Create a new position including an entry transaction tax.
    pub(crate) fn new_with_tax(
        side: PositionSide,
        entry_timestamp: i64,
        entry_price: f64,
        quantity: f64,
        entry_commission: f64,
        entry_transaction_tax: f64,
        entry_signal: Signal,
    ) -> Self {
        let bracket_stop_loss_pct = entry_signal.bracket_stop_loss_pct;
        let bracket_take_profit_pct = entry_signal.bracket_take_profit_pct;
        let bracket_trailing_stop_pct = entry_signal.bracket_trailing_stop_pct;
        Self {
            side,
            entry_timestamp,
            entry_price,
            quantity,
            entry_quantity: quantity,
            entry_commission,
            entry_transaction_tax,
            entry_signal,
            dividend_income: 0.0,
            unreinvested_dividends: 0.0,
            scale_in_count: 0,
            partial_close_count: 0,
            bracket_stop_loss_pct,
            bracket_take_profit_pct,
            bracket_trailing_stop_pct,
        }
    }

    /// Net contribution of this position to portfolio equity at `current_price`.
    ///
    /// **Sign convention (important):** returns a *positive* value for long
    /// positions and a *negative* value for short positions.  The negative
    /// short value is deliberate: when the engine opens a short it credits
    /// `cash` with the sale proceeds (`cash += entry_price × quantity`), so
    /// the correct running equity is `cash + current_value(price)`.  As the
    /// price falls the negative value grows less negative, and the net equity
    /// rises — exactly the expected profit behaviour for a short.
    ///
    /// If you need the raw notional exposure (always positive), use
    /// `self.quantity * current_price` directly.
    pub fn current_value(&self, current_price: f64) -> f64 {
        match self.side {
            PositionSide::Long => self.quantity * current_price,
            PositionSide::Short => -(self.quantity * current_price),
        }
    }

    /// Calculate unrealized P&L at given price (before exit commission)
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let initial_value = self.entry_price * self.entry_quantity;
        let current_value = self.current_value(current_price);

        let gross_pnl = match self.side {
            PositionSide::Long => current_value - initial_value,
            // For shorts: `current_value` is negative `-(quantity * price)`.
            // Initial value is assumed positive margin equivalent, so PnL = expected margin - cost to cover.
            // Wait, current_value for short is `-(self.quantity * current_price)`.
            // The cost to open was `entry_value` = `entry_price * entry_quantity`.
            // Better to be explicit:
            PositionSide::Short => {
                (self.entry_price * self.entry_quantity) - (current_price * self.quantity)
            }
        };
        gross_pnl - self.entry_commission - self.entry_transaction_tax + self.unreinvested_dividends
    }

    /// Calculate unrealized return percentage
    pub fn unrealized_return_pct(&self, current_price: f64) -> f64 {
        let entry_value = self.entry_price * self.entry_quantity;
        if entry_value == 0.0 {
            return 0.0;
        }
        let pnl = self.unrealized_pnl(current_price);
        (pnl / entry_value) * 100.0
    }

    /// Check if position is profitable at given price
    pub fn is_profitable(&self, current_price: f64) -> bool {
        self.unrealized_pnl(current_price) > 0.0
    }

    /// Check if this is a long position
    pub fn is_long(&self) -> bool {
        matches!(self.side, PositionSide::Long)
    }

    /// Check if this is a short position
    pub fn is_short(&self) -> bool {
        matches!(self.side, PositionSide::Short)
    }

    /// Credit dividend cashflow to this position.
    ///
    /// `income` **must be pre-signed by the caller**:
    /// - Long positions *receive* dividends → pass `+per_share × quantity`
    /// - Short positions *owe* dividends to the stock lender → pass
    ///   `-(per_share × quantity)`
    ///
    /// The engine's `credit_dividends` helper handles this negation
    /// automatically.  Passing an unsigned (always-positive) value to a short
    /// position would incorrectly record dividend *income* instead of a
    /// *liability*.
    ///
    /// When `reinvest` is `true`, only **positive** `income` is reinvested
    /// into additional units (long-side reinvestment only).
    pub fn credit_dividend(&mut self, income: f64, close_price: f64, reinvest: bool) {
        if reinvest && income > 0.0 && close_price > 0.0 {
            self.quantity += income / close_price;
        } else {
            self.unreinvested_dividends += income;
        }
        self.dividend_income += income;
    }

    /// Add shares to this position (pyramid / scale-in).
    ///
    /// Updates the weighted-average `entry_price` and `entry_quantity` to reflect
    /// the blended cost basis and increments `scale_in_count`. The caller is
    /// responsible for debiting the entry cost from available cash and for applying
    /// slippage/spread to `fill_price` before calling this method.
    ///
    /// # Arguments
    ///
    /// * `fill_price`      – Adjusted entry price for the new shares.
    /// * `additional_qty`  – Number of shares to add. No-op if `<= 0.0`.
    /// * `commission`      – Commission paid for this add (already applied to cash).
    /// * `entry_tax`       – Transaction tax for this add (already applied to cash).
    pub fn scale_in(
        &mut self,
        fill_price: f64,
        additional_qty: f64,
        commission: f64,
        entry_tax: f64,
    ) {
        if additional_qty <= 0.0 {
            return;
        }

        let old_value = self.entry_price * self.quantity;
        let new_value = fill_price * additional_qty;
        let total_qty = self.quantity + additional_qty;

        self.entry_price = (old_value + new_value) / total_qty;
        self.quantity = total_qty;
        // Keep entry_quantity in sync so close_with_tax computes the correct cost basis.
        self.entry_quantity = total_qty;
        // Track commission and tax in their respective fields for correct proportional
        // slicing in subsequent partial_close calls.
        self.entry_commission += commission;
        self.entry_transaction_tax += entry_tax;
        self.scale_in_count += 1;
    }

    /// Partially close this position and return a completed [`Trade`].
    ///
    /// Closes `fraction` of the current position quantity, allocating a
    /// proportional share of accumulated entry costs and dividend income to the
    /// trade P&L. The remaining position stays open with reduced quantity,
    /// dividend balances, and entry cost bases.
    ///
    /// [`Trade::is_partial`] is `true` for all trades returned by this method.
    /// For a full close prefer [`Position::close_with_tax`], which sets
    /// `is_partial = false`. The engine's `scale_out_position` delegates
    /// `fraction >= 1.0` to `close_position` for exactly this reason.
    ///
    /// The caller is responsible for updating cash from the returned trade's
    /// exit proceeds.
    ///
    /// # Arguments
    ///
    /// * `fraction`   – Portion of current quantity to close (`0.0..=1.0`).
    /// * `exit_ts`    – Timestamp of the fill.
    /// * `exit_price` – Adjusted exit price (after slippage/spread).
    /// * `commission` – Exit-side commission for this close.
    /// * `exit_tax`   – Exit-side transaction tax for this close.
    /// * `signal`     – Signal that triggered the partial exit.
    #[must_use = "the returned Trade must be used to update cash and record the partial close"]
    pub fn partial_close(
        &mut self,
        fraction: f64,
        exit_ts: i64,
        exit_price: f64,
        commission: f64,
        exit_tax: f64,
        signal: Signal,
    ) -> Trade {
        let fraction = fraction.clamp(0.0, 1.0);
        let qty_closed = self.quantity * fraction;
        let qty_remaining = self.quantity - qty_closed;

        // Proportional dividend income for the closed slice.
        let div_income = self.dividend_income * fraction;
        let unreinvested = self.unreinvested_dividends * fraction;
        let entry_comm_slice = self.entry_commission * fraction;
        let entry_tax_slice = self.entry_transaction_tax * fraction;

        // Reduce the open position; keep entry_quantity in sync with quantity so
        // close_with_tax computes the correct cost basis for the remainder.
        self.quantity = qty_remaining;
        self.entry_quantity = qty_remaining;
        self.dividend_income -= div_income;
        self.unreinvested_dividends -= unreinvested;
        self.entry_commission -= entry_comm_slice;
        self.entry_transaction_tax -= entry_tax_slice;

        let gross_pnl = match self.side {
            PositionSide::Long => (exit_price - self.entry_price) * qty_closed,
            PositionSide::Short => (self.entry_price - exit_price) * qty_closed,
        };
        let partial_commission = entry_comm_slice + commission;
        let partial_tax = entry_tax_slice + exit_tax;

        let pnl = gross_pnl - partial_commission - partial_tax + unreinvested;
        let entry_value = self.entry_price * qty_closed;
        let return_pct = if entry_value > 0.0 {
            (pnl / entry_value) * 100.0
        } else {
            0.0
        };

        let seq = self.partial_close_count;
        self.partial_close_count += 1;

        Trade {
            side: self.side,
            entry_timestamp: self.entry_timestamp,
            exit_timestamp: exit_ts,
            entry_price: self.entry_price,
            exit_price,
            quantity: qty_closed,
            entry_quantity: qty_closed,
            commission: partial_commission,
            transaction_tax: partial_tax,
            pnl,
            return_pct,
            dividend_income: div_income,
            unreinvested_dividends: unreinvested,
            entry_signal: self.entry_signal.clone(),
            exit_signal: signal,
            tags: self.entry_signal.tags.clone(),
            is_partial: true,
            scale_sequence: seq,
        }
    }

    /// Close this position and create a Trade.
    ///
    /// `dividend_income` accumulated during the hold is added to P&L and
    /// preserved on the returned `Trade` for reporting purposes.
    pub fn close(
        self,
        exit_timestamp: i64,
        exit_price: f64,
        exit_commission: f64,
        exit_signal: Signal,
    ) -> Trade {
        self.close_with_tax(
            exit_timestamp,
            exit_price,
            exit_commission,
            0.0,
            exit_signal,
        )
    }

    /// Close the position, including an exit transaction tax (e.g. on short covers).
    pub(crate) fn close_with_tax(
        self,
        exit_timestamp: i64,
        exit_price: f64,
        exit_commission: f64,
        exit_transaction_tax: f64,
        exit_signal: Signal,
    ) -> Trade {
        let total_commission = self.entry_commission + exit_commission;
        let total_transaction_tax = self.entry_transaction_tax + exit_transaction_tax;

        let initial_value = self.entry_price * self.entry_quantity;
        let exit_value = exit_price * self.quantity;

        let gross_pnl = match self.side {
            PositionSide::Long => exit_value - initial_value,
            PositionSide::Short => initial_value - exit_value,
        };
        let pnl =
            gross_pnl - total_commission - total_transaction_tax + self.unreinvested_dividends;

        let entry_value = self.entry_price * self.entry_quantity;
        let return_pct = if entry_value > 0.0 {
            (pnl / entry_value) * 100.0
        } else {
            0.0
        };

        Trade {
            side: self.side,
            entry_timestamp: self.entry_timestamp,
            exit_timestamp,
            entry_price: self.entry_price,
            exit_price,
            quantity: self.quantity,
            entry_quantity: self.entry_quantity,
            commission: total_commission,
            transaction_tax: total_transaction_tax,
            pnl,
            return_pct,
            dividend_income: self.dividend_income,
            unreinvested_dividends: self.unreinvested_dividends,
            tags: self.entry_signal.tags.clone(),
            entry_signal: self.entry_signal,
            exit_signal,
            is_partial: false,
            scale_sequence: 0,
        }
    }
}

/// A completed trade (closed position)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade direction
    pub side: PositionSide,

    /// Entry timestamp
    pub entry_timestamp: i64,

    /// Exit timestamp
    pub exit_timestamp: i64,

    /// Entry price
    pub entry_price: f64,

    /// Exit price
    pub exit_price: f64,

    /// Number of shares/units at exit
    pub quantity: f64,

    /// Number of shares/units at entry
    #[serde(default)]
    pub entry_quantity: f64,

    /// Total commission paid (entry + exit).
    pub commission: f64,

    /// Total transaction tax paid (entry + exit).
    ///
    /// Non-zero only when [`BacktestConfig::transaction_tax_pct`] is set.
    /// Deducted from P&L along with commission.
    ///
    /// [`BacktestConfig::transaction_tax_pct`]: crate::backtesting::BacktestConfig::transaction_tax_pct
    #[serde(default)]
    pub transaction_tax: f64,

    /// Realized P&L (after commission and transaction tax, including any unreinvested dividend income)
    pub pnl: f64,

    /// Return as percentage
    pub return_pct: f64,

    /// Dividend income received while this position was open
    pub dividend_income: f64,

    /// Dividend income that was NOT reinvested (i.e. remains as cash).
    /// Used internally for correct cash-accounting.
    #[serde(default)]
    pub unreinvested_dividends: f64,

    /// Signal that triggered entry
    pub entry_signal: Signal,

    /// Signal that triggered exit
    pub exit_signal: Signal,

    /// Tags inherited from the entry signal for subgroup analysis.
    ///
    /// Populated automatically from [`Signal::tags`] when the position closes.
    /// Query via `BacktestResult::trades_by_tag` and `metrics_by_tag`.
    ///
    /// Placed last so that JSON field order is consistent with [`Signal::tags`]
    /// (both appear after all other fields).
    #[serde(default)]
    pub tags: Vec<String>,

    /// `true` when this trade represents a **partial** close of a position
    /// (generated by [`Position::partial_close`] / a `ScaleOut` signal).
    ///
    /// `false` for full position closes and for the final close of a scaled
    /// position.
    #[serde(default)]
    pub is_partial: bool,

    /// Zero-based sequence number among the partial closes of this position.
    ///
    /// For the first `ScaleOut` on a given position this is `0`, the second is
    /// `1`, etc. Always `0` for non-partial trades.
    #[serde(default)]
    pub scale_sequence: usize,
}

impl Trade {
    /// Check if trade was profitable
    pub fn is_profitable(&self) -> bool {
        self.pnl > 0.0
    }

    /// Check if trade was a loss
    pub fn is_loss(&self) -> bool {
        self.pnl < 0.0
    }

    /// Check if this was a long trade
    pub fn is_long(&self) -> bool {
        matches!(self.side, PositionSide::Long)
    }

    /// Check if this was a short trade
    pub fn is_short(&self) -> bool {
        matches!(self.side, PositionSide::Short)
    }

    /// Get trade duration in seconds
    pub fn duration_secs(&self) -> i64 {
        self.exit_timestamp - self.entry_timestamp
    }

    /// Get entry value (cost basis)
    pub fn entry_value(&self) -> f64 {
        self.entry_price * self.entry_quantity
    }

    /// Get exit value
    pub fn exit_value(&self) -> f64 {
        self.exit_price * self.quantity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry_signal() -> Signal {
        Signal::long(1000, 100.0)
    }

    fn make_exit_signal() -> Signal {
        Signal::exit(2000, 110.0)
    }

    #[test]
    fn test_position_long_profit() {
        let pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            1.0, // $1 commission
            make_entry_signal(),
        );

        // Price goes up to 110
        let pnl = pos.unrealized_pnl(110.0);
        // (110 - 100) * 10 - 1 = 99
        assert!((pnl - 99.0).abs() < 0.01);
        assert!(pos.is_profitable(110.0));
    }

    #[test]
    fn test_position_long_loss() {
        let pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            1.0,
            make_entry_signal(),
        );

        // Price goes down to 90
        let pnl = pos.unrealized_pnl(90.0);
        // (90 - 100) * 10 - 1 = -101
        assert!((pnl - (-101.0)).abs() < 0.01);
        assert!(!pos.is_profitable(90.0));
    }

    #[test]
    fn test_position_short_profit() {
        let pos = Position::new(
            PositionSide::Short,
            1000,
            100.0,
            10.0,
            1.0,
            Signal::short(1000, 100.0),
        );

        // Price goes down to 90 (profit for short)
        let pnl = pos.unrealized_pnl(90.0);
        // (100 - 90) * 10 - 1 = 99
        assert!((pnl - 99.0).abs() < 0.01);
        assert!(pos.is_profitable(90.0));
    }

    #[test]
    fn test_position_close_to_trade() {
        let pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            1.0,
            make_entry_signal(),
        );

        let trade = pos.close(2000, 110.0, 1.0, make_exit_signal());

        assert_eq!(trade.entry_price, 100.0);
        assert_eq!(trade.exit_price, 110.0);
        assert_eq!(trade.quantity, 10.0);
        assert_eq!(trade.commission, 2.0); // 1 + 1
        // (110 - 100) * 10 - 2 = 98
        assert!((trade.pnl - 98.0).abs() < 0.01);
        assert!(trade.is_profitable());
        assert!(trade.is_long());
        assert_eq!(trade.duration_secs(), 1000);
    }

    #[test]
    fn test_credit_dividend_no_reinvest() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );
        pos.credit_dividend(5.0, 110.0, false);
        assert!((pos.dividend_income - 5.0).abs() < 1e-10);
        assert!((pos.quantity - 10.0).abs() < 1e-10); // unchanged
    }

    #[test]
    fn test_credit_dividend_reinvest() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );
        // $1/share × 10 shares = $10 income; reinvested at $110 → 10/110 ≈ 0.0909 new shares
        pos.credit_dividend(10.0, 110.0, true);
        assert!((pos.dividend_income - 10.0).abs() < 1e-10);
        let expected_qty = 10.0 + 10.0 / 110.0;
        assert!((pos.quantity - expected_qty).abs() < 1e-10);
    }

    #[test]
    fn test_credit_dividend_zero_price_no_reinvest() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );
        // reinvest=true but price=0.0 → should not divide by zero
        pos.credit_dividend(5.0, 0.0, true);
        assert!((pos.dividend_income - 5.0).abs() < 1e-10);
        assert!((pos.quantity - 10.0).abs() < 1e-10); // quantity unchanged
    }

    #[test]
    fn test_credit_dividend_short_is_negative_and_not_reinvested() {
        let mut pos = Position::new(
            PositionSide::Short,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        // Short positions pay dividends (negative cashflow).
        pos.credit_dividend(-5.0, 110.0, true);

        assert!((pos.dividend_income + 5.0).abs() < 1e-10);
        assert!((pos.quantity - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_trade_return_pct() {
        let pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        let trade = pos.close(2000, 110.0, 0.0, make_exit_signal());

        // Entry value = 1000, PnL = 100, return = 10%
        assert!((trade.return_pct - 10.0).abs() < 0.01);
    }

    // ── scale_in ─────────────────────────────────────────────────────────────

    #[test]
    fn test_scale_in_updates_weighted_avg_price() {
        // Entry: 10 shares @ $100 → entry_price = $100
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        // Scale in: 10 more shares @ $120
        pos.scale_in(120.0, 10.0, 0.0, 0.0);

        // Weighted avg = (100*10 + 120*10) / 20 = 2200/20 = $110
        assert!((pos.entry_price - 110.0).abs() < 1e-10);
        assert!((pos.quantity - 20.0).abs() < 1e-10);
        // entry_quantity must stay in sync for close_with_tax cost-basis arithmetic.
        assert!((pos.entry_quantity - 20.0).abs() < 1e-10);
        assert_eq!(pos.scale_in_count, 1);
    }

    #[test]
    fn test_scale_in_commission_accumulated() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            2.0, // initial commission
            make_entry_signal(),
        );

        pos.scale_in(110.0, 5.0, 1.5, 0.25); // commission=1.5, tax=0.25

        // Commission and tax stored in separate fields (not conflated).
        assert!((pos.entry_commission - 3.5).abs() < 1e-10); // 2.0 + 1.5
        assert!((pos.entry_transaction_tax - 0.25).abs() < 1e-10); // 0.0 + 0.25
    }

    #[test]
    fn test_scale_in_multiple_tranches() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        pos.scale_in(110.0, 10.0, 0.0, 0.0); // avg = (1000+1100)/20 = 105
        pos.scale_in(120.0, 10.0, 0.0, 0.0); // avg = (2100+1200)/30 = 110

        assert!((pos.entry_price - 110.0).abs() < 1e-10);
        assert!((pos.quantity - 30.0).abs() < 1e-10);
        assert_eq!(pos.scale_in_count, 2);
    }

    // ── partial_close ─────────────────────────────────────────────────────────

    #[test]
    fn test_partial_close_reduces_quantity() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        let trade = pos.partial_close(0.5, 2000, 110.0, 0.0, 0.0, make_exit_signal());

        // 50% of 10 shares closed = 5 shares remaining
        assert!((pos.quantity - 5.0).abs() < 1e-10);
        // entry_quantity must track quantity for close_with_tax cost-basis arithmetic.
        assert!((pos.entry_quantity - 5.0).abs() < 1e-10);
        assert!((trade.quantity - 5.0).abs() < 1e-10);
        assert!(trade.is_partial);
        assert_eq!(trade.scale_sequence, 0);
    }

    #[test]
    fn test_partial_close_pnl_is_proportional() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        // Close 50% at $120 → closed 5 shares, gross PnL = (120-100)*5 = $100
        // return_pct = pnl / (entry_price * qty_closed) = 100 / 500 = 20%
        let trade = pos.partial_close(0.5, 2000, 120.0, 0.0, 0.0, make_exit_signal());

        assert!((trade.pnl - 100.0).abs() < 1e-10);
        assert!((trade.return_pct - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_partial_close_sequence_increments() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            20.0,
            0.0,
            make_entry_signal(),
        );

        let t1 = pos.partial_close(0.25, 1000, 110.0, 0.0, 0.0, make_exit_signal());
        let t2 = pos.partial_close(0.25, 2000, 115.0, 0.0, 0.0, make_exit_signal());

        assert_eq!(t1.scale_sequence, 0);
        assert_eq!(t2.scale_sequence, 1);
        assert!(t1.is_partial);
        assert!(t2.is_partial);
        // After two 25% closes: 20 * 0.75 * 0.75 = 11.25 remaining
        assert!((pos.quantity - 11.25).abs() < 1e-10);
    }

    #[test]
    fn test_partial_close_full_fraction_closes_position() {
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        // fraction = 1.0 → qty_remaining = 0
        let trade = pos.partial_close(1.0, 2000, 110.0, 0.0, 0.0, make_exit_signal());

        assert!((pos.quantity - 0.0).abs() < 1e-10);
        assert!((trade.quantity - 10.0).abs() < 1e-10);
        assert!(trade.is_partial);
    }

    #[test]
    fn test_close_after_scale_in_uses_correct_cost_basis() {
        // Tests for entry_quantity not updated after scale_in causing
        // close_with_tax to compute gross_pnl = exit_value - (avg_price × orig_qty)
        // instead of exit_value - (avg_price × total_qty).
        //
        // Enter 10 @ $100, scale_in 10 @ $120 → avg=$110, total=20
        // Exit all 20 @ $115 with no commission.
        // Expected gross_pnl = (115 − 110) × 20 = $100.
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        pos.scale_in(120.0, 10.0, 0.0, 0.0);
        assert!((pos.entry_price - 110.0).abs() < 1e-10);

        let trade = pos.close(2000, 115.0, 0.0, make_exit_signal());

        // gross_pnl = (115 - 110) * 20 = 100
        assert!(
            (trade.pnl - 100.0).abs() < 1e-6,
            "expected pnl=100.0, got {:.6} (entry_quantity not synced after scale_in?)",
            trade.pnl
        );
        assert!((trade.quantity - 20.0).abs() < 1e-10);
        assert!(!trade.is_partial);
    }

    #[test]
    fn test_close_after_partial_close_uses_remaining_cost_basis() {
        // Tests for entry_quantity not updated after partial_close causing
        // the final close_with_tax to use the full original entry_quantity.
        //
        // Enter 20 @ $100, partial_close 50% @ $110, final close @ $120.
        // After partial: 10 shares remain, entry_quantity should = 10.
        // Expected final gross_pnl = (120 − 100) × 10 = $200.
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            20.0,
            0.0,
            make_entry_signal(),
        );

        let _partial = pos.partial_close(0.5, 1500, 110.0, 0.0, 0.0, make_exit_signal());
        assert!((pos.entry_quantity - 10.0).abs() < 1e-10);

        let trade = pos.close(2000, 120.0, 0.0, make_exit_signal());

        assert!(
            (trade.pnl - 200.0).abs() < 1e-6,
            "expected pnl=200.0, got {:.6} (entry_quantity not synced after partial_close?)",
            trade.pnl
        );
        assert!(!trade.is_partial);
    }

    #[test]
    fn test_scale_in_then_partial_close_full_exit() {
        // Pyramid: buy 10@100, add 10@120, exit half, exit rest
        let mut pos = Position::new(
            PositionSide::Long,
            1000,
            100.0,
            10.0,
            0.0,
            make_entry_signal(),
        );

        pos.scale_in(120.0, 10.0, 0.0, 0.0);
        // Entry price = (100*10 + 120*10) / 20 = 110, qty = 20

        // Scale out 50% at $130
        let partial_trade = pos.partial_close(0.5, 2000, 130.0, 0.0, 0.0, make_exit_signal());
        // closed 10 shares; gross PnL = (130 - 110) * 10 = $200
        assert!((partial_trade.pnl - 200.0).abs() < 1e-10);
        assert!((pos.quantity - 10.0).abs() < 1e-10);

        // Full close at $140
        let final_trade = pos.close(3000, 140.0, 0.0, make_exit_signal());
        // closed 10 shares; gross PnL = (140 - 110) * 10 = $300
        assert!((final_trade.pnl - 300.0).abs() < 1e-10);
        assert!(!final_trade.is_partial);
    }
}
