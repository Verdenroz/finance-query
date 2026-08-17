use crate::backtesting::position::Position;
use crate::backtesting::signal::Signal;
use crate::backtesting::strategy::PositionExtremes;
use crate::models::chart::Candle;

use super::BacktestEngine;

// The `#[inline]` markers below are load-bearing: `simulate`'s per-candle loop
// lives in a sibling module, and `[profile.bench]` builds without LTO, so
// nothing crosses a codegen-unit boundary unless the MIR travels with it.

impl BacktestEngine {
    /// Check if stop-loss, take-profit, or trailing stop should trigger intrabar.
    ///
    /// Uses `candle.low` / `candle.high` to detect breaches that occur during the
    /// bar, not just at the close.  Returns an exit [`Signal`] whose `price` field
    /// is the computed fill price (stop/TP level with a gap-guard: if the bar opens
    /// through the level the open price is used instead so the fill is never better
    /// than the market).
    ///
    /// `hwm` is the intrabar high-water mark for longs (`candle.high` is
    /// incorporated each bar) or the low-water mark for shorts.
    ///
    /// # Exit Priority
    ///
    /// When multiple exit conditions are satisfied on the same bar, the first
    /// one checked wins: **stop-loss → take-profit → trailing stop**.
    ///
    /// In reality, the intrabar order of events is unknowable from OHLCV data
    /// alone — a bar could open through the take-profit level before touching
    /// the stop-loss, or vice versa.  The fixed priority errs on the side of
    /// pessimism (stop-loss before take-profit) for conservative simulation.
    /// Strategies with both SL and TP set should be aware of this ordering
    /// when both levels are close together relative to typical bar ranges.
    #[inline]
    pub(super) fn check_sl_tp(
        &self,
        position: &Position,
        candle: &Candle,
        hwm: Option<f64>,
    ) -> Option<Signal> {
        // Per-trade bracket overrides take precedence over config-level defaults.
        let sl_pct = position.bracket_stop_loss_pct.or(self.config.stop_loss_pct);
        let tp_pct = position
            .bracket_take_profit_pct
            .or(self.config.take_profit_pct);
        let trail_pct = position
            .bracket_trailing_stop_pct
            .or(self.config.trailing_stop_pct);

        // Stop-loss — intrabar breach via low (long) or high (short)
        if let Some(sl_pct) = sl_pct {
            let stop_price = if position.is_long() {
                position.entry_price * (1.0 - sl_pct)
            } else {
                position.entry_price * (1.0 + sl_pct)
            };
            let triggered = if position.is_long() {
                candle.low <= stop_price
            } else {
                candle.high >= stop_price
            };
            if triggered {
                // if the bar already opened through the stop level, fill
                // at the open (slippage/gap) rather than the stop price.
                let fill_price = if position.is_long() {
                    candle.open.min(stop_price)
                } else {
                    candle.open.max(stop_price)
                };
                let return_pct = position.unrealized_return_pct(fill_price);
                return Some(
                    Signal::exit(candle.timestamp, fill_price)
                        .with_reason(format!("Stop-loss triggered ({:.1}%)", return_pct)),
                );
            }
        }

        // Take-profit — intrabar breach via high (long) or low (short)
        if let Some(tp_pct) = tp_pct {
            let tp_price = if position.is_long() {
                position.entry_price * (1.0 + tp_pct)
            } else {
                position.entry_price * (1.0 - tp_pct)
            };
            let triggered = if position.is_long() {
                candle.high >= tp_price
            } else {
                candle.low <= tp_price
            };
            if triggered {
                // Gap guard: a gap-up open past TP gives a better fill at the open.
                let fill_price = if position.is_long() {
                    candle.open.max(tp_price)
                } else {
                    candle.open.min(tp_price)
                };
                let return_pct = position.unrealized_return_pct(fill_price);
                return Some(
                    Signal::exit(candle.timestamp, fill_price)
                        .with_reason(format!("Take-profit triggered ({:.1}%)", return_pct)),
                );
            }
        }

        // Trailing stop — checked after SL/TP so explicit levels take priority.
        //    `hwm` is already updated to the intrabar extreme before this call.
        if let Some(trail_pct) = trail_pct
            && let Some(extreme) = hwm
            && extreme > 0.0
        {
            let trail_stop_price = if position.is_long() {
                extreme * (1.0 - trail_pct)
            } else {
                extreme * (1.0 + trail_pct)
            };
            let triggered = if position.is_long() {
                candle.low <= trail_stop_price
            } else {
                candle.high >= trail_stop_price
            };
            if triggered {
                let fill_price = if position.is_long() {
                    candle.open.min(trail_stop_price)
                } else {
                    candle.open.max(trail_stop_price)
                };
                let adverse_move_pct = if position.is_long() {
                    (extreme - fill_price) / extreme
                } else {
                    (fill_price - extreme) / extreme
                };
                return Some(
                    Signal::exit(candle.timestamp, fill_price).with_reason(format!(
                        "Trailing stop triggered ({:.1}% adverse move)",
                        adverse_move_pct * 100.0
                    )),
                );
            }
        }

        None
    }
}

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Update the trailing-stop high-water mark (peak for longs, trough for shorts).
///
/// Uses the candle's intrabar extreme (`high` for longs, `low` for shorts) so
/// that the trailing stop correctly reflects the best price reached during the bar,
/// not just the close.
///
/// Cleared to `None` when no position is open so it resets on next entry.
/// Also used by the portfolio engine.
/// Fold `candle` into the running extremes for the open position.
///
/// Cleared to `None` when no position is open so it resets on the next entry —
/// the same lifecycle as [`update_trailing_hwm`], which tracks the single value
/// the intrabar stop needs while this tracks the four the trailing conditions do.
#[inline]
pub(crate) fn update_position_extremes(
    position: Option<&Position>,
    extremes: &mut Option<PositionExtremes>,
    candle: &Candle,
) {
    if position.is_none() {
        *extremes = None;
        return;
    }
    match extremes {
        Some(e) => e.update(candle),
        None => *extremes = Some(PositionExtremes::new(candle)),
    }
}

