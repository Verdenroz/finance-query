use crate::backtesting::config::SizingContext;
use crate::backtesting::position::{Position, PositionSide, Trade};
use crate::backtesting::signal::{Signal, SignalDirection};
use crate::models::chart::Candle;

use super::BacktestEngine;
use super::margin;

// The `#[inline]` markers below are load-bearing: `simulate`'s per-candle loop
// lives in a sibling module and `[profile.bench]` builds without LTO.

impl BacktestEngine {
    /// Execute a signal, modifying position and cash
    #[inline]
    pub(super) fn execute_signal(
        &self,
        signal: &Signal,
        candle: &Candle,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        sizing: &SizingContext,
    ) -> bool {
        match signal.direction {
            SignalDirection::Long => {
                if position.is_some() {
                    return false; // Already have a position
                }
                self.open_position(position, cash, candle, signal, sizing)
            }
            SignalDirection::Short => {
                if position.is_some() {
                    return false; // Already have a position
                }
                if !self.config.allow_short {
                    return false; // Shorts not allowed
                }
                self.open_position(position, cash, candle, signal, sizing)
            }
            SignalDirection::Exit => {
                if position.is_none() {
                    return false; // No position to exit
                }
                self.close_position(position, cash, trades, candle, signal)
            }
            SignalDirection::ScaleIn => self.scale_into_position(position, cash, signal, candle),
            SignalDirection::ScaleOut => {
                self.scale_out_position(position, cash, trades, signal, candle)
            }
            SignalDirection::Hold => false,
        }
    }

