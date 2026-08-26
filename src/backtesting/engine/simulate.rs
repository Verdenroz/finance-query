use crate::backtesting::error::{BacktestError, Result};
use crate::backtesting::position::{Position, Trade};
use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics, SignalRecord};
use crate::backtesting::signal::{OrderType, PendingOrder, Signal, SignalDirection};
use crate::backtesting::strategy::{PositionExtremes, Strategy, StrategyContext};
use crate::models::chart::{Candle, Dividend};

use super::BacktestEngine;
use super::exits::{update_position_extremes, update_trailing_hwm};

impl BacktestEngine {
    // ── Core simulation ───────────────────────────────────────────────────────

    /// Internal simulation core. All public `run*` methods delegate here.
    ///
    /// Assumes `candles` and `dividends` are already known to be ascending —
    /// the public entry points check that via [`validate_series_order`]. Sweeps
    /// run thousands of simulations over one series, so the O(n) scan is hoisted
    /// to the boundary rather than repeated per candidate.
    ///
    /// [`validate_series_order`]: super::validate_series_order
    pub(crate) fn simulate<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        mut strategy: S,
        dividends: &[Dividend],
    ) -> Result<BacktestResult> {
        let warmup = strategy.warmup_period().max(self.config.sizing_warmup());
        if candles.len() < warmup {
            return Err(BacktestError::insufficient_data(warmup, candles.len()));
        }

        // Pre-compute all required indicators (base timeframe + HTF stretched arrays)
        let mut indicators = self.compute_indicators(candles, &strategy)?;
        indicators.extend(self.compute_htf_indicators(candles, &strategy)?);

        // Let the strategy cache direct pointers into the indicator map, eliminating
        // per-bar HashMap lookups in on_candle.
        strategy.setup(&indicators);

        let sizing_series = self.compute_sizing_series(candles);

        // Initialize state
        let mut equity = self.config.initial_capital;
        let mut cash = self.config.initial_capital;
        let mut position: Option<Position> = None;
        let mut trades: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<EquityPoint> = Vec::with_capacity(candles.len());
        let mut signals: Vec<SignalRecord> = Vec::new();
        let mut peak_equity = equity;
        let mut max_leverage_used = 0.0_f64;
        // Both hooks are no-ops on a default config, and the bar loop is hot
        // enough that the cross-module call to find that out is worth skipping.
        let financing_enabled =
            self.config.short_borrow_rate > 0.0 || self.config.margin_interest_rate > 0.0;
        let margin_enabled = self.config.max_leverage > 1.0 || self.config.allow_short;
        // High-water mark for the trailing stop: tracks peak price (longs) or
        // trough price (shorts) since entry. Reset to None when no position is open.
        let mut hwm: Option<f64> = None;
        // The same running scan widened to the four values trailing conditions
        // need. Owned by the position rather than by any one condition, so every
        // trailing condition on a position reads one shared value.
        let mut extremes: Option<PositionExtremes> = None;
        // Only strategies with a trailing condition read this, and folding it
        // per bar is pure overhead for every strategy that doesn't.
        let track_extremes = strategy.tracks_position_extremes();

        // Dividend processing pointer: dividends must be sorted by timestamp.
        // We advance this index forward as the simulation progresses in time.
        let mut div_idx: usize = 0;

        // Pending limit / stop orders placed by the strategy.
        // Checked each bar before strategy signal evaluation.
        let mut pending_orders: Vec<PendingOrder> = Vec::new();

        // Main simulation loop
        for i in 0..candles.len() {
            let candle = &candles[i];

            if financing_enabled {
                self.accrue_financing(&mut position, &mut cash, candle);
            }

            equity = Self::update_equity_and_curve(
                position.as_ref(),
                candle,
                cash,
                &mut peak_equity,
                &mut equity_curve,
            );

            update_trailing_hwm(position.as_ref(), &mut hwm, candle);
            if track_extremes {
                // Pending orders fill after this has run for their bar, so the
                // bar a limit/stop entry opened on would never be folded in.
                // Rebuilding from the entry bar on the first update after an
                // entry keeps this in step with the scan in `threshold.rs`.
                if extremes.is_none()
                    && let Some(pos) = position.as_ref()
                {
                    let entry =
                        candles[..=i].partition_point(|c| c.timestamp < pos.entry_timestamp);
                    extremes = PositionExtremes::from_candles(&candles[entry..=i]);
                }
                update_position_extremes(position.as_ref(), &mut extremes, candle);
            }

            // Credit dividend income for any dividends ex-dated on or before this bar.
            self.credit_dividends(&mut position, candle, dividends, &mut div_idx);

            if let Some(pos) = position.as_ref() {
                // Recomputed after the dividend credit so the peak matches the
                // ratio the margin call measures.
                let margin_equity =
                    cash + pos.current_value(candle.close) + pos.unreinvested_dividends;
                let exposure = pos.quantity * candle.close;
                if margin_equity > 0.0 && exposure > max_leverage_used * margin_equity {
                    max_leverage_used = exposure / margin_equity;
                }
            }

            // Check stop-loss / take-profit / trailing-stop on existing position.
            // The signal carries the intrabar fill price (stop/TP level with gap guard),
            // so we execute on the current bar at that price — no next-bar deferral needed.
            if let Some(ref pos) = position
                && let Some(exit_signal) = self.check_sl_tp(pos, candle, hwm)
            {
                let fill_price = exit_signal.price;
                let executed = self.close_position_at(
                    &mut position,
                    &mut cash,
                    &mut trades,
                    candle,
                    fill_price,
                    &exit_signal,
                );

                signals.push(SignalRecord {
                    timestamp: candle.timestamp,
                    price: fill_price,
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                    reason: exit_signal.reason.clone(),
                    executed,
                    tags: exit_signal.tags.clone(),
                });

                if executed {
                    hwm = None; // Reset HWM when position is closed
                    extremes = None;
                    continue; // Skip strategy signal this bar
                }
            }

            // After the stops: a stop fills intrabar and the maintenance check
            // reads the close, so a bar that trips both filled the stop first.
            if margin_enabled
                && let Some(margin_signal) = self.check_margin_call(position.as_ref(), cash, candle)
            {
                let fill_price = margin_signal.price;
                let executed = self.close_position_at(
                    &mut position,
                    &mut cash,
                    &mut trades,
                    candle,
                    fill_price,
                    &margin_signal,
                );

                signals.push(SignalRecord {
                    timestamp: candle.timestamp,
                    price: fill_price,
                    direction: SignalDirection::Exit,
                    strength: 1.0,
                    reason: margin_signal.reason.clone(),
                    executed,
                    tags: margin_signal.tags.clone(),
                });

                if executed {
                    hwm = None;
                    extremes = None;
                    continue;
                }
            }

            // ── Pending limit / stop orders ───────────────────────────────
            // Check queued orders against the current bar before evaluating
            // the strategy. This preserves the realistic ordering where a
            // pending order placed on bar N can first fill on bar N+1.
            //
            // `retain_mut` preserves FIFO queue order (critical for correct
            // order matching) while avoiding the temporary index vec and the
            // ordering-destroying `swap_remove` used previously.
            let mut filled_this_bar = false;
            pending_orders.retain_mut(|order| {
                // Expire orders past their GTC lifetime.
                if let Some(exp) = order.expires_in_bars
                    && i >= order.created_bar + exp
                {
                    return false; // drop
                }

                // Cannot fill into an existing position, or if another
                // pending order already filled on this bar.
                if position.is_some() || filled_this_bar {
                    return true; // keep
                }

                // Short orders require allow_short.
                if matches!(order.signal.direction, SignalDirection::Short)
                    && !self.config.allow_short
                {
                    return true; // keep (config could change via re-run)
                }

                // BuyStopLimit state machine: if the stop price is triggered
                // but the bar opens above the limit price the order can't fill
                // this bar. In reality the stop has already "activated" the
                // order, which now rests in the book as a plain limit order.
                // Downgrade so subsequent bars treat it as a BuyLimit.
                let upgrade_to_limit = match &order.order_type {
                    OrderType::BuyStopLimit {
                        stop_price,
                        limit_price,
                    } if candle.high >= *stop_price => {
                        let trigger_fill = candle.open.max(*stop_price);
                        if trigger_fill > *limit_price {
                            Some(*limit_price) // triggered, limit not reached
                        } else {
                            None // triggered and fillable — handled below
                        }
                    }
                    _ => None,
                };
                if let Some(new_limit) = upgrade_to_limit {
                    order.order_type = OrderType::BuyLimit {
                        limit_price: new_limit,
                    };
                    return true; // keep as plain BuyLimit; skip fill this bar
                }

                if let Some(fill_price) = order.order_type.try_fill(candle) {
                    let sizing =
                        self.build_sizing_context(i.saturating_sub(1), &sizing_series, &trades);
                    let executed = self.open_position_at_price(
                        &mut position,
                        &mut cash,
                        candle,
                        &order.signal,
                        fill_price,
                        &sizing,
                    );
                    if executed {
                        hwm = position.as_ref().map(|p| p.entry_price);
                        signals.push(SignalRecord {
                            timestamp: candle.timestamp,
                            price: fill_price,
                            direction: order.signal.direction,
                            strength: order.signal.strength.value(),
                            reason: order.signal.reason.clone(),
                            executed: true,
                            tags: order.signal.tags.clone(),
                        });
                        filled_this_bar = true;
                        return false; // drop — order filled
                    }
                }

                true // keep unfilled order
            });

            // Skip strategy signals during warmup period
            if i < warmup.saturating_sub(1) {
                continue;
            }

            // Build strategy context
            let ctx = StrategyContext {
                candles: &candles[..=i],
                index: i,
                position: position.as_ref(),
                equity,
                indicators: &indicators,
                extremes: position.as_ref().and(extremes.as_ref()),
            };

            // Get strategy signal
            let signal = strategy.on_candle(&ctx);

            // Skip hold signals
            if signal.is_hold() {
                continue;
            }

            // Check signal strength threshold
            if signal.strength.value() < self.config.min_signal_strength {
                signals.push(SignalRecord {
                    timestamp: signal.timestamp,
                    price: signal.price,
                    direction: signal.direction,
                    strength: signal.strength.value(),
                    reason: signal.reason.clone(),
                    executed: false,
                    tags: signal.tags.clone(),
                });
                continue;
            }

            // Market orders execute on next bar to avoid same-bar close-fill
            // bias.  Limit and stop entry orders are queued as PendingOrders
            // and fill on a subsequent bar when the price level is reached.
            // Non-Market directions other than Long/Short (Exit, ScaleIn,
            // ScaleOut) are always treated as market orders.
            let executed = match &signal.order_type {
                OrderType::Market => {
                    if let Some(fill_candle) = candles.get(i + 1) {
                        let sizing = self.build_sizing_context(i, &sizing_series, &trades);
                        self.execute_signal(
                            &signal,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
                            &sizing,
                        )
                    } else {
                        false
                    }
                }
                _ if matches!(
                    signal.direction,
                    SignalDirection::Long | SignalDirection::Short
                ) =>
                {
                    // Reject short orders immediately if shorts are disabled —
                    // no point burning queue space for orders that can never fill.
                    if matches!(signal.direction, SignalDirection::Short)
                        && !self.config.allow_short
                    {
                        false
                    } else {
                        // Queue as a pending order; the signal record below will
                        // show executed: false (order placed but not yet filled).
                        pending_orders.push(PendingOrder {
                            order_type: signal.order_type.clone(),
                            expires_in_bars: signal.expires_in_bars,
                            created_bar: i,
                            signal: signal.clone(),
                        });
                        false
                    }
                }
                _ => {
                    // Non-market Exit / ScaleIn / ScaleOut — execute as market.
                    if let Some(fill_candle) = candles.get(i + 1) {
                        let sizing = self.build_sizing_context(i, &sizing_series, &trades);
                        self.execute_signal(
                            &signal,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
                            &sizing,
                        )
                    } else {
                        false
                    }
                }
            };

            if executed
                && position.is_some()
                && matches!(
                    signal.direction,
                    SignalDirection::Long | SignalDirection::Short
                )
            {
                hwm = position.as_ref().map(|p| p.entry_price);
                // Seeded on the next bar rather than from the entry price: the
                // extremes are bar highs/lows since entry, and the entry bar is
                // folded in at the top of the loop.
                extremes = None;
            }

            // Reset the trailing-stop HWM whenever a position is closed
            if executed && position.is_none() {
                hwm = None;
                extremes = None;

                // Re-evaluate strategy on the same bar after an exit so that
                // a crossover that simultaneously closes one side and triggers
                // the opposite entry is not lost.
                let ctx2 = StrategyContext {
                    candles: &candles[..=i],
                    index: i,
                    position: None,
                    equity,
                    indicators: &indicators,
                    extremes: None,
                };
                let follow = strategy.on_candle(&ctx2);
                if !follow.is_hold() && follow.strength.value() >= self.config.min_signal_strength {
                    let follow_executed = if let Some(fill_candle) = candles.get(i + 1) {
                        let sizing = self.build_sizing_context(i, &sizing_series, &trades);
                        self.execute_signal(
                            &follow,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
                            &sizing,
                        )
                    } else {
                        false
                    };
                    if follow_executed && position.is_some() {
                        hwm = position.as_ref().map(|p| p.entry_price);
                    }
                    signals.push(SignalRecord {
                        timestamp: follow.timestamp,
                        price: follow.price,
                        direction: follow.direction,
                        strength: follow.strength.value(),
                        reason: follow.reason,
                        executed: follow_executed,
                        tags: follow.tags,
                    });
                }
            }

            signals.push(SignalRecord {
                timestamp: signal.timestamp,
                price: signal.price,
                direction: signal.direction,
                strength: signal.strength.value(),
                reason: signal.reason,
                executed,
                tags: signal.tags,
            });
        }

        // Close any open position at end if configured
        if self.config.close_at_end
            && let Some(pos) = position.take()
        {
            let last_candle = candles
                .last()
                .expect("candles non-empty: position open implies loop ran");
            let exit_price_slipped = self
                .config
                .apply_exit_slippage(last_candle.close, pos.is_long());
            let exit_price = self
                .config
                .apply_exit_spread(exit_price_slipped, pos.is_long());
            let exit_commission = self.config.calculate_commission(pos.quantity, exit_price);
            // Tax on buy orders only: short covers are buys
            let exit_tax = self
                .config
                .calculate_transaction_tax(exit_price * pos.quantity, !pos.is_long());

            let exit_signal = Signal::exit(last_candle.timestamp, last_candle.close)
                .with_reason("End of backtest");

            let trade = pos.close_with_tax(
                last_candle.timestamp,
                exit_price,
                exit_commission,
                exit_tax,
                exit_signal,
            );
            if trade.is_long() {
                cash += trade.exit_value() - exit_commission + trade.unreinvested_dividends;
            } else {
                cash -=
                    trade.exit_value() + exit_commission + exit_tax - trade.unreinvested_dividends;
            }
            trades.push(trade);

            Self::sync_terminal_equity_point(&mut equity_curve, last_candle.timestamp, cash);
        }

        // Final equity
        let final_equity = if let Some(ref pos) = position {
            cash + pos.current_value(
                candles
                    .last()
                    .expect("candles non-empty: position open implies loop ran")
                    .close,
            ) + pos.unreinvested_dividends
        } else {
            cash
        };

        if let Some(last_candle) = candles.last() {
            Self::sync_terminal_equity_point(
                &mut equity_curve,
                last_candle.timestamp,
                final_equity,
            );
        }

        // Calculate metrics
        let executed_signals = signals.iter().filter(|s| s.executed).count();
        let mut metrics = PerformanceMetrics::calculate(
            &trades,
            &equity_curve,
            self.config.initial_capital,
            signals.len(),
            executed_signals,
            self.config.risk_free_rate,
            self.config.bars_per_year,
        );
        if let Some(ref pos) = position {
            metrics.total_financing_cost += pos.financing_cost_accrued;
        }

        let start_timestamp = candles.first().map(|c| c.timestamp).unwrap_or(0);
        let end_timestamp = candles.last().map(|c| c.timestamp).unwrap_or(0);

        // Build diagnostics for likely misconfigurations
        let mut diagnostics = Vec::new();
        if trades.is_empty() {
            if signals.is_empty() {
                diagnostics.push(
                    "No signals were generated. Check that the strategy's warmup \
                     period is shorter than the data length and that indicator \
                     conditions can be satisfied."
                        .into(),
                );
            } else {
                let short_signals = signals
                    .iter()
                    .filter(|s| matches!(s.direction, SignalDirection::Short))
                    .count();
                if short_signals > 0 && !self.config.allow_short {
                    diagnostics.push(format!(
                        "{short_signals} short signal(s) were generated but \
                         config.allow_short is false. Enable it with \
                         BacktestConfig::builder().allow_short(true)."
                    ));
                }
                diagnostics.push(format!(
                    "{} signal(s) generated but none executed. Check \
                     min_signal_strength ({}) and capital requirements.",
                    signals.len(),
                    self.config.min_signal_strength
                ));
            }
        }

        Ok(BacktestResult {
            symbol: symbol.to_string(),
            strategy_name: strategy.name().to_string(),
            config: self.config.clone(),
            start_timestamp,
            end_timestamp,
            initial_capital: self.config.initial_capital,
            final_equity,
            metrics,
            trades,
            equity_curve,
            signals,
            open_position: position,
            benchmark: None, // Populated by run_with_benchmark when a benchmark is supplied
            diagnostics,
            max_leverage_used,
        })
    }

    // ── Simulation helpers ────────────────────────────────────────────────────

    /// Compute current equity, track peak/drawdown, and append an equity curve point.
    ///
    /// Returns the updated equity value.
    fn update_equity_and_curve(
        position: Option<&Position>,
        candle: &Candle,
        cash: f64,
        peak_equity: &mut f64,
        equity_curve: &mut Vec<EquityPoint>,
    ) -> f64 {
        let equity = match position {
            Some(pos) => cash + pos.current_value(candle.close) + pos.unreinvested_dividends,
            None => cash,
        };
        if equity > *peak_equity {
            *peak_equity = equity;
        }
        let drawdown_pct = if *peak_equity > 0.0 {
            (*peak_equity - equity) / *peak_equity
        } else {
            0.0
        };
        equity_curve.push(EquityPoint {
            timestamp: candle.timestamp,
            equity,
            drawdown_pct,
        });
        equity
    }

    /// Credit any dividends whose ex-date falls on or before the current candle.
    ///
    /// Advances `div_idx` forward so each dividend is credited exactly once.
    fn credit_dividends(
        &self,
        position: &mut Option<Position>,
        candle: &Candle,
        dividends: &[Dividend],
        div_idx: &mut usize,
    ) {
        while *div_idx < dividends.len() && dividends[*div_idx].timestamp <= candle.timestamp {
            if let Some(pos) = position.as_mut() {
                let per_share = dividends[*div_idx].amount;
                let income = if pos.is_long() {
                    per_share * pos.quantity
                } else {
                    -(per_share * pos.quantity)
                };
                pos.credit_dividend(income, candle.close, self.config.reinvest_dividends);
            }
            *div_idx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::config::BacktestConfig;
    use crate::backtesting::engine::fixtures::*;
    use crate::backtesting::strategy::SmaCrossover;

    #[test]
    fn test_insufficient_data() {
        let candles = make_candles(&[100.0, 101.0, 102.0]); // Only 3 candles
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(10, 20); // Needs at least 21 candles

        let result = engine.run("TEST", &candles, strategy);
        assert!(result.is_err());
    }

    /// The fundamental invariant: final cash (when no position is open) must equal
    /// initial_capital plus the sum of all realized trade P&Ls.  This guards against
    /// the double-counting of commissions that existed before the fix.
    #[test]
    fn test_commission_accounting_invariant() {
        // Steadily rising prices so SmaCrossover(3,6) will definitely enter and exit.
        let prices: Vec<f64> = (0..40)
            .map(|i| {
                if i < 30 {
                    100.0 + i as f64
                } else {
                    129.0 - (i - 30) as f64 * 5.0
                }
            })
            .collect();
        let candles = make_candles(&prices);

        // Use both flat AND percentage commission to expose any double-counting.
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission(5.0) // $5 flat fee per trade
            .commission_pct(0.001) // + 0.1% per trade
            .slippage_pct(0.0)
            .close_at_end(true)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config.clone());
        let result = engine
            .run("TEST", &candles, SmaCrossover::new(3, 6))
            .unwrap();

        // When all positions are closed, cash == initial_capital + sum(trade pnls)
        let sum_pnl: f64 = result.trades.iter().map(|t| t.pnl).sum();
        let expected = config.initial_capital + sum_pnl;
        let actual = result.final_equity;
        assert!(
            (actual - expected).abs() < 1e-6,
            "Commission accounting: final_equity {actual:.6} != initial_capital + sum(pnl) {expected:.6}",
        );
    }

    #[test]
    fn test_unsorted_dividends_returns_error() {
        use crate::models::chart::Dividend;

        let prices: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);

        // Intentionally unsorted
        let dividends = vec![
            Dividend {
                timestamp: 20,
                amount: 1.0,
                provider_id: None,
            },
            Dividend {
                timestamp: 10,
                amount: 1.0,
                provider_id: None,
            },
        ];

        let engine = BacktestEngine::new(BacktestConfig::default());
        let result =
            engine.run_with_dividends("TEST", &candles, SmaCrossover::new(3, 6), &dividends);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("sorted"),
            "error should mention sorting: {msg}"
        );
    }

    #[test]
    fn test_short_dividend_is_liability() {
        use crate::models::chart::Dividend;

        let candles = make_candles(&[100.0, 100.0, 100.0]);
        let dividends = vec![Dividend {
            timestamp: candles[1].timestamp,
            amount: 1.0,
            provider_id: None,
        }];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .allow_short(true)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_dividends("TEST", &candles, EnterShortHold, &dividends)
            .unwrap();

        assert_eq!(result.trades.len(), 1);
        assert!(result.trades[0].dividend_income < 0.0);
        assert!(result.final_equity < 10_000.0);
    }

    #[test]
    fn test_open_position_final_equity_includes_accrued_dividends() {
        use crate::models::chart::Dividend;

        let candles = make_candles(&[100.0, 100.0, 100.0]);
        let dividends = vec![Dividend {
            timestamp: candles[1].timestamp,
            amount: 1.0,
            provider_id: None,
        }];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .close_at_end(false)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_dividends("TEST", &candles, EnterLongHold, &dividends)
            .unwrap();

        assert!(result.open_position.is_some());
        assert!((result.final_equity - 10_100.0).abs() < 1e-6);
        let last_equity = result.equity_curve.last().map(|p| p.equity).unwrap_or(0.0);
        assert!((last_equity - 10_100.0).abs() < 1e-6);
    }

    #[test]
    fn unsorted_candles_are_rejected() {
        let mut candles = make_candles(&[100.0, 101.0, 102.0, 103.0]);
        candles.swap(1, 2);
        use crate::backtesting::refs::*;
        use crate::backtesting::strategy::StrategyBuilder;
        let strategy = StrategyBuilder::new("s")
            .entry(price().above(0.0))
            .exit(price().below(0.0))
            .build();
        let err = BacktestEngine::new(BacktestConfig::default())
            .run("TEST", &candles, strategy)
            .unwrap_err();
        assert!(
            format!("{err}").contains("candles"),
            "expected a candle-ordering error, got {err}"
        );
    }

    /// Enters once via a limit order, then exits on a trailing stop.
    ///
    /// `track` selects which path supplies the peak: `true` uses the engine's
    /// running extremes, `false` makes the condition fall back to scanning from
    /// the entry bar. Both must agree.
    #[derive(Clone)]
    struct LimitEntryTrailing {
        track: bool,
        trail: crate::backtesting::condition::TrailingStop,
        limit: f64,
    }

    impl Strategy for LimitEntryTrailing {
        fn name(&self) -> &str {
            "limit-entry-trailing"
        }

        fn required_indicators(&self) -> Vec<(String, crate::indicators::Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            use crate::backtesting::condition::Condition;
            if ctx.position.is_none() {
                if ctx.index == 0 {
                    return Signal::buy_limit(ctx.timestamp(), ctx.close(), self.limit);
                }
                return Signal::hold();
            }
            if self.trail.evaluate(ctx) {
                return ctx.signal_exit();
            }
            Signal::hold()
        }

        fn tracks_position_extremes(&self) -> bool {
            self.track
        }
    }

    #[test]
    fn a_limit_entry_counts_its_own_fill_bar_in_the_peak() {
        // Pending orders fill partway through a bar, after the engine has
        // already folded that bar's extremes for a still-empty position. The
        // fill bar carries the highest high here, so skipping it lowers the
        // peak and moves the trailing-stop exit.
        let candles = vec![
            // bar 0: signal bar, queues the limit order
            Candle {
                timestamp: 0,
                open: 100.0,
                high: 100.0,
                low: 100.0,
                close: 100.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
            // bar 1: dips to 95 (fills), then spikes to 130 — the peak
            Candle {
                timestamp: 1,
                open: 99.0,
                high: 130.0,
                low: 94.0,
                close: 120.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
            Candle {
                timestamp: 2,
                open: 119.0,
                high: 121.0,
                low: 115.0,
                close: 116.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
            Candle {
                timestamp: 3,
                open: 115.0,
                high: 116.0,
                low: 110.0,
                close: 111.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
            Candle {
                timestamp: 4,
                open: 110.0,
                high: 111.0,
                low: 104.0,
                close: 105.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
            Candle {
                timestamp: 5,
                open: 104.0,
                high: 105.0,
                low: 100.0,
                close: 101.0,
                volume: 1000,
                adj_close: None,
                provider_id: None,
            },
        ];

        let config = BacktestConfig {
            initial_capital: 10_000.0,
            ..Default::default()
        };
        let run = |track: bool| {
            BacktestEngine::new(config.clone())
                .run(
                    "TEST",
                    &candles,
                    LimitEntryTrailing {
                        track,
                        trail: crate::backtesting::condition::TrailingStop::new(0.10),
                        limit: 95.0,
                    },
                )
                .unwrap()
        };

        let engine_path = run(true);
        let scan_path = run(false);

        assert_eq!(
            engine_path.trades.len(),
            1,
            "the limit order should fill and the trailing stop should close it"
        );
        assert_eq!(
            engine_path.trades.len(),
            scan_path.trades.len(),
            "the two peak sources disagreed on whether a trade closed"
        );
        assert_eq!(
            engine_path.trades[0].exit_timestamp, scan_path.trades[0].exit_timestamp,
            "engine-tracked extremes and the entry-bar scan chose different exits"
        );
        assert_eq!(
            engine_path.trades[0].pnl, scan_path.trades[0].pnl,
            "same exit bar should mean same P&L"
        );
    }

    #[test]
    fn sweeps_reject_unsorted_candles_at_their_own_entry_point() {
        // Validation moved off the per-candidate path, so each sweep entry point
        // has to check the series itself or an unsorted run would slip through.
        use crate::backtesting::optimizer::{BayesianSearch, GridSearch, ParamRange, ParamValue};
        use crate::backtesting::refs::*;
        use crate::backtesting::strategy::StrategyBuilder;
        use std::collections::HashMap;

        let mut candles = make_candles(&(0..80).map(|i| 100.0 + i as f64).collect::<Vec<f64>>());
        candles.swap(1, 2);
        let config = BacktestConfig::default();
        let factory = |_: &HashMap<String, ParamValue>| {
            StrategyBuilder::new("s")
                .entry(price().above(0.0))
                .exit(price().below(0.0))
                .build()
        };

        let grid_err = GridSearch::new()
            .param("p", ParamRange::int_range(1, 2, 1))
            .run("TEST", &candles, &config, factory)
            .unwrap_err();
        assert!(
            format!("{grid_err}").contains("candles"),
            "grid search should reject unsorted candles, got {grid_err}"
        );

        let bayes_err = BayesianSearch::new()
            .param("p", ParamRange::int_range(1, 2, 1))
            .max_evaluations(4)
            .run("TEST", &candles, &config, factory)
            .unwrap_err();
        assert!(
            format!("{bayes_err}").contains("candles"),
            "bayesian search should reject unsorted candles, got {bayes_err}"
        );
    }
}