#[inline]
pub(crate) fn update_trailing_hwm(
    position: Option<&Position>,
    hwm: &mut Option<f64>,
    candle: &Candle,
) {
    if let Some(pos) = position {
        *hwm = Some(match *hwm {
            None => {
                if pos.is_long() {
                    candle.high
                } else {
                    candle.low
                }
            }
            Some(prev) => {
                if pos.is_long() {
                    prev.max(candle.high)
                } else {
                    prev.min(candle.low) // trough for shorts
                }
            }
        });
    } else {
        *hwm = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::fixtures::*;

    // ── Intrabar stop / take-profit tests ────────────────────────────────────

    #[test]
    fn test_intrabar_stop_loss_fills_at_stop_price_not_next_open() {
        // Bar 0: open=100, high=101, low=99, close=100 — entry signal fires, filled on bar 1.
        // Bar 1: open=100, high=100, low=100, close=100 — entry fills at 100.
        // Bar 2: open=99, high=99, low=90, close=94 — low(90) < stop(95); fill at min(open=99, stop=95) = 95.
        // With close-only detection, stop would not trigger here (close=94 > stop=95*... wait)
        // Actually close=94 < 95 so close-only WOULD trigger, but on the NEXT bar's open (bar 3).
        // With intrabar detection, it triggers on bar 2 itself and fills at stop_price=95.
        let candles = vec![
            make_candle_ohlc(0, 100.0, 101.0, 99.0, 100.0), // bar 0: entry signal
            make_candle_ohlc(1, 100.0, 102.0, 99.0, 100.0), // bar 1: entry fill at 100
            make_candle_ohlc(2, 99.0, 99.0, 90.0, 94.0),    // bar 2: low=90 < stop=95 → fill at 95
            make_candle_ohlc(3, 94.0, 95.0, 93.0, 94.0), // bar 3: would be next-bar fill in old code
        ];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .stop_loss_pct(0.05) // 5% → stop at 100 * 0.95 = 95
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine.run("TEST", &candles, EnterLongBar0).unwrap();

        let sl_trade = result.trades.iter().find(|t| {
            t.exit_signal
                .reason
                .as_ref()
                .map(|r| r.contains("Stop-loss"))
                .unwrap_or(false)
        });
        assert!(sl_trade.is_some(), "expected a stop-loss trade");
        let trade = sl_trade.unwrap();

        // Fill must be at the stop price (95.0), not at bar 3's open (94.0).
        assert!(
            (trade.exit_price - 95.0).abs() < 1e-9,
            "expected exit at stop price 95.0, got {:.6}",
            trade.exit_price
        );
        // Exit must be recorded on bar 2's timestamp, not bar 3.
        assert_eq!(
            trade.exit_timestamp, 2,
            "exit should be on bar 2 (intrabar)"
        );
    }

    #[test]
    fn test_intrabar_stop_loss_gap_down_fills_at_open() {
        // Bar 1: entry at open=100.
        // Bar 2: open=92 (already below stop=95) → gap guard → fill at open=92.
        let candles = vec![
            make_candle_ohlc(0, 100.0, 101.0, 99.0, 100.0), // bar 0: entry signal
            make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0), // bar 1: entry fill at 100
            make_candle_ohlc(2, 92.0, 92.0, 90.0, 90.0),    // bar 2: gap below stop → fill at 92
        ];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .stop_loss_pct(0.05) // stop at 95
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine.run("TEST", &candles, EnterLongBar0).unwrap();

        let sl_trade = result
            .trades
            .iter()
            .find(|t| {
                t.exit_signal
                    .reason
                    .as_ref()
                    .map(|r| r.contains("Stop-loss"))
                    .unwrap_or(false)
            })
            .expect("expected a stop-loss trade");

        // Gap-down: open (92) < stop (95) → fill at open.
        assert!(
            (sl_trade.exit_price - 92.0).abs() < 1e-9,
            "expected gap-down fill at 92.0, got {:.6}",
            sl_trade.exit_price
        );
    }

    #[test]
    fn test_intrabar_take_profit_fills_at_tp_price() {
        // Bar 1: entry at 100.
        // Bar 2: high=112 > tp=110 → fill at 110 (not next bar's open).
        let candles = vec![
            make_candle_ohlc(0, 100.0, 101.0, 99.0, 100.0),
            make_candle_ohlc(1, 100.0, 100.0, 100.0, 100.0), // entry fill
            make_candle_ohlc(2, 105.0, 112.0, 104.0, 111.0), // high > tp → fill at 110
            make_candle_ohlc(3, 112.0, 113.0, 111.0, 112.0), // would be next-bar fill in old code
        ];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .take_profit_pct(0.10) // TP at 100 * 1.10 = 110
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine.run("TEST", &candles, EnterLongBar0).unwrap();

        let tp_trade = result
            .trades
            .iter()
            .find(|t| {
                t.exit_signal
                    .reason
                    .as_ref()
                    .map(|r| r.contains("Take-profit"))
                    .unwrap_or(false)
            })
            .expect("expected a take-profit trade");

        assert!(
            (tp_trade.exit_price - 110.0).abs() < 1e-9,
            "expected TP fill at 110.0, got {:.6}",
            tp_trade.exit_price
        );
        assert_eq!(
            tp_trade.exit_timestamp, 2,
            "exit should be on bar 2 (intrabar)"
        );
    }

    // ── Per-trade bracket orders (Phase 11) ──────────────────────────────────

    // Each bracket type is tested for both Long and Short sides.
    // Long:  SL fires on low breach; TP fires on high breach; trail tracks HWM (peak).
    // Short: SL fires on high breach; TP fires on low breach; trail tracks LWM (trough).

    // ── Long stop-loss ────────────────────────────────────────────────────────

    #[test]
    fn test_per_trade_stop_loss_triggers_when_set() {
        // Bar 0: signal @ 100. Bar 1: fill @ open=100.
        // Bar 2: intrabar low=79.2. 5% stop = $95. low(79.2) <= 95 → stop fires.
        // Gap-down guard: fill = min(open=80, stop=95) = 80.
        let prices = [100.0, 100.0, 80.0, 80.0];
        let mut candles = make_candles(&prices);
        candles[2].low = 79.2;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketLongStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "stop-loss should have closed the position"
        );
        assert!(
            result.trades[0].pnl < 0.0,
            "stop-loss trade should be a loss"
        );
    }

    #[test]
    fn test_per_trade_stop_loss_overrides_config_none() {
        // Config has no stop-loss; per-trade bracket stop of 5% should still fire.
        let prices = [100.0, 100.0, 80.0, 80.0];
        let mut candles = make_candles(&prices);
        candles[2].low = 79.2;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap();

        assert!(
            config.stop_loss_pct.is_none(),
            "config must not have a default stop-loss for this test"
        );

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketLongStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "per-trade bracket stop should fire even when config stop_loss_pct is None"
        );
    }

    #[test]
    fn test_per_trade_stop_loss_overrides_config_looser() {
        // Config has a loose 20% stop ($80); per-trade bracket stop of 5% ($95) fires first.
        // Bar 2 opens at $97 (above $95) and dips to $93 intrabar — no gap-down — so the
        // fill resolves to min(open=97, stop=95) = $95, proving it's the tighter bracket
        // that fired and not the config's $80 level.
        //
        //   5% stop  = $95 → triggers (low=93 ≤ 95), fill = min(97, 95) = 95
        //   20% stop = $80 → would NOT trigger (low=93 > 80)
        let prices = [100.0, 100.0, 97.0, 97.0];
        let mut candles = make_candles(&prices);
        candles[2].low = 93.0; // below 5% stop=95, above 20% stop=80

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .stop_loss_pct(0.20) // loose config default
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketLongStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(!result.trades.is_empty());
        let trade = &result.trades[0];
        // Exit at the 5% bracket level ($95), not the 20% config level ($80).
        assert!(
            trade.exit_price > 90.0,
            "expected exit near 5% bracket stop ($95), got {:.2}",
            trade.exit_price
        );
    }

    // ── Short stop-loss ───────────────────────────────────────────────────────

    #[test]
    fn test_per_trade_short_stop_loss_triggers_when_set() {
        // Bar 0: signal short @ 100. Bar 1: fill @ open=100.
        // Bar 2: intrabar high=112.5. 5% stop = $105. high(112.5) >= 105 → stop fires.
        // Gap-up guard: fill = max(open=112, stop=105) = 112.
        let prices = [100.0, 100.0, 112.0, 112.0];
        let mut candles = make_candles(&prices);
        candles[2].high = 112.5;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketShortStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "short stop-loss should have closed the position"
        );
        assert!(
            result.trades[0].pnl < 0.0,
            "short stop-loss trade should be a loss (price rose against the short)"
        );
    }

    #[test]
    fn test_per_trade_short_stop_loss_overrides_config_none() {
        // Config has no stop-loss; per-trade bracket stop of 5% should still fire for shorts.
        let prices = [100.0, 100.0, 112.0, 112.0];
        let mut candles = make_candles(&prices);
        candles[2].high = 112.5;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .close_at_end(false)
            .build()
            .unwrap();

        assert!(config.stop_loss_pct.is_none());

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketShortStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "per-trade bracket stop should fire for shorts even with no config stop-loss"
        );
    }

    #[test]
    fn test_per_trade_short_stop_loss_overrides_config_looser() {
        // Config has a loose 20% stop ($120); per-trade bracket stop of 5% ($105) fires first.
        // Bar 2 opens at $103 (below $105) and rises to $108 intrabar — no gap-up — so
        // the fill resolves to max(open=103, stop=105) = $105, not the config's $120.
        //
        //   5% stop  = $105 → triggers (high=108 ≥ 105), fill = max(103, 105) = 105
        //   20% stop = $120 → would NOT trigger (high=108 < 120)
        let prices = [100.0, 100.0, 103.0, 103.0];
        let mut candles = make_candles(&prices);
        candles[2].high = 108.0; // above 5% stop=105, below 20% stop=120

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .stop_loss_pct(0.20) // loose config default
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketShortStopLossStrategy { stop_pct: 0.05 },
            )
            .unwrap();

        assert!(!result.trades.is_empty());
        let trade = &result.trades[0];
        // Exit at the 5% bracket level ($105), not the 20% config level ($120).
        assert!(
            trade.exit_price < 115.0,
            "expected exit near 5% bracket stop ($105), got {:.2}",
            trade.exit_price
        );
    }

    // ── Take-profit ───────────────────────────────────────────────────────────

    #[test]
    fn test_per_trade_take_profit_triggers() {
        // Bar 0: signal @ 100. Bar 1: fill @ open=100.
        // Bar 2: intrabar high=121.2. TP at 10% = $110. high(121.2) >= 110 → fires.
        // Gap-up guard: fill = max(open=120, tp=110) = 120.
        let prices = [100.0, 100.0, 120.0, 120.0];
        let mut candles = make_candles(&prices);
        candles[2].high = 121.2;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketLongTakeProfitStrategy { tp_pct: 0.10 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "long take-profit should have fired"
        );
        assert!(
            result.trades[0].pnl > 0.0,
            "long take-profit trade should be profitable"
        );
    }

    #[test]
    fn test_per_trade_short_take_profit_triggers() {
        // Bar 0: signal short @ 100. Bar 1: fill @ open=100.
        // Bar 2: intrabar low=84.15. TP at 10% = $90. low(84.15) <= 90 → fires.
        // Gap-down guard: fill = min(open=85, tp=90) = 85.
        let prices = [100.0, 100.0, 85.0, 85.0];
        let mut candles = make_candles(&prices);
        candles[2].low = 84.15;

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketShortTakeProfitStrategy { tp_pct: 0.10 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "short take-profit should have fired"
        );
        assert!(
            result.trades[0].pnl > 0.0,
            "short take-profit trade should be profitable (price fell in favor of short)"
        );
    }

    // ── Trailing stop ─────────────────────────────────────────────────────────

    #[test]
    fn test_per_trade_trailing_stop_triggers() {
        // Bar 0: signal @ 100.
        // Bar 1: fill @ open=100. HWM initialised to entry_price=100.
        // Bar 2: high=121.0 → HWM = max(100, 121) = 121. Trail stop = 121*(1-0.05) = 114.95.
        //        low=118.8 → 118.8 > 114.95 → no trigger.
        // Bar 3: low=108.9 → 108.9 <= 114.95 → trailing stop fires.
        //        fill = min(open=110, trail=114.95) = 110. pnl > 0.
        let prices = [100.0, 100.0, 120.0, 110.0, 110.0];
        let mut candles = make_candles(&prices);
        candles[2].high = 121.0;
        candles[3].low = 108.9; // below 5% trail from 121.0 (= 114.95)

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketLongTrailingStopStrategy { trail_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "long trailing stop should have fired"
        );
        assert!(
            result.trades[0].pnl > 0.0,
            "long trailing stop should exit in profit (entry $100, exit near $110)"
        );
    }

    #[test]
    fn test_per_trade_short_trailing_stop_triggers() {
        // Bar 0: signal short @ 100.
        // Bar 1: fill @ open=100. LWM (trough) initialised to entry_price=100.
        // Bar 2: price=80, low=79.2 → LWM = min(100, 79.2) = 79.2.
        //        Trail stop = 79.2*(1+0.05) = 83.16. high=80.8 → 80.8 < 83.16 → no trigger.
        // Bar 3: price=88, high=88.88 → 88.88 >= 83.16 → trailing stop fires.
        //        fill = max(open=88, trail=83.16) = 88. pnl > 0 (short from 100, exit at 88).
        let prices = [100.0, 100.0, 80.0, 88.0, 88.0];
        let mut candles = make_candles(&prices);
        candles[2].low = 79.2; // drives LWM to 79.2; trail stop = 79.2 * 1.05 = 83.16

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .allow_short(true)
            .close_at_end(false)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run(
                "TEST",
                &candles,
                BracketShortTrailingStopStrategy { trail_pct: 0.05 },
            )
            .unwrap();

        assert!(
            !result.trades.is_empty(),
            "short trailing stop should have fired"
        );
        assert!(
            result.trades[0].pnl > 0.0,
            "short trailing stop should exit in profit (entry $100, exit near $88)"
        );
    }
}
