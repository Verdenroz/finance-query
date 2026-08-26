use crate::backtesting::config::BacktestConfig;
use crate::backtesting::position::Position;
use crate::backtesting::signal::Signal;
use crate::models::chart::Candle;

use super::BacktestEngine;

/// Unsigned notional of the open position.
///
/// [`Position::current_value`] is negative for shorts, but margin rules are
/// written against the size of the borrowing, not its direction.
#[inline]
pub(super) fn gross_exposure(position: Option<&Position>, price: f64) -> f64 {
    position.map_or(0.0, |pos| pos.quantity * price)
}

/// Capital an entry may commit, including any margin loan.
#[inline]
pub(super) fn entry_buying_power(cash: f64, config: &BacktestConfig) -> f64 {
    cash.max(0.0) * config.max_leverage
}

/// Capital a scale-in may add on top of an open position.
///
/// Unlevered this is plain cash; levered it is the unused portion of the
/// exposure ceiling.
#[inline]
pub(super) fn add_buying_power(
    cash: f64,
    position: &Position,
    price: f64,
    config: &BacktestConfig,
) -> f64 {
    if config.max_leverage <= 1.0 {
        return cash;
    }
    let equity = cash + position.current_value(price) + position.unreinvested_dividends;
    (equity * config.max_leverage - gross_exposure(Some(position), price)).max(0.0)
}

impl BacktestEngine {
    /// Charge one bar of borrowed-capital cost against cash and the position.
    ///
    /// Shorts pay to borrow the shares; a debit cash balance pays margin
    /// interest. The fee is attributed to the position so it leaves via that
    /// trade's P&L rather than vanishing from cash.
    #[inline]
    pub(super) fn accrue_financing(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        candle: &Candle,
    ) {
        if self.config.short_borrow_rate <= 0.0 && self.config.margin_interest_rate <= 0.0 {
            return;
        }
        let Some(pos) = position.as_mut() else {
            return;
        };

        let per_bar = 1.0 / self.config.bars_per_year;
        let borrow = if pos.is_short() {
            pos.quantity * candle.close * self.config.short_borrow_rate * per_bar
        } else {
            0.0
        };
        let interest = (-*cash).max(0.0) * self.config.margin_interest_rate * per_bar;

        let fee = borrow + interest;
        if fee > 0.0 {
            *cash -= fee;
            pos.accrue_financing_cost(fee);
        }
    }