    /// Add to an existing open position (pyramid / scale in).
    ///
    /// Allocates `signal.scale_fraction` of current portfolio equity to additional
    /// shares at the next-bar fill price. Updates the position's weighted-average
    /// entry price. No-op when no position is open.
    fn scale_into_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        signal: &Signal,
        candle: &Candle,
    ) -> bool {
        let fraction = signal.scale_fraction.unwrap_or(0.0).clamp(0.0, 1.0);
        if fraction <= 0.0 {
            return false;
        }

        let pos = match position.as_mut() {
            Some(p) => p,
            None => return false,
        };

        let is_long = pos.is_long();
        let fill_price_slipped = self.config.apply_entry_slippage(candle.open, is_long);
        let fill_price = self.config.apply_entry_spread(fill_price_slipped, is_long);

        // Allocate `fraction` of current portfolio equity to the additional tranche.
        let equity = *cash + pos.current_value(candle.open) + pos.unreinvested_dividends;
        let additional_value = equity * fraction;
        let additional_qty = if fill_price > 0.0 {
            additional_value / fill_price
        } else {
            return false;
        };

        if additional_qty <= 0.0 {
            return false;
        }

        let commission = self.config.calculate_commission(additional_qty, fill_price);
        let entry_tax = self
            .config
            .calculate_transaction_tax(additional_value, is_long);
        let total_cost = if is_long {
            additional_value + commission + entry_tax
        } else {
            commission
        };

        if total_cost > margin::add_buying_power(*cash, pos, candle.open, &self.config) {
            return false; // Not enough buying power
        }

        if is_long {
            *cash -= additional_value + commission + entry_tax;
        } else {
            *cash += additional_value - commission;
        }

        pos.scale_in(fill_price, additional_qty, commission, entry_tax);
        true
    }

    /// Partially or fully close an existing open position (scale out).
    ///
    /// Closes `signal.scale_fraction` of the current position quantity at the
    /// next-bar fill price.  A fraction of `1.0` is equivalent to a full
    /// [`Signal::exit`] and delegates to [`close_position`](Self::close_position).
    /// No-op when no position is open.
    fn scale_out_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        signal: &Signal,
        candle: &Candle,
    ) -> bool {
        let fraction = signal.scale_fraction.unwrap_or(0.0).clamp(0.0, 1.0);
        if fraction <= 0.0 {
            return false;
        }

        // Full close — delegate to the standard exit path so all bookkeeping
        // (cash credit, HWM reset, re-evaluation) is handled identically.
        if fraction >= 1.0 {
            return self.close_position(position, cash, trades, candle, signal);
        }

        let pos = match position.as_mut() {
            Some(p) => p,
            None => return false,
        };

        let is_long = pos.is_long();
        let exit_price_slipped = self.config.apply_exit_slippage(candle.open, is_long);
        let exit_price = self.config.apply_exit_spread(exit_price_slipped, is_long);
        let qty_closed = pos.quantity * fraction;
        let commission = self.config.calculate_commission(qty_closed, exit_price);
        let exit_tax = self
            .config
            .calculate_transaction_tax(exit_price * qty_closed, !is_long);

        let trade = pos.partial_close(
            fraction,
            candle.timestamp,
            exit_price,
            commission,
            exit_tax,
            signal.clone(),
        );

        // `commission` and `exit_tax` here are the exit-side cash flows only.
        // `trade.commission` / `trade.transaction_tax` also include the proportional
        // entry cost slice (for P&L reporting), but those were already debited from
        // cash at entry time and must not be debited again here.
        if trade.is_long() {
            *cash += trade.exit_value() - commission + trade.unreinvested_dividends;
        } else {
            *cash -= trade.exit_value() + commission + exit_tax - trade.unreinvested_dividends;
        }
        trades.push(trade);
        true
    }

    /// Open a new position at `candle.open` (market fill).
    fn open_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        candle: &Candle,
        signal: &Signal,
        sizing: &SizingContext,
    ) -> bool {
        self.open_position_at_price(position, cash, candle, signal, candle.open, sizing)
    }

    /// Open a new position at an explicit fill price.
    ///
    /// Used for pending limit/stop order fills where the computed order price
    /// (with gap guard) is the fill price rather than the next bar's open.
    #[inline]
    pub(super) fn open_position_at_price(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        candle: &Candle,
        signal: &Signal,
        fill_price_raw: f64,
        sizing: &SizingContext,
    ) -> bool {
        let is_long = matches!(signal.direction, SignalDirection::Long);
        let entry_price_slipped = self.config.apply_entry_slippage(fill_price_raw, is_long);
        let entry_price = self.config.apply_entry_spread(entry_price_slipped, is_long);
        let quantity = self
            .config
            .calculate_position_size_with_context(*cash, entry_price, sizing);

        if quantity <= 0.0 {
            return false; // Not enough capital
        }

        let entry_value = entry_price * quantity;
        let commission = self.config.calculate_commission(quantity, entry_price);
        // Tax on buy orders only: long entries are buys
        let entry_tax = self.config.calculate_transaction_tax(entry_value, is_long);

        let buying_power = margin::entry_buying_power(*cash, &self.config);
        if is_long {
            if entry_value + commission + entry_tax > buying_power {
                return false; // Not enough buying power including commission and tax
            }
        } else {
            if commission > *cash {
                return false; // Not enough cash to pay entry commission
            }
            if entry_value > buying_power {
                return false; // Notional exceeds leveraged buying power
            }
        }

        let side = if is_long {
            PositionSide::Long
        } else {
            PositionSide::Short
        };

        if is_long {
            *cash -= entry_value + commission + entry_tax;
        } else {
            *cash += entry_value - commission;
        }
        *position = Some(Position::new_with_tax(
            side,
            candle.timestamp,
            entry_price,
            quantity,
            commission,
            entry_tax,
            signal.clone(),
        ));

        true
    }

    /// Close an existing position at the next bar's open (used for strategy-signal exits).
    fn close_position(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        candle: &Candle,
        signal: &Signal,
    ) -> bool {
        self.close_position_at(position, cash, trades, candle, candle.open, signal)
    }

    /// Close an existing position at an explicit `fill_price`.
    ///
    /// Used for intrabar SL/TP/trailing-stop exits where the fill price is the
    /// computed stop/TP level (with gap guard) rather than the next bar's open.
    #[inline]
    pub(super) fn close_position_at(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        candle: &Candle,
        fill_price: f64,
        signal: &Signal,
    ) -> bool {
        let pos = match position.take() {
            Some(p) => p,
            None => return false,
        };

        let exit_price_slipped = self.config.apply_exit_slippage(fill_price, pos.is_long());
        let exit_price = self
            .config
            .apply_exit_spread(exit_price_slipped, pos.is_long());
        let exit_commission = self.config.calculate_commission(pos.quantity, exit_price);
        // Tax on buy orders only: short covers are buys
        let exit_tax = self
            .config
            .calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());

        let trade = pos.close_with_tax(
            candle.timestamp,
            exit_price,
            exit_commission,
            exit_tax,
            signal.clone(),
        );

        if trade.is_long() {
            *cash += trade.exit_value() - exit_commission + trade.unreinvested_dividends;
        } else {
            *cash -= trade.exit_value() + exit_commission + exit_tax - trade.unreinvested_dividends;
        }
        trades.push(trade);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::fixtures::*;
    use crate::backtesting::strategy::{Strategy, StrategyContext};
    use crate::indicators::Indicator;

    // ── Position scaling integration tests ───────────────────────────────────

    #[test]
    fn test_scale_in_adds_to_position() {
        // 4 candles: entry bar 0, fill bar 1, scale-in bar 1, fill bar 2, exit bar 2, fill bar 3
        let prices = [100.0, 100.0, 110.0, 120.0, 120.0];
        let candles = make_candles(&prices);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(true)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine.run("TEST", &candles, EnterScaleInExit).unwrap();

        // Exactly one closed trade (from the final exit)
        assert_eq!(result.trades.len(), 1);
        let trade = &result.trades[0];
        assert!(!trade.is_partial);
        // Position was scaled in, so quantity > initial allocation
        assert!(trade.quantity > 0.0);
        // Strategy ran; equity curve has entries
        assert!(!result.equity_curve.is_empty());
        // Scale-in signal recorded
        let scale_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| matches!(s.direction, SignalDirection::ScaleIn))
            .collect();
        assert!(!scale_signals.is_empty());
    }

    #[test]
    fn test_scale_out_produces_partial_trade() {
        let prices = [100.0, 100.0, 110.0, 120.0, 120.0];
        let candles = make_candles(&prices);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(true)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine.run("TEST", &candles, EnterScaleOutExit).unwrap();

        // Two trades: partial close + final close
        assert!(result.trades.len() >= 2);
        let partial = result
            .trades
            .iter()
            .find(|t| t.is_partial)
            .expect("expected at least one partial trade");
        assert_eq!(partial.scale_sequence, 0);

        let final_trade = result.trades.iter().find(|t| !t.is_partial);
        assert!(final_trade.is_some());
    }

    #[test]
    fn test_scale_out_full_fraction_is_equivalent_to_exit() {
        /// Strategy: enter on bar 0, scale_out(1.0) on bar 1 — should fully close.
        #[derive(Clone)]
        struct EnterScaleOutFull;
        impl Strategy for EnterScaleOutFull {
            fn name(&self) -> &str {
                "EnterScaleOutFull"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, ctx: &StrategyContext) -> Signal {
                match ctx.index {
                    0 => Signal::long(ctx.timestamp(), ctx.close()),
                    1 if ctx.has_position() => Signal::scale_out(1.0, ctx.timestamp(), ctx.close()),
                    _ => Signal::hold(),
                }
            }
        }

        let prices = [100.0, 100.0, 120.0, 120.0];
        let candles = make_candles(&prices);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result_scale = engine.run("TEST", &candles, EnterScaleOutFull).unwrap();

        // Full scale_out(1.0) should close position, leaving no open position
        assert!(result_scale.open_position.is_none());
        assert!(!result_scale.trades.is_empty());

        // Compare against a plain Exit strategy for identical P&L
        #[derive(Clone)]
        struct EnterThenExit;
        impl Strategy for EnterThenExit {
            fn name(&self) -> &str {
                "EnterThenExit"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, ctx: &StrategyContext) -> Signal {
                match ctx.index {
                    0 => Signal::long(ctx.timestamp(), ctx.close()),
                    1 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                    _ => Signal::hold(),
                }
            }
        }

        let engine2 = BacktestEngine::new(config);
        let result_exit = engine2.run("TEST", &candles, EnterThenExit).unwrap();

        let pnl_scale: f64 = result_scale.trades.iter().map(|t| t.pnl).sum();
        let pnl_exit: f64 = result_exit.trades.iter().map(|t| t.pnl).sum();
        assert!(
            (pnl_scale - pnl_exit).abs() < 1e-6,
            "scale_out(1.0) PnL {pnl_scale:.6} should equal exit PnL {pnl_exit:.6}"
        );
    }

    #[test]
    fn test_scale_in_noop_without_position() {
        /// Strategy: scale_in on bar 0 (no position open) — should be ignored.
        #[derive(Clone)]
        struct ScaleInNoPos;
        impl Strategy for ScaleInNoPos {
            fn name(&self) -> &str {
                "ScaleInNoPos"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, ctx: &StrategyContext) -> Signal {
                if ctx.index == 0 {
                    Signal::scale_in(0.5, ctx.timestamp(), ctx.close())
                } else {
                    Signal::hold()
                }
            }
        }

        let prices = [100.0, 100.0, 100.0];
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result = engine.run("TEST", &candles, ScaleInNoPos).unwrap();

        assert!(result.trades.is_empty());
        assert!((result.final_equity - config.initial_capital).abs() < 1e-6);
    }

    #[test]
    fn test_scale_out_noop_without_position() {
        /// Strategy: scale_out on bar 0 (no position open) — should be ignored.
        #[derive(Clone)]
        struct ScaleOutNoPos;
        impl Strategy for ScaleOutNoPos {
            fn name(&self) -> &str {
                "ScaleOutNoPos"
            }
            fn required_indicators(&self) -> Vec<(String, Indicator)> {
                vec![]
            }
            fn on_candle(&self, ctx: &StrategyContext) -> Signal {
                if ctx.index == 0 {
                    Signal::scale_out(0.5, ctx.timestamp(), ctx.close())
                } else {
                    Signal::hold()
                }
            }
        }

        let prices = [100.0, 100.0, 100.0];
        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result = engine.run("TEST", &candles, ScaleOutNoPos).unwrap();

        assert!(result.trades.is_empty());
        assert!((result.final_equity - config.initial_capital).abs() < 1e-6);
    }

    #[test]
    fn test_scale_in_pnl_uses_weighted_avg_cost_basis() {
        // Tests for issue where entry_quantity was not updated after scale_in,
        // causing close_with_tax to use the original (too-small) entry_quantity and
        // overstate gross PnL.
        //
        // Setup:
        //   bar 0 – long signal, fill bar 1 @ $100, buy 10 shares (position_size_pct=0.1)
        //   bar 1 – scale_in(0.5) signal, fill bar 2 @ $100, buy ~50% equity more
        //   bar 2 – exit signal, fill bar 3 @ $110
        //   No commission/slippage so PnL is pure price × qty arithmetic.
        let prices = [100.0, 100.0, 100.0, 110.0, 110.0];
        let candles = make_candles(&prices);

        let config = BacktestConfig::builder()
            .initial_capital(1_000.0)
            .position_size_pct(0.1) // buy 10% of cash = $100 / $100 = 1 share initially
            .commission_pct(0.0)
            .commission(0.0)
            .slippage_pct(0.0)
            .close_at_end(true)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result = engine.run("TEST", &candles, EnterScaleInExit).unwrap();

        // Confirm the scale-in fired.
        let si_executed = result
            .signals
            .iter()
            .any(|s| matches!(s.direction, SignalDirection::ScaleIn) && s.executed);
        assert!(
            si_executed,
            "scale-in did not execute — test is inconclusive"
        );

        // With no commission/slippage:
        //   trade.pnl  == (exit_price - entry_price) × qty_closed   (per-share basis)
        //              == ($110 − $100) × qty_closed
        // And final_equity == initial_capital + sum(pnl)
        let sum_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
        assert!(sum_pnl > 0.0, "expected a profit, got {sum_pnl:.6}");
        assert!(
            (result.final_equity - (config.initial_capital + sum_pnl)).abs() < 1e-6,
            "accounting invariant: final_equity={:.6}, expected={:.6}",
            result.final_equity,
            config.initial_capital + sum_pnl
        );
    }

    #[test]
    fn test_accounting_invariant_holds_with_scaling() {
        // Verifies: final_equity == initial_capital + sum(trade.pnl) after a
        // scale-in followed by a full exit.  Uses position_size_pct=0.2 so that
        // 80% of cash remains after the initial entry, giving the scale-in
        // (fraction=0.5 of equity) enough room to execute.
        let prices = [100.0, 100.0, 100.0, 110.0, 110.0, 120.0];
        let candles = make_candles(&prices);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .position_size_pct(0.2) // 20% per entry → 80% cash left for scale-in
            .commission_pct(0.001)
            .slippage_pct(0.0)
            .close_at_end(true)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result = engine.run("TEST", &candles, EnterScaleInExit).unwrap();

        // Confirm the scale-in actually fired (scale_in signal recorded as executed).
        let scale_in_executed = result
            .signals
            .iter()
            .any(|s| matches!(s.direction, SignalDirection::ScaleIn) && s.executed);
        assert!(
            scale_in_executed,
            "scale-in signal was not executed — test is inconclusive"
        );

        let sum_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
        let expected = config.initial_capital + sum_pnl;
        assert!(
            (result.final_equity - expected).abs() < 1e-4,
            "accounting invariant failed: final_equity={:.6}, expected={:.6}",
            result.final_equity,
            expected
        );
    }

    fn entry_quantity(allow_short: bool, max_leverage: f64) -> f64 {
        let candles = make_candles(&[100.0; 10]);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(allow_short)
            .max_leverage(max_leverage)
            .close_at_end(false)
            .build()
            .unwrap();
        let engine = BacktestEngine::new(config);
        let result = if allow_short {
            engine.run("TEST", &candles, EnterShortHold).unwrap()
        } else {
            engine.run("TEST", &candles, EnterLongHold).unwrap()
        };
        result.open_position.unwrap().quantity
    }

    #[test]
    fn test_short_entry_unchanged_at_default_leverage() {
        assert!((entry_quantity(true, 1.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_leverage_scales_entry_notional() {
        assert!((entry_quantity(false, 2.0) - 2.0 * entry_quantity(false, 1.0)).abs() < 1e-9);
        assert!((entry_quantity(true, 2.0) - 2.0 * entry_quantity(true, 1.0)).abs() < 1e-9);
    }
}
