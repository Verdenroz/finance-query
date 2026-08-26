//! Position sizing: how much capital an entry commits.

use serde::{Deserialize, Serialize};

use super::BacktestConfig;

/// How an entry's size is derived from available equity.
///
/// Every scheme targets a fraction of equity clamped to the risk budget
/// ([`BacktestConfig::position_size_pct`], raised by
/// [`BacktestConfig::max_leverage`] when levered), and falls back to that budget
/// when its inputs are unavailable.
///
/// Scale-in signals carry an explicit fraction of their own and are not sized
/// by the active scheme.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum PositionSizing {
    /// Commit [`BacktestConfig::position_size_pct`] of equity on every entry.
    #[default]
    FixedFraction,

    /// Risk a fixed fraction of equity across an ATR-derived stop distance.
    ///
    /// Wider ranges produce smaller positions, holding currency risk per trade
    /// roughly constant.
    Atr {
        /// Fraction of equity to risk if price moves `atr_multiple` ATRs against
        /// the entry (0.0 - 1.0).
        risk_pct: f64,
        /// Lookback period for the ATR computation.
        atr_period: usize,
        /// Stop distance as a multiple of ATR.
        atr_multiple: f64,
    },

    /// Scale exposure inversely to realized volatility.
    VolatilityTarget {
        /// Target per-bar volatility contribution as a fraction (`0.01` = 1%).
        target_vol_pct: f64,
        /// Trailing bars used to estimate realized volatility.
        lookback: usize,
    },

    /// Size by a fraction of the Kelly-optimal bet implied by recent trades.
    ///
    /// A trade count is not a bar count, so unlike the other schemes this one
    /// cannot extend the warmup period. Entries before the window holds both a
    /// win and a loss fall back to the risk budget.
    FractionalKelly {
        /// Multiplier on the full Kelly fraction (`0.5` = half-Kelly).
        kelly_fraction: f64,
        /// Trailing fully-closed trades used to estimate win rate and payoff
        /// ratio. Partial closes from `Signal::scale_out` are excluded, so one
        /// entry contributes one observation.
        lookback_trades: usize,
    },
}