    /// Liquidation signal when equity has fallen through the maintenance
    /// requirement.
    ///
    /// Equity is recomputed here rather than taken from the bar's opening
    /// snapshot, so the check sees this bar's financing accrual and dividend
    /// credit.
    #[inline]
    pub(super) fn check_margin_call(
        &self,
        position: Option<&Position>,
        cash: f64,
        candle: &Candle,
    ) -> Option<Signal> {
        if self.config.max_leverage <= 1.0 {
            return None;
        }
        let pos = position?;

        let gross = gross_exposure(Some(pos), candle.close);
        if gross <= 0.0 {
            return None;
        }

        let equity = cash + pos.current_value(candle.close) + pos.unreinvested_dividends;
        if equity < gross * self.config.maintenance_margin_pct {
            return Some(
                Signal::exit(candle.timestamp, candle.close)
                    .with_reason("Margin call: equity below maintenance margin requirement"),
            );
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::engine::fixtures::{
        EnterLongHold, EnterShortHold, EnterShortScaleIn, make_candles,
    };
    use crate::backtesting::result::BacktestResult;
    use crate::models::chart::Dividend;

    fn levered_config(max_leverage: f64) -> BacktestConfig {
        BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .max_leverage(max_leverage)
            .maintenance_margin_pct(0.25)
            .close_at_end(false)
            .build()
            .unwrap()
    }

    fn margin_calls(result: &BacktestResult) -> usize {
        result
            .trades
            .iter()
            .filter(|t| {
                t.exit_signal
                    .reason
                    .as_deref()
                    .is_some_and(|r| r.contains("Margin call"))
            })
            .count()
    }

    fn long_position(quantity: f64, entry_price: f64) -> Position {
        Position::new(
            crate::backtesting::position::PositionSide::Long,
            0,
            entry_price,
            quantity,
            0.0,
            Signal::long(0, entry_price),
        )
    }

    #[test]
    fn test_add_buying_power_is_plain_cash_when_unlevered() {
        let config = BacktestConfig::default();
        let pos = long_position(50.0, 100.0);
        assert_eq!(add_buying_power(5_000.0, &pos, 100.0, &config), 5_000.0);
    }

    #[test]
    fn test_add_buying_power_is_unused_exposure_when_levered() {
        let config = levered_config(2.0);
        let pos = long_position(50.0, 100.0);
        assert_eq!(add_buying_power(5_000.0, &pos, 100.0, &config), 15_000.0);
    }

    #[test]
    fn test_add_buying_power_floors_at_zero_when_fully_committed() {
        let config = levered_config(2.0);
        let pos = long_position(300.0, 100.0);
        assert_eq!(add_buying_power(-20_000.0, &pos, 100.0, &config), 0.0);
    }

    #[test]
    fn test_margin_call_liquidates_a_levered_position_on_a_crash() {
        let candles = make_candles(&[100.0, 100.0, 100.0, 85.0, 85.0]);
        let result = BacktestEngine::new(levered_config(3.0))
            .run("TEST", &candles, EnterLongHold)
            .unwrap();

        assert_eq!(result.trades.len(), 1);
        assert_eq!(margin_calls(&result), 1);
        assert!(result.open_position.is_none());
        assert_eq!(result.trades[0].exit_timestamp, 3);
    }

    #[test]
    fn test_no_margin_call_at_default_leverage() {
        let candles = make_candles(&[100.0, 100.0, 100.0, 85.0, 85.0]);
        let result = BacktestEngine::new(levered_config(1.0))
            .run("TEST", &candles, EnterLongHold)
            .unwrap();

        assert_eq!(margin_calls(&result), 0);
        assert!(result.open_position.is_some());
    }

    #[test]
    fn test_margin_call_liquidates_a_levered_short_when_price_rises() {
        let candles = make_candles(&[100.0, 100.0, 100.0, 115.0, 115.0]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .max_leverage(3.0)
            .maintenance_margin_pct(0.25)
            .close_at_end(false)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterShortHold)
            .unwrap();

        assert_eq!(margin_calls(&result), 1);
        assert_eq!(result.trades[0].exit_timestamp, 3);
        assert!(result.open_position.is_none());
    }

    #[test]
    fn test_margin_call_fill_pays_exit_slippage() {
        let candles = make_candles(&[100.0, 100.0, 100.0, 85.0, 85.0]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.01)
            .max_leverage(3.0)
            .maintenance_margin_pct(0.25)
            .close_at_end(false)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterLongHold)
            .unwrap();

        assert_eq!(margin_calls(&result), 1);
        assert!(result.trades[0].exit_price < candles[3].close);
    }

    #[test]
    fn test_margin_call_accounts_for_the_current_bar_dividend() {
        let candles = make_candles(&[100.0, 100.0, 100.0, 85.0, 85.0]);
        let dividends = vec![Dividend {
            timestamp: 3,
            amount: 20.0,
            provider_id: None,
        }];
        let engine = BacktestEngine::new(levered_config(3.0));

        let without = engine.run("TEST", &candles, EnterLongHold).unwrap();
        let with = engine
            .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
            .unwrap();

        assert_eq!(margin_calls(&without), 1);
        assert_eq!(margin_calls(&with), 0);
    }

    #[test]
    fn test_short_borrow_cost_accrues_and_reduces_pnl() {
        let candles = make_candles(&[100.0; 20]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .short_borrow_rate(0.10)
            .bars_per_year(252.0)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterShortHold)
            .unwrap();

        assert!(result.metrics.total_financing_cost > 0.0);
        assert!((result.trades[0].pnl + result.trades[0].financing_cost).abs() < 1e-9);
    }

    #[test]
    fn test_margin_interest_accrues_on_a_levered_long() {
        let candles = make_candles(&[100.0; 20]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .max_leverage(2.0)
            .margin_interest_rate(0.10)
            .bars_per_year(252.0)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterLongHold)
            .unwrap();

        assert!(result.metrics.total_financing_cost > 0.0);
        assert!(result.trades[0].pnl < 0.0);
    }

    #[test]
    fn test_max_leverage_used_reports_the_exposure_actually_taken() {
        let candles = make_candles(&[100.0; 20]);

        let flat = BacktestEngine::new(levered_config(2.0))
            .run("TEST", &candles, EnterLongHold)
            .unwrap();
        assert!((flat.max_leverage_used - 2.0).abs() < 0.01);

        let half = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .position_size_pct(0.5)
            .max_leverage(2.0)
            .close_at_end(false)
            .build()
            .unwrap();
        let partial = BacktestEngine::new(half)
            .run("TEST", &candles, EnterLongHold)
            .unwrap();
        assert!((partial.max_leverage_used - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_short_scale_in_cannot_breach_the_leverage_ceiling() {
        let candles = make_candles(&[100.0; 6]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .max_leverage(2.0)
            .maintenance_margin_pct(0.25)
            .close_at_end(false)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterShortScaleIn)
            .unwrap();

        let pos = result.open_position.expect("short stays open");
        assert!((pos.quantity - 200.0).abs() < 1e-9);
        assert!(result.max_leverage_used <= 2.0 + 1e-9);
    }

    #[test]
    fn test_max_leverage_used_accounts_for_the_current_bar_dividend() {
        let candles = make_candles(&[100.0; 6]);
        let dividends = vec![Dividend {
            timestamp: 1,
            amount: 5.0,
            provider_id: None,
        }];
        let engine = BacktestEngine::new(levered_config(2.0));

        let without = engine.run("TEST", &candles, EnterLongHold).unwrap();
        let with = engine
            .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
            .unwrap();

        assert!((without.max_leverage_used - 2.0).abs() < 0.01);
        assert!(with.max_leverage_used < without.max_leverage_used);
    }

    #[test]
    fn test_no_financing_cost_at_default_rates() {
        let candles = make_candles(&[100.0; 20]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .build()
            .unwrap();
        let result = BacktestEngine::new(config)
            .run("TEST", &candles, EnterShortHold)
            .unwrap();

        assert_eq!(result.metrics.total_financing_cost, 0.0);
    }

    #[test]
    fn test_accounting_invariant_holds_with_financing() {
        let candles = make_candles(&[100.0; 20]);

        for (allow_short, leverage) in [(true, 1.0), (false, 2.0)] {
            let config = BacktestConfig::builder()
                .initial_capital(10_000.0)
                .commission_pct(0.001)
                .allow_short(allow_short)
                .max_leverage(leverage)
                .short_borrow_rate(0.10)
                .margin_interest_rate(0.10)
                .close_at_end(true)
                .build()
                .unwrap();
            let engine = BacktestEngine::new(config);
            let result = if allow_short {
                engine.run("TEST", &candles, EnterShortHold).unwrap()
            } else {
                engine.run("TEST", &candles, EnterLongHold).unwrap()
            };

            let sum_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
            let expected = 10_000.0 + sum_pnl;
            assert!(
                (result.final_equity - expected).abs() < 1e-6,
                "final_equity {:.6} != initial + sum(pnl) {expected:.6}",
                result.final_equity,
            );
        }
    }
}
