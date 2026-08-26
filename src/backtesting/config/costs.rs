//! Trading friction: commission, slippage, spread, and transaction tax.

use std::fmt;
use std::sync::Arc;

use super::BacktestConfig;

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

impl BacktestConfig {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