/// Market and trade-history inputs a [`PositionSizing`] scheme reads at entry.
///
/// A `None` field means the engine had no value to supply, and the scheme falls
/// back to [`BacktestConfig::position_size_pct`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SizingContext {
    /// ATR at the entry bar.
    pub atr: Option<f64>,
    /// Realized per-bar return volatility over the scheme's lookback.
    pub recent_volatility: Option<f64>,
    /// Win rate over the trailing closed-trade window (0.0 - 1.0).
    pub win_rate: Option<f64>,
    /// Mean win divided by mean loss, both as absolute return fractions.
    pub payoff_ratio: Option<f64>,
}

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
        self.size_from_fraction(
            available_capital,
            price,
            self.position_size_pct * self.max_leverage,
        )
    }

    /// Calculate position size under the active [`PositionSizing`] scheme.
    ///
    /// The scheme's own fraction is clamped to the risk budget
    /// ([`position_size_pct`] times [`max_leverage`]), so leverage raises the
    /// ceiling a scheme may reach rather than multiplying what it asked for.
    /// `price` carries the same fully-adjusted requirement as
    /// [`calculate_position_size`].
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
        let fraction = self.sizing_fraction(price, ctx);
        self.size_from_fraction(available_capital, price, fraction)
    }

    fn sizing_fraction(&self, price: f64, ctx: &SizingContext) -> f64 {
        let budget = self.position_size_pct * self.max_leverage;
        let base = match self.position_sizing {
            PositionSizing::FixedFraction => budget,
            PositionSizing::Atr {
                risk_pct,
                atr_multiple,
                ..
            } => match ctx.atr {
                Some(atr) if atr > 0.0 && atr_multiple > 0.0 && price > 0.0 => {
                    (risk_pct * price) / (atr_multiple * atr)
                }
                _ => budget,
            },
            PositionSizing::VolatilityTarget { target_vol_pct, .. } => {
                match ctx.recent_volatility {
                    Some(vol) if vol > 0.0 => target_vol_pct / vol,
                    _ => budget,
                }
            }
            PositionSizing::FractionalKelly { kelly_fraction, .. } => {
                match (ctx.win_rate, ctx.payoff_ratio) {
                    (Some(win_rate), Some(payoff)) if payoff > 0.0 => {
                        let kelly = win_rate - (1.0 - win_rate) / payoff;
                        kelly_fraction * kelly
                    }
                    _ => budget,
                }
            }
        };

        base.clamp(0.0, budget)
    }

    /// Bars of history the active [`PositionSizing`] scheme needs before it can
    /// size an entry from real data.
    ///
    /// The engine folds this into the strategy's own warmup so an early entry
    /// cannot silently fall back to [`position_size_pct`].
    /// [`PositionSizing::FractionalKelly`] returns `0` because its window counts
    /// closed trades, which no number of bars guarantees.
    ///
    /// [`position_size_pct`]: Self::position_size_pct
    pub fn sizing_warmup(&self) -> usize {
        match self.position_sizing {
            PositionSizing::Atr { atr_period, .. } => atr_period + 1,
            PositionSizing::VolatilityTarget { lookback, .. } => lookback + 1,
            PositionSizing::FixedFraction | PositionSizing::FractionalKelly { .. } => 0,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn scheme(sizing: PositionSizing, position_size_pct: f64) -> BacktestConfig {
        BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(position_size_pct)
            .position_sizing(sizing)
            .build()
            .unwrap()
    }

    #[test]
    fn test_fixed_fraction_context_matches_plain_sizing() {
        let config = scheme(PositionSizing::FixedFraction, 0.5);
        let with_ctx =
            config.calculate_position_size_with_context(10_000.0, 100.0, &SizingContext::default());
        let plain = config.calculate_position_size(10_000.0, 100.0);
        assert!((with_ctx - plain).abs() < 1e-12);
    }

    #[test]
    fn test_atr_sizing_uses_risk_over_stop_distance() {
        let config = scheme(
            PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            },
            1.0,
        );
        let ctx = SizingContext {
            atr: Some(2.0),
            ..SizingContext::default()
        };
        // (0.02 * 100) / (2.0 * 2.0) = 0.5 of equity
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert!((size - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_atr_sizing_falls_back_without_atr() {
        let config = scheme(
            PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            },
            0.4,
        );
        let size =
            config.calculate_position_size_with_context(10_000.0, 100.0, &SizingContext::default());
        assert!((size - config.calculate_position_size(10_000.0, 100.0)).abs() < 1e-12);
    }

    #[test]
    fn test_volatility_target_scales_inversely_to_volatility() {
        let config = scheme(
            PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            },
            1.0,
        );
        let calm = SizingContext {
            recent_volatility: Some(0.02),
            ..SizingContext::default()
        };
        let wild = SizingContext {
            recent_volatility: Some(0.04),
            ..SizingContext::default()
        };
        let calm_size = config.calculate_position_size_with_context(10_000.0, 100.0, &calm);
        let wild_size = config.calculate_position_size_with_context(10_000.0, 100.0, &wild);
        assert!((calm_size - 50.0).abs() < 1e-9);
        assert!((wild_size - 25.0).abs() < 1e-9);
    }

    #[test]
    fn test_volatility_target_falls_back_without_data() {
        let config = scheme(
            PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            },
            0.3,
        );
        let size =
            config.calculate_position_size_with_context(10_000.0, 100.0, &SizingContext::default());
        assert!((size - config.calculate_position_size(10_000.0, 100.0)).abs() < 1e-12);
    }

    #[test]
    fn test_fractional_kelly_matches_formula() {
        let config = scheme(
            PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 20,
            },
            1.0,
        );
        let ctx = SizingContext {
            win_rate: Some(0.6),
            payoff_ratio: Some(2.0),
            ..SizingContext::default()
        };
        // kelly = 0.6 - 0.4 / 2.0 = 0.4; half-Kelly = 0.2
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert!((size - 20.0).abs() < 1e-9);
    }

    #[test]
    fn test_fractional_kelly_negative_edge_sizes_to_zero() {
        let config = scheme(
            PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 20,
            },
            1.0,
        );
        let ctx = SizingContext {
            win_rate: Some(0.3),
            payoff_ratio: Some(1.0),
            ..SizingContext::default()
        };
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert_eq!(size, 0.0);
    }

    #[test]
    fn test_leverage_raises_the_budget_without_scaling_the_scheme() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(1.0)
            .max_leverage(3.0)
            .position_sizing(PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            })
            .build()
            .unwrap();

        let ctx = SizingContext {
            atr: Some(2.0),
            ..SizingContext::default()
        };
        // (0.02 * 100) / (2.0 * 2.0) = 0.5 of equity, leverage or not.
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert!((size - 50.0).abs() < 1e-9);

        let tight_stop = SizingContext {
            atr: Some(0.1),
            ..SizingContext::default()
        };
        // Asks for 10x equity, capped at the 3x budget.
        let capped = config.calculate_position_size_with_context(10_000.0, 100.0, &tight_stop);
        assert!((capped - 300.0).abs() < 1e-9);
    }

    #[test]
    fn test_leverage_falls_back_to_the_full_budget() {
        let config = BacktestConfig::builder()
            .commission_pct(0.0)
            .position_size_pct(0.5)
            .max_leverage(2.0)
            .position_sizing(PositionSizing::VolatilityTarget {
                target_vol_pct: 0.01,
                lookback: 20,
            })
            .build()
            .unwrap();

        let size =
            config.calculate_position_size_with_context(10_000.0, 100.0, &SizingContext::default());
        assert!((size - config.calculate_position_size(10_000.0, 100.0)).abs() < 1e-12);
        assert!((size - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_fractional_kelly_falls_back_without_history() {
        let config = scheme(
            PositionSizing::FractionalKelly {
                kelly_fraction: 0.5,
                lookback_trades: 20,
            },
            0.25,
        );
        let size =
            config.calculate_position_size_with_context(10_000.0, 100.0, &SizingContext::default());
        assert!((size - config.calculate_position_size(10_000.0, 100.0)).abs() < 1e-12);
    }

    #[test]
    fn test_scheme_cannot_exceed_the_risk_budget() {
        let config = scheme(
            PositionSizing::Atr {
                risk_pct: 0.02,
                atr_period: 14,
                atr_multiple: 2.0,
            },
            0.1,
        );
        // A tiny ATR asks for 20x equity; the budget caps it at 10%.
        let ctx = SizingContext {
            atr: Some(0.005),
            ..SizingContext::default()
        };
        let size = config.calculate_position_size_with_context(10_000.0, 100.0, &ctx);
        assert!((size - config.calculate_position_size(10_000.0, 100.0)).abs() < 1e-12);
        assert!((size - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_sizing_warmup_per_scheme() {
        assert_eq!(BacktestConfig::default().sizing_warmup(), 0);
        assert_eq!(
            scheme(
                PositionSizing::Atr {
                    risk_pct: 0.02,
                    atr_period: 14,
                    atr_multiple: 2.0,
                },
                1.0,
            )
            .sizing_warmup(),
            15
        );
        assert_eq!(
            scheme(
                PositionSizing::VolatilityTarget {
                    target_vol_pct: 0.01,
                    lookback: 20,
                },
                1.0,
            )
            .sizing_warmup(),
            21
        );
        assert_eq!(
            scheme(
                PositionSizing::FractionalKelly {
                    kelly_fraction: 0.5,
                    lookback_trades: 20,
                },
                1.0,
            )
            .sizing_warmup(),
            0
        );
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
