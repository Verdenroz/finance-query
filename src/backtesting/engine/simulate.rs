use crate::backtesting::config::SizingContext;
use crate::backtesting::error::{BacktestError, Result};
use crate::backtesting::position::{Position, Trade};
use crate::backtesting::result::{BacktestResult, EquityPoint, PerformanceMetrics, SignalRecord};
use crate::backtesting::signal::{OrderType, PendingOrder, Signal, SignalDirection};
use crate::backtesting::strategy::{PositionExtremes, Strategy, StrategyContext};
use crate::models::chart::{Candle, Dividend};

use super::BacktestEngine;
use super::exits::{check_sl_tp, update_position_extremes, update_trailing_hwm};
use super::sizing::SizingSeries;

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

            // Credited before the equity snapshot so the ex-date bar's curve
            // point and ctx.equity match the post-dividend value the margin
            // check below measures.
            self.credit_dividends(&mut position, candle, dividends, &mut div_idx);

            let peak_before_bar = peak_equity;
            equity = Self::update_equity_and_curve(
                position.as_ref(),
                candle,
                cash,
                &mut peak_equity,
                &mut equity_curve,
            );

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

            if let Some(pos) = position.as_ref() {
                let exposure = pos.quantity * candle.close;
                if equity > 0.0 && exposure > max_leverage_used * equity {
                    max_leverage_used = exposure / equity;
                }
            }

            // Check stop-loss / take-profit / trailing-stop on existing position.
            // The signal carries the intrabar fill price (stop/TP level with gap guard),
            // so we execute on the current bar at that price — no next-bar deferral needed.
            if let Some(ref pos) = position
                && let Some(exit_signal) = check_sl_tp(pos, candle, hwm, &self.config)
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
                    Self::resync_equity_point(
                        &mut equity_curve,
                        &mut peak_equity,
                        peak_before_bar,
                        cash,
                    );
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
                    Self::resync_equity_point(
                        &mut equity_curve,
                        &mut peak_equity,
                        peak_before_bar,
                        cash,
                    );
                    hwm = None;
                    extremes = None;
                    continue;
                }
            }

            // Runs after check_sl_tp so the trail level it just used excludes
            // this bar's own high/low.
            update_trailing_hwm(position.as_ref(), &mut hwm, candle);

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
                // Expire orders past their GTC lifetime. Strict `>`: exp=n grants
                // fill attempts on bars created_bar+1..=created_bar+n.
                if let Some(exp) = order.expires_in_bars
                    && i > order.created_bar + exp
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
                        // update_trailing_hwm already ran for this bar while the
                        // position was still None, so seed the fold here too.
                        hwm = position.as_ref().map(|p| {
                            if p.is_long() {
                                p.entry_price.max(candle.high)
                            } else {
                                p.entry_price.min(candle.low)
                            }
                        });
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

            // The bar-top snapshot ran before this fill, so a limit/stop entry
            // would otherwise go unmeasured until the next bar (never, on the
            // final bar). Equity is recomputed: the fill moved cash.
            if filled_this_bar && let Some(pos) = position.as_ref() {
                let margin_equity =
                    cash + pos.current_value(candle.close) + pos.unreinvested_dividends;
                let exposure = pos.quantity * candle.close;
                if margin_equity > 0.0 && exposure > max_leverage_used * margin_equity {
                    max_leverage_used = exposure / margin_equity;
                }
            }

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
                indicator_index: None,
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
            let executed = self.dispatch_entry_signal(
                &signal,
                i,
                candles,
                &mut position,
                &mut cash,
                &mut trades,
                &sizing_series,
                &mut pending_orders,
            );

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
                    indicator_index: None,
                };
                let follow = strategy.on_candle(&ctx2);
                if !follow.is_hold() && follow.strength.value() >= self.config.min_signal_strength {
                    let follow_executed = self.dispatch_entry_signal(
                        &follow,
                        i,
                        candles,
                        &mut position,
                        &mut cash,
                        &mut trades,
                        &sizing_series,
                        &mut pending_orders,
                    );
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
        // A position left open reaches no trade, so its accrued costs, income,
        // and span since its last logged exit are folded in here.
        if let Some(ref pos) = position {
            metrics.total_financing_cost += pos.financing_cost_accrued;
            metrics.total_dividend_income += pos.dividend_income;
            metrics.total_commission += pos.entry_commission;
            if let (Some(first), Some(last)) = (candles.first(), candles.last())
                && last.timestamp > first.timestamp
            {
                let counted_to = trades
                    .iter()
                    .filter(|t| t.entry_timestamp == pos.entry_timestamp)
                    .map(|t| t.exit_timestamp)
                    .max()
                    .unwrap_or(pos.entry_timestamp);
                let span = (last.timestamp - first.timestamp) as f64;
                let open_span = (last.timestamp - counted_to).max(0) as f64;
                metrics.time_in_market_pct =
                    (metrics.time_in_market_pct + open_span / span).min(1.0);
            }
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

    /// Resync the bar's already-pushed equity point to realized cash after an
    /// intrabar exit, so it isn't marked at a close price never traded through.
    ///
    /// The pre-exit snapshot may also have raised `peak_equity` to a value the
    /// account never held, so the peak reverts to the bar-entry value
    /// (`peak_before_bar`, the running max over every earlier point) or cash.
    fn resync_equity_point(
        equity_curve: &mut [EquityPoint],
        peak_equity: &mut f64,
        peak_before_bar: f64,
        cash: f64,
    ) {
        *peak_equity = peak_before_bar.max(cash);
        if let Some(last) = equity_curve.last_mut() {
            last.equity = cash;
            last.drawdown_pct = if *peak_equity > 0.0 {
                (*peak_equity - cash) / *peak_equity
            } else {
                0.0
            };
        }
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

    /// Dispatch a signal's order type: `Market` (and non-Long/Short directions)
    /// execute against the next bar's open; a Long/Short signal carrying a
    /// limit/stop order type is queued as a [`PendingOrder`] instead.
    #[allow(clippy::too_many_arguments)]
    fn dispatch_entry_signal(
        &self,
        signal: &Signal,
        i: usize,
        candles: &[Candle],
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
        sizing_series: &SizingSeries,
        pending_orders: &mut Vec<PendingOrder>,
    ) -> bool {
        match &signal.order_type {
            OrderType::Market => {
                if let Some(fill_candle) = candles.get(i + 1) {
                    // Only an entry reads the sizing context; exits and scale
                    // signals get a default so the Kelly window isn't rebuilt.
                    let sizing = match signal.direction {
                        SignalDirection::Long | SignalDirection::Short => {
                            self.build_sizing_context(i, sizing_series, trades)
                        }
                        _ => SizingContext::default(),
                    };
                    self.execute_signal(signal, fill_candle, position, cash, trades, &sizing)
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
                if matches!(signal.direction, SignalDirection::Short) && !self.config.allow_short {
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
                    let sizing = SizingContext::default();
                    self.execute_signal(signal, fill_candle, position, cash, trades, &sizing)
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "simulate_tests.rs"]
mod tests;
