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

    /// Entry commission paid
    pub entry_commission: f64,

    /// Signal that triggered entry
    pub entry_signal: Signal,
}

impl Position {
    /// Create a new position
    pub fn new(
        side: PositionSide,
        entry_timestamp: i64,
        entry_price: f64,
        quantity: f64,
        entry_commission: f64,
        entry_signal: Signal,
    ) -> Self {
        Self {
            side,
            entry_timestamp,
            entry_price,
            quantity,
            entry_commission,
            entry_signal,
        }
    }

    /// Calculate current value at given price
    pub fn current_value(&self, current_price: f64) -> f64 {
        self.quantity * current_price
    }

    /// Calculate unrealized P&L at given price (before exit commission)
    pub fn unrealized_pnl(&self, current_price: f64) -> f64 {
        let gross_pnl = match self.side {
            PositionSide::Long => (current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - current_price) * self.quantity,
        };
        gross_pnl - self.entry_commission
    }

    /// Calculate unrealized return percentage
    pub fn unrealized_return_pct(&self, current_price: f64) -> f64 {
        let entry_value = self.entry_price * self.quantity;
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

    /// Close this position and create a Trade
    pub fn close(
        self,
        exit_timestamp: i64,
        exit_price: f64,
        exit_commission: f64,
        exit_signal: Signal,
    ) -> Trade {
        let total_commission = self.entry_commission + exit_commission;

        let gross_pnl = match self.side {
            PositionSide::Long => (exit_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - exit_price) * self.quantity,
        };
        let pnl = gross_pnl - total_commission;

        let entry_value = self.entry_price * self.quantity;
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
            commission: total_commission,
            pnl,
            return_pct,
            entry_signal: self.entry_signal,
            exit_signal,
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

    /// Number of shares/units
    pub quantity: f64,

    /// Total commission (entry + exit)
    pub commission: f64,

    /// Realized P&L (after commission)
    pub pnl: f64,

    /// Return as percentage
    pub return_pct: f64,

    /// Signal that triggered entry
    pub entry_signal: Signal,

    /// Signal that triggered exit
    pub exit_signal: Signal,
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
        self.entry_price * self.quantity
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
}
