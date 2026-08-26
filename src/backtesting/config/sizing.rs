//! Position sizing: how much capital an entry commits.

use super::BacktestConfig;

impl BacktestConfig {
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
    /// [`commission_fn`]: Self::commission_fn
    pub fn calculate_position_size(&self, available_capital: f64, price: f64) -> f64 {
        let capital_to_use = available_capital * self.position_size_pct;

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

#[cfg(test)]
mod tests {
    use super::*;

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
        // But adjusted for entry + exit commission: 5000 / 1.002 = 4990.019960...
        // So shares = 4990.019960 / 100 = 49.90...
        let size = config.calculate_position_size(10_000.0, 100.0);
        let expected = 5000.0 / 1.002 / 100.0;
        assert!((size - expected).abs() < 0.01);
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
}
