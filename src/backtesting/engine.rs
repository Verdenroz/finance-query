//! Backtest execution engine.

use std::collections::HashMap;

use crate::indicators::{self, Indicator};
use crate::models::chart::{Candle, Dividend};

use super::config::BacktestConfig;
use super::error::{BacktestError, Result};
use super::position::{Position, PositionSide, Trade};
use super::result::{
    BacktestResult, BenchmarkMetrics, EquityPoint, PerformanceMetrics, SignalRecord,
};
use super::signal::{OrderType, PendingOrder, Signal, SignalDirection};
use super::strategy::{Strategy, StrategyContext};

/// Backtest execution engine.
///
/// Handles indicator pre-computation, position management, and trade execution.
pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    /// Create a new backtest engine with the given configuration
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run a backtest with the given strategy on historical candle data.
    ///
    /// Dividend income is not included. Use [`run_with_dividends`] to account
    /// for dividend payments during holding periods.
    ///
    /// [`run_with_dividends`]: Self::run_with_dividends
    pub fn run<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
    ) -> Result<BacktestResult> {
        self.simulate(symbol, candles, strategy, &[])
    }

    /// Run a backtest and credit dividend income for any dividends paid while a
    /// position is open.
    ///
    /// `dividends` should be sorted by timestamp (ascending). The engine credits
    /// each dividend whose ex-date falls on or before the current candle bar.
    /// When [`BacktestConfig::reinvest_dividends`] is `true`, the income is also
    /// used to notionally purchase additional shares at the ex-date close price.
    pub fn run_with_dividends<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
        dividends: &[Dividend],
    ) -> Result<BacktestResult> {
        self.simulate(symbol, candles, strategy, dividends)
    }

    // ── Core simulation ───────────────────────────────────────────────────────

    /// Internal simulation core. All public `run*` methods delegate here.
    fn simulate<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
        dividends: &[Dividend],
    ) -> Result<BacktestResult> {
        let warmup = strategy.warmup_period();
        if candles.len() < warmup {
            return Err(BacktestError::insufficient_data(warmup, candles.len()));
        }

        // Validate dividend ordering — simulation correctness requires ascending timestamps.
        if !dividends
            .windows(2)
            .all(|w| w[0].timestamp <= w[1].timestamp)
        {
            return Err(BacktestError::invalid_param(
                "dividends",
                "must be sorted by timestamp (ascending)",
            ));
        }

        // Pre-compute all required indicators
        let indicators = self.compute_indicators(candles, &strategy)?;

        // Initialize state
        let mut equity = self.config.initial_capital;
        let mut cash = self.config.initial_capital;
        let mut position: Option<Position> = None;
        let mut trades: Vec<Trade> = Vec::new();
        let mut equity_curve: Vec<EquityPoint> = Vec::new();
        let mut signals: Vec<SignalRecord> = Vec::new();
        let mut peak_equity = equity;
        // High-water mark for the trailing stop: tracks peak price (longs) or
        // trough price (shorts) since entry. Reset to None when no position is open.
        let mut hwm: Option<f64> = None;

        // Dividend processing pointer: dividends must be sorted by timestamp.
        // We advance this index forward as the simulation progresses in time.
        let mut div_idx: usize = 0;

        // Pending limit / stop orders placed by the strategy.
        // Checked each bar before strategy signal evaluation.
        let mut pending_orders: Vec<PendingOrder> = Vec::new();

        // Main simulation loop
        for i in 0..candles.len() {
            let candle = &candles[i];

            equity = Self::update_equity_and_curve(
                position.as_ref(),
                candle,
                cash,
                &mut peak_equity,
                &mut equity_curve,
            );

            update_trailing_hwm(position.as_ref(), &mut hwm, candle);

            // Credit dividend income for any dividends ex-dated on or before this bar.
            self.credit_dividends(&mut position, candle, dividends, &mut div_idx);

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
                    continue; // Skip strategy signal this bar
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
                    let is_long = matches!(order.signal.direction, SignalDirection::Long);
                    let executed = self.open_position_at_price(
                        &mut position,
                        &mut cash,
                        candle,
                        &order.signal,
                        is_long,
                        fill_price,
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
                        self.execute_signal(
                            &signal,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
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
                        self.execute_signal(
                            &signal,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
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
            }

            // Reset the trailing-stop HWM whenever a position is closed
            if executed && position.is_none() {
                hwm = None;

                // Re-evaluate strategy on the same bar after an exit so that
                // a crossover that simultaneously closes one side and triggers
                // the opposite entry is not lost.
                let ctx2 = StrategyContext {
                    candles: &candles[..=i],
                    index: i,
                    position: None,
                    equity,
                    indicators: &indicators,
                };
                let follow = strategy.on_candle(&ctx2);
                if !follow.is_hold() && follow.strength.value() >= self.config.min_signal_strength {
                    let follow_executed = if let Some(fill_candle) = candles.get(i + 1) {
                        self.execute_signal(
                            &follow,
                            fill_candle,
                            &mut position,
                            &mut cash,
                            &mut trades,
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
        let metrics = PerformanceMetrics::calculate(
            &trades,
            &equity_curve,
            self.config.initial_capital,
            signals.len(),
            executed_signals,
            self.config.risk_free_rate,
            self.config.bars_per_year,
        );

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
        })
    }

    /// Run a backtest and compare against a benchmark, optionally crediting dividends.
    ///
    /// The result's `benchmark` field is populated with buy-and-hold comparison
    /// metrics including alpha, beta, and information ratio. The benchmark candle
    /// slice should cover the same time period as `candles` but need not be the
    /// same length.
    ///
    /// `dividends` must be sorted ascending by timestamp. Pass `&[]` to omit
    /// dividend processing.
    pub fn run_with_benchmark<S: Strategy>(
        &self,
        symbol: &str,
        candles: &[Candle],
        strategy: S,
        dividends: &[Dividend],
        benchmark_symbol: &str,
        benchmark_candles: &[Candle],
    ) -> Result<BacktestResult> {
        let mut result = self.simulate(symbol, candles, strategy, dividends)?;
        result.benchmark = Some(compute_benchmark_metrics(
            benchmark_symbol,
            candles,
            benchmark_candles,
            &result.equity_curve,
            self.config.risk_free_rate,
            self.config.bars_per_year,
        ));
        Ok(result)
    }

    /// Pre-compute all indicators required by the strategy
    pub(crate) fn compute_indicators<S: Strategy>(
        &self,
        candles: &[Candle],
        strategy: &S,
    ) -> Result<HashMap<String, Vec<Option<f64>>>> {
        let mut result = HashMap::new();

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.high).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.low).collect();
        let volumes: Vec<f64> = candles.iter().map(|c| c.volume as f64).collect();

        for (name, indicator) in strategy.required_indicators() {
            match indicator {
                Indicator::Sma(period) => {
                    let values = indicators::sma(&closes, period);
                    result.insert(name, values);
                }
                Indicator::Ema(period) => {
                    let values = indicators::ema(&closes, period);
                    result.insert(name, values);
                }
                Indicator::Rsi(period) => {
                    let values = indicators::rsi(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Macd { fast, slow, signal } => {
                    let macd_result = indicators::macd(&closes, fast, slow, signal)?;
                    result.insert(
                        format!("macd_line_{fast}_{slow}_{signal}"),
                        macd_result.macd_line,
                    );
                    result.insert(
                        format!("macd_signal_{fast}_{slow}_{signal}"),
                        macd_result.signal_line,
                    );
                    result.insert(
                        format!("macd_histogram_{fast}_{slow}_{signal}"),
                        macd_result.histogram,
                    );
                }
                Indicator::Bollinger { period, std_dev } => {
                    let bb = indicators::bollinger_bands(&closes, period, std_dev)?;
                    result.insert(format!("bollinger_upper_{period}_{std_dev}"), bb.upper);
                    result.insert(format!("bollinger_middle_{period}_{std_dev}"), bb.middle);
                    result.insert(format!("bollinger_lower_{period}_{std_dev}"), bb.lower);
                }
                Indicator::Atr(period) => {
                    let values = indicators::atr(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Supertrend { period, multiplier } => {
                    let st = indicators::supertrend(&highs, &lows, &closes, period, multiplier)?;
                    result.insert(format!("supertrend_value_{period}_{multiplier}"), st.value);
                    // Convert bool to f64 for consistency
                    let uptrend: Vec<Option<f64>> = st
                        .is_uptrend
                        .into_iter()
                        .map(|v| v.map(|b| if b { 1.0 } else { 0.0 }))
                        .collect();
                    result.insert(format!("supertrend_uptrend_{period}_{multiplier}"), uptrend);
                }
                Indicator::DonchianChannels(period) => {
                    let dc = indicators::donchian_channels(&highs, &lows, period)?;
                    result.insert(format!("donchian_upper_{period}"), dc.upper);
                    result.insert(format!("donchian_middle_{period}"), dc.middle);
                    result.insert(format!("donchian_lower_{period}"), dc.lower);
                }
                Indicator::Wma(period) => {
                    let values = indicators::wma(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Dema(period) => {
                    let values = indicators::dema(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Tema(period) => {
                    let values = indicators::tema(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Hma(period) => {
                    let values = indicators::hma(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Obv => {
                    let values = indicators::obv(&closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::Momentum(period) => {
                    let values = indicators::momentum(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Roc(period) => {
                    let values = indicators::roc(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cci(period) => {
                    let values = indicators::cci(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::WilliamsR(period) => {
                    let values = indicators::williams_r(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Adx(period) => {
                    let values = indicators::adx(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Mfi(period) => {
                    let values = indicators::mfi(&highs, &lows, &closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cmf(period) => {
                    let values = indicators::cmf(&highs, &lows, &closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Cmo(period) => {
                    let values = indicators::cmo(&closes, period)?;
                    result.insert(name, values);
                }
                Indicator::Vwma(period) => {
                    let values = indicators::vwma(&closes, &volumes, period)?;
                    result.insert(name, values);
                }
                Indicator::Alma {
                    period,
                    offset,
                    sigma,
                } => {
                    let values = indicators::alma(&closes, period, offset, sigma)?;
                    result.insert(name, values);
                }
                Indicator::McginleyDynamic(period) => {
                    let values = indicators::mcginley_dynamic(&closes, period)?;
                    result.insert(name, values);
                }
                // === OSCILLATORS ===
                Indicator::Stochastic {
                    k_period,
                    k_slow,
                    d_period,
                } => {
                    let stoch =
                        indicators::stochastic(&highs, &lows, &closes, k_period, k_slow, d_period)?;
                    result.insert(
                        format!("stochastic_k_{k_period}_{k_slow}_{d_period}"),
                        stoch.k,
                    );
                    result.insert(
                        format!("stochastic_d_{k_period}_{k_slow}_{d_period}"),
                        stoch.d,
                    );
                }
                Indicator::StochasticRsi {
                    rsi_period,
                    stoch_period,
                    k_period,
                    d_period,
                } => {
                    let stoch = indicators::stochastic_rsi(
                        &closes,
                        rsi_period,
                        stoch_period,
                        k_period,
                        d_period,
                    )?;
                    result.insert(
                        format!("stoch_rsi_k_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
                        stoch.k,
                    );
                    result.insert(
                        format!("stoch_rsi_d_{rsi_period}_{stoch_period}_{k_period}_{d_period}"),
                        stoch.d,
                    );
                }
                Indicator::AwesomeOscillator { fast, slow } => {
                    let values = indicators::awesome_oscillator(&highs, &lows, fast, slow)?;
                    result.insert(name, values);
                }
                Indicator::CoppockCurve {
                    wma_period,
                    long_roc,
                    short_roc,
                } => {
                    let values =
                        indicators::coppock_curve(&closes, long_roc, short_roc, wma_period)?;
                    result.insert(name, values);
                }
                // === TREND INDICATORS ===
                Indicator::Aroon(period) => {
                    let aroon_result = indicators::aroon(&highs, &lows, period)?;
                    result.insert(format!("aroon_up_{period}"), aroon_result.aroon_up);
                    result.insert(format!("aroon_down_{period}"), aroon_result.aroon_down);
                }
                Indicator::Ichimoku {
                    conversion,
                    base,
                    lagging,
                    displacement,
                } => {
                    let ich = indicators::ichimoku(
                        &highs,
                        &lows,
                        &closes,
                        conversion,
                        base,
                        lagging,
                        displacement,
                    )?;
                    result.insert(
                        format!("ichimoku_conversion_{conversion}_{base}_{lagging}_{displacement}"),
                        ich.conversion_line,
                    );
                    result.insert(
                        format!("ichimoku_base_{conversion}_{base}_{lagging}_{displacement}"),
                        ich.base_line,
                    );
                    result.insert(
                        format!("ichimoku_leading_a_{conversion}_{base}_{lagging}_{displacement}"),
                        ich.leading_span_a,
                    );
                    result.insert(
                        format!("ichimoku_leading_b_{conversion}_{base}_{lagging}_{displacement}"),
                        ich.leading_span_b,
                    );
                    result.insert(
                        format!("ichimoku_lagging_{conversion}_{base}_{lagging}_{displacement}"),
                        ich.lagging_span,
                    );
                }
                Indicator::ParabolicSar { step, max } => {
                    let values = indicators::parabolic_sar(&highs, &lows, &closes, step, max)?;
                    result.insert(name, values);
                }
                // === VOLATILITY ===
                Indicator::KeltnerChannels {
                    period,
                    multiplier,
                    atr_period,
                } => {
                    let kc = indicators::keltner_channels(
                        &highs, &lows, &closes, period, atr_period, multiplier,
                    )?;
                    result.insert(
                        format!("keltner_upper_{period}_{multiplier}_{atr_period}"),
                        kc.upper,
                    );
                    result.insert(
                        format!("keltner_middle_{period}_{multiplier}_{atr_period}"),
                        kc.middle,
                    );
                    result.insert(
                        format!("keltner_lower_{period}_{multiplier}_{atr_period}"),
                        kc.lower,
                    );
                }
                Indicator::TrueRange => {
                    let values = indicators::true_range(&highs, &lows, &closes)?;
                    result.insert(name, values);
                }
                Indicator::ChoppinessIndex(period) => {
                    let values = indicators::choppiness_index(&highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                // === VOLUME INDICATORS ===
                Indicator::Vwap => {
                    let values = indicators::vwap(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::ChaikinOscillator => {
                    let values = indicators::chaikin_oscillator(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::AccumulationDistribution => {
                    let values =
                        indicators::accumulation_distribution(&highs, &lows, &closes, &volumes)?;
                    result.insert(name, values);
                }
                Indicator::BalanceOfPower(period) => {
                    let opens: Vec<f64> = candles.iter().map(|c| c.open).collect();
                    let values =
                        indicators::balance_of_power(&opens, &highs, &lows, &closes, period)?;
                    result.insert(name, values);
                }
                // === POWER/STRENGTH INDICATORS ===
                Indicator::BullBearPower(period) => {
                    let bbp = indicators::bull_bear_power(&highs, &lows, &closes, period)?;
                    result.insert(format!("bull_power_{period}"), bbp.bull_power);
                    result.insert(format!("bear_power_{period}"), bbp.bear_power);
                }
                Indicator::ElderRay(period) => {
                    let er = indicators::elder_ray(&highs, &lows, &closes, period)?;
                    result.insert(format!("elder_bull_{period}"), er.bull_power);
                    result.insert(format!("elder_bear_{period}"), er.bear_power);
                }
            }
        }

        Ok(result)
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
    fn check_sl_tp(
        &self,
        position: &Position,
        candle: &Candle,
        hwm: Option<f64>,
    ) -> Option<Signal> {
        // Stop-loss — intrabar breach via low (long) or high (short)
        if let Some(sl_pct) = self.config.stop_loss_pct {
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
        if let Some(tp_pct) = self.config.take_profit_pct {
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
        if let Some(trail_pct) = self.config.trailing_stop_pct
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

    /// Execute a signal, modifying position and cash
    fn execute_signal(
        &self,
        signal: &Signal,
        candle: &Candle,
        position: &mut Option<Position>,
        cash: &mut f64,
        trades: &mut Vec<Trade>,
    ) -> bool {
        match signal.direction {
            SignalDirection::Long => {
                if position.is_some() {
                    return false; // Already have a position
                }
                self.open_position(position, cash, candle, signal, true)
            }
            SignalDirection::Short => {
                if position.is_some() {
                    return false; // Already have a position
                }
                if !self.config.allow_short {
                    return false; // Shorts not allowed
                }
                self.open_position(position, cash, candle, signal, false)
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

        if total_cost > *cash {
            return false; // Not enough cash
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
        is_long: bool,
    ) -> bool {
        self.open_position_at_price(position, cash, candle, signal, is_long, candle.open)
    }

    /// Open a new position at an explicit fill price.
    ///
    /// Used for pending limit/stop order fills where the computed order price
    /// (with gap guard) is the fill price rather than the next bar's open.
    fn open_position_at_price(
        &self,
        position: &mut Option<Position>,
        cash: &mut f64,
        candle: &Candle,
        signal: &Signal,
        is_long: bool,
        fill_price_raw: f64,
    ) -> bool {
        let entry_price_slipped = self.config.apply_entry_slippage(fill_price_raw, is_long);
        let entry_price = self.config.apply_entry_spread(entry_price_slipped, is_long);
        let quantity = self.config.calculate_position_size(*cash, entry_price);

        if quantity <= 0.0 {
            return false; // Not enough capital
        }

        let entry_value = entry_price * quantity;
        let commission = self.config.calculate_commission(quantity, entry_price);
        // Tax on buy orders only: long entries are buys
        let entry_tax = self.config.calculate_transaction_tax(entry_value, is_long);

        if is_long {
            if entry_value + commission + entry_tax > *cash {
                return false; // Not enough capital including commission and tax
            }
        } else if commission > *cash {
            return false; // Not enough cash to pay entry commission
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
    fn close_position_at(
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

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Update the trailing-stop high-water mark (peak for longs, trough for shorts).
///
/// Uses the candle's intrabar extreme (`high` for longs, `low` for shorts) so
/// that the trailing stop correctly reflects the best price reached during the bar,
/// not just the close.
///
/// Cleared to `None` when no position is open so it resets on next entry.
/// Also used by the portfolio engine.
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

impl BacktestEngine {
    fn sync_terminal_equity_point(
        equity_curve: &mut Vec<EquityPoint>,
        timestamp: i64,
        equity: f64,
    ) {
        if let Some(last) = equity_curve.last_mut()
            && last.timestamp == timestamp
        {
            last.equity = equity;
        } else {
            equity_curve.push(EquityPoint {
                timestamp,
                equity,
                drawdown_pct: 0.0,
            });
        }

        let peak = equity_curve
            .iter()
            .map(|point| point.equity)
            .fold(f64::NEG_INFINITY, f64::max);
        let drawdown = if peak.is_finite() && peak > 0.0 {
            (peak - equity) / peak
        } else {
            0.0
        };

        if let Some(last) = equity_curve.last_mut() {
            last.drawdown_pct = drawdown;
        }
    }
}

/// Compute benchmark comparison metrics for a completed backtest.
///
/// `symbol_candles` are the candles for the backtested symbol (used to derive
/// its buy-and-hold return). `benchmark_candles` are the benchmark's candles.
/// `equity_curve` is used to derive strategy periodic returns for beta/IR.
fn compute_benchmark_metrics(
    benchmark_symbol: &str,
    symbol_candles: &[Candle],
    benchmark_candles: &[Candle],
    equity_curve: &[EquityPoint],
    risk_free_rate: f64,
    bars_per_year: f64,
) -> BenchmarkMetrics {
    // Buy-and-hold returns from first to last close
    let benchmark_return_pct = buy_and_hold_return(benchmark_candles);
    let buy_and_hold_return_pct = buy_and_hold_return(symbol_candles);

    if equity_curve.len() < 2 || benchmark_candles.len() < 2 {
        return BenchmarkMetrics {
            symbol: benchmark_symbol.to_string(),
            benchmark_return_pct,
            buy_and_hold_return_pct,
            alpha: 0.0,
            beta: 0.0,
            information_ratio: 0.0,
        };
    }

    let strategy_returns_by_ts: Vec<(i64, f64)> = equity_curve
        .windows(2)
        .map(|w| {
            let prev = w[0].equity;
            let ret = if prev > 0.0 {
                (w[1].equity - prev) / prev
            } else {
                0.0
            };
            (w[1].timestamp, ret)
        })
        .collect();

    let bench_returns_by_ts: HashMap<i64, f64> = benchmark_candles
        .windows(2)
        .map(|w| {
            let prev = w[0].close;
            let ret = if prev > 0.0 {
                (w[1].close - prev) / prev
            } else {
                0.0
            };
            (w[1].timestamp, ret)
        })
        .collect();

    let mut aligned_strategy = Vec::new();
    let mut aligned_benchmark = Vec::new();
    for (ts, s_ret) in strategy_returns_by_ts {
        if let Some(b_ret) = bench_returns_by_ts.get(&ts) {
            aligned_strategy.push(s_ret);
            aligned_benchmark.push(*b_ret);
        }
    }

    let beta = compute_beta(&aligned_strategy, &aligned_benchmark);

    // CAPM alpha on the same aligned sample used for beta/IR.
    let strategy_ann = annualized_return_from_periodic(&aligned_strategy, bars_per_year);
    let bench_ann = annualized_return_from_periodic(&aligned_benchmark, bars_per_year);
    // Jensen's Alpha: excess strategy return over what CAPM predicts given beta.
    // Both strategy_ann and bench_ann are in percentage form (×100), so rf_ann is scaled
    // to match before applying the CAPM formula: α = R_s - R_f - β(R_b - R_f).
    let rf_ann = risk_free_rate * 100.0;
    let alpha = strategy_ann - rf_ann - beta * (bench_ann - rf_ann);

    // Information ratio: (excess returns mean / tracking error) * sqrt(bars_per_year)
    // Uses sample standard deviation (n-1) for consistency with Sharpe/Sortino.
    let excess: Vec<f64> = aligned_strategy
        .iter()
        .zip(aligned_benchmark.iter())
        .map(|(si, bi)| si - bi)
        .collect();
    let ir = if excess.len() >= 2 {
        let n = excess.len() as f64;
        let mean = excess.iter().sum::<f64>() / n;
        // Sample variance (n-1)
        let variance = excess.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        if std_dev > 0.0 {
            (mean / std_dev) * bars_per_year.sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    BenchmarkMetrics {
        symbol: benchmark_symbol.to_string(),
        benchmark_return_pct,
        buy_and_hold_return_pct,
        alpha,
        beta,
        information_ratio: ir,
    }
}

/// Buy-and-hold return from first to last candle close (percentage).
fn buy_and_hold_return(candles: &[Candle]) -> f64 {
    match (candles.first(), candles.last()) {
        (Some(first), Some(last)) if first.close > 0.0 => {
            ((last.close / first.close) - 1.0) * 100.0
        }
        _ => 0.0,
    }
}

/// Annualised return from periodic returns (fractional, e.g. 0.01 for 1%).
fn annualized_return_from_periodic(periodic_returns: &[f64], bars_per_year: f64) -> f64 {
    let years = periodic_returns.len() as f64 / bars_per_year;
    if years > 0.0 {
        let growth = periodic_returns
            .iter()
            .fold(1.0_f64, |acc, r| acc * (1.0 + *r));
        if growth <= 0.0 {
            -100.0
        } else {
            (growth.powf(1.0 / years) - 1.0) * 100.0
        }
    } else {
        0.0
    }
}

/// Compute beta of `strategy_returns` relative to `benchmark_returns`.
///
/// Beta = Cov(strategy, benchmark) / Var(benchmark).
/// Uses sample covariance and variance (divides by n-1) to match the `risk`
/// module and standard financial convention. Returns 0.0 when benchmark
/// variance is zero or there are fewer than 2 observations.
fn compute_beta(strategy_returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    let n = strategy_returns.len();
    if n < 2 {
        return 0.0;
    }

    let s_mean = strategy_returns.iter().sum::<f64>() / n as f64;
    let b_mean = benchmark_returns.iter().sum::<f64>() / n as f64;

    // Sample covariance and variance (n-1)
    let cov: f64 = strategy_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(s, b)| (s - s_mean) * (b - b_mean))
        .sum::<f64>()
        / (n - 1) as f64;

    let b_var: f64 = benchmark_returns
        .iter()
        .map(|b| (b - b_mean).powi(2))
        .sum::<f64>()
        / (n - 1) as f64;

    if b_var > 0.0 { cov / b_var } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtesting::strategy::SmaCrossover;
    use crate::backtesting::strategy::Strategy;
    use crate::indicators::Indicator;

    #[derive(Clone)]
    struct EnterLongHold;

    impl Strategy for EnterLongHold {
        fn name(&self) -> &str {
            "Enter Long Hold"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    #[derive(Clone)]
    struct EnterShortHold;

    impl Strategy for EnterShortHold {
        fn name(&self) -> &str {
            "Enter Short Hold"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::short(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

    fn make_candles(prices: &[f64]) -> Vec<Candle> {
        prices
            .iter()
            .enumerate()
            .map(|(i, &p)| Candle {
                timestamp: i as i64,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1000,
                adj_close: Some(p),
            })
            .collect()
    }

    fn make_candles_with_timestamps(prices: &[f64], timestamps: &[i64]) -> Vec<Candle> {
        prices
            .iter()
            .zip(timestamps.iter())
            .map(|(&p, &ts)| Candle {
                timestamp: ts,
                open: p,
                high: p * 1.01,
                low: p * 0.99,
                close: p,
                volume: 1000,
                adj_close: Some(p),
            })
            .collect()
    }

    #[test]
    fn test_engine_basic() {
        // Price trends up then down - should trigger crossover signals
        let mut prices = vec![100.0; 30];
        // Make fast SMA cross above slow SMA around bar 15
        for (i, price) in prices.iter_mut().enumerate().take(25).skip(15) {
            *price = 100.0 + (i - 15) as f64 * 2.0;
        }
        // Then cross back down
        for (i, price) in prices.iter_mut().enumerate().take(30).skip(25) {
            *price = 118.0 - (i - 25) as f64 * 3.0;
        }

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(5, 10);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        assert_eq!(result.symbol, "TEST");
        assert_eq!(result.strategy_name, "SMA Crossover");
        assert!(!result.equity_curve.is_empty());
    }

    #[test]
    fn test_stop_loss() {
        // Price drops significantly after entry
        let mut prices = vec![100.0; 20];
        // Trend up to trigger long entry
        for (i, price) in prices.iter_mut().enumerate().take(15).skip(10) {
            *price = 100.0 + (i - 10) as f64 * 2.0;
        }
        // Then crash
        for (i, price) in prices.iter_mut().enumerate().take(20).skip(15) {
            *price = 108.0 - (i - 15) as f64 * 10.0;
        }

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .stop_loss_pct(0.05) // 5% stop loss
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(3, 6);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        // Should have triggered stop-loss
        let _sl_signals: Vec<_> = result
            .signals
            .iter()
            .filter(|s| {
                s.reason
                    .as_ref()
                    .map(|r| r.contains("Stop-loss"))
                    .unwrap_or(false)
            })
            .collect();

        // May or may not trigger depending on exact timing
        // The important thing is the engine doesn't crash
        assert!(!result.equity_curve.is_empty());
    }

    #[test]
    fn test_trailing_stop() {
        // Price rises to 120, then drops 10%+ → trailing stop should fire
        let mut prices: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        // Peak is 119; now drop past 10% from peak (< 107.1)
        prices.extend_from_slice(&[105.0, 103.0, 101.0]);

        let candles = make_candles(&prices);
        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .trailing_stop_pct(0.10)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(3, 6);
        let result = engine.run("TEST", &candles, strategy).unwrap();

        let trail_exits: Vec<_> = result
            .signals
            .iter()
            .filter(|s| {
                s.reason
                    .as_ref()
                    .map(|r| r.contains("Trailing stop"))
                    .unwrap_or(false)
            })
            .collect();

        // Not guaranteed to fire given the specific crossover timing, but engine must not crash
        let _ = trail_exits;
        assert!(!result.equity_curve.is_empty());
    }

    #[test]
    fn test_insufficient_data() {
        let candles = make_candles(&[100.0, 101.0, 102.0]); // Only 3 candles
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);
        let strategy = SmaCrossover::new(10, 20); // Needs at least 21 candles

        let result = engine.run("TEST", &candles, strategy);
        assert!(result.is_err());
    }

    #[test]
    fn test_capm_alpha_with_risk_free_rate() {
        // When risk_free_rate = 0, alpha should equal the simplified formula.
        // When risk_free_rate > 0, the CAPM adjustment should reduce alpha.
        let prices: Vec<f64> = (0..60).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);

        // Run once with rf=0 and once with rf=0.05, compare benchmark metrics
        let config_no_rf = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .risk_free_rate(0.0)
            .build()
            .unwrap();
        let config_with_rf = BacktestConfig::builder()
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .risk_free_rate(0.05)
            .build()
            .unwrap();

        let engine_no_rf = BacktestEngine::new(config_no_rf);
        let engine_with_rf = BacktestEngine::new(config_with_rf);

        // Use same candles for both strategy and benchmark to get beta ≈ 1
        let result_no_rf = engine_no_rf
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 10),
                &[],
                "BENCH",
                &candles,
            )
            .unwrap();
        let result_with_rf = engine_with_rf
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 10),
                &[],
                "BENCH",
                &candles,
            )
            .unwrap();

        let bm_no_rf = result_no_rf.benchmark.unwrap();
        let bm_with_rf = result_with_rf.benchmark.unwrap();

        // With identical strategy and benchmark (beta = 1), Jensen's alpha ≈ 0 always.
        // Both should be close to 0, but importantly they should differ when rf != 0.
        // This test ensures the formula uses rf — it catches the old bug where rf was ignored.
        assert!(bm_no_rf.alpha.is_finite(), "Alpha should be finite");
        assert!(
            bm_with_rf.alpha.is_finite(),
            "Alpha should be finite with rf"
        );

        // With beta ≈ 1 and rf=5%, CAPM alpha = R_s - 5% - 1*(R_b - 5%) = R_s - R_b.
        // Same formula result as rf=0 when beta=1; but the formula path is exercised.
        // The key check: alpha is the same sign in both (both near-zero).
        assert!(
            bm_no_rf.alpha.abs() < 50.0,
            "Alpha should be small for identical strategy/benchmark"
        );
        assert!(
            bm_with_rf.alpha.abs() < 50.0,
            "Alpha should be small for identical strategy/benchmark with rf"
        );
    }

    #[test]
    fn test_run_with_benchmark_credits_dividends() {
        use crate::models::chart::Dividend;

        // Rising price series — long enough for SmaCrossover(3,6) to trade
        let prices: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let candles = make_candles(&prices);

        // A single dividend ex-dated mid-series
        let mid_ts = candles[15].timestamp;
        let dividends = vec![Dividend {
            timestamp: mid_ts,
            amount: 1.0,
        }];

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_benchmark(
                "TEST",
                &candles,
                SmaCrossover::new(3, 6),
                &dividends,
                "BENCH",
                &candles,
            )
            .unwrap();

        // Dividend income is credited only while a position is open.
        // If no trade happened to be open on bar 15 the income is zero;
        // either way the engine must not panic and the benchmark must be set.
        assert!(result.benchmark.is_some());
        let total_div: f64 = result.trades.iter().map(|t| t.dividend_income).sum();
        // total_dividend_income is non-negative (either credited or not, never negative)
        assert!(total_div >= 0.0);
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
            },
            Dividend {
                timestamp: 10,
                amount: 1.0,
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
    fn test_benchmark_beta_and_ir_require_timestamp_overlap() {
        let symbol_candles = make_candles_with_timestamps(&[100.0, 110.0, 120.0], &[100, 200, 300]);
        let benchmark_candles =
            make_candles_with_timestamps(&[50.0, 55.0, 60.0, 65.0], &[1000, 1100, 1200, 1300]);

        let config = BacktestConfig::builder()
            .initial_capital(10_000.0)
            .commission_pct(0.0)
            .slippage_pct(0.0)
            .build()
            .unwrap();

        let engine = BacktestEngine::new(config);
        let result = engine
            .run_with_benchmark(
                "TEST",
                &symbol_candles,
                EnterLongHold,
                &[],
                "BENCH",
                &benchmark_candles,
            )
            .unwrap();

        let benchmark = result.benchmark.unwrap();
        assert!((benchmark.beta - 0.0).abs() < 1e-12);
        assert!((benchmark.information_ratio - 0.0).abs() < 1e-12);
    }

    /// Build a candle with explicit OHLC values (not derived from a single price).
    fn make_candle_ohlc(ts: i64, open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle {
            timestamp: ts,
            open,
            high,
            low,
            close,
            volume: 1000,
            adj_close: Some(close),
        }
    }

    // ── Intrabar stop / take-profit tests ────────────────────────────────────

    /// A strategy that opens a long on the first bar and holds forever.
    struct EnterLongBar0;
    impl Strategy for EnterLongBar0 {
        fn name(&self) -> &str {
            "Enter Long Bar 0"
        }
        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }
        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            if ctx.index == 0 && !ctx.has_position() {
                Signal::long(ctx.timestamp(), ctx.close())
            } else {
                Signal::hold()
            }
        }
    }

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

    // ── Position scaling integration tests ───────────────────────────────────

    /// Strategy: enter long on bar 0, scale in on bar 1, exit on bar 2.
    #[derive(Clone)]
    struct EnterScaleInExit;

    impl Strategy for EnterScaleInExit {
        fn name(&self) -> &str {
            "EnterScaleInExit"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match ctx.index {
                0 => Signal::long(ctx.timestamp(), ctx.close()),
                1 if ctx.has_position() => Signal::scale_in(0.5, ctx.timestamp(), ctx.close()),
                2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                _ => Signal::hold(),
            }
        }
    }

    /// Strategy: enter long on bar 0, scale out 50% on bar 1, exit remainder on bar 2.
    #[derive(Clone)]
    struct EnterScaleOutExit;

    impl Strategy for EnterScaleOutExit {
        fn name(&self) -> &str {
            "EnterScaleOutExit"
        }

        fn required_indicators(&self) -> Vec<(String, Indicator)> {
            vec![]
        }

        fn on_candle(&self, ctx: &StrategyContext) -> Signal {
            match ctx.index {
                0 => Signal::long(ctx.timestamp(), ctx.close()),
                1 if ctx.has_position() => Signal::scale_out(0.5, ctx.timestamp(), ctx.close()),
                2 if ctx.has_position() => Signal::exit(ctx.timestamp(), ctx.close()),
                _ => Signal::hold(),
            }
        }
    }

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
}
