use std::collections::HashSet;

use finance_query::Interval;
use finance_query::backtesting::condition::HtfIndicatorSpec;
use finance_query::backtesting::{Signal, Strategy, StrategyContext};
use finance_query::indicators::Indicator;

use super::types::{
    BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup, LogicalOp,
    LongOrderType, ShortOrderType,
};

/// Relative tolerance for floating-point equality comparisons in strategy conditions.
/// Uses a relative epsilon so comparisons remain meaningful for both penny stocks
/// and high-priced assets (e.g. BRK-A ~$700k, BTC ~$90k).
const FLOAT_EPSILON: f64 = 1e-6;

/// A strategy built at runtime from the TUI's entry/exit condition groups.
///
/// The engine pre-computes all required indicators and passes them via
/// `StrategyContext::indicator()`. This strategy reads those values and
/// evaluates the user-defined conditions each bar.
#[derive(Clone)]
pub struct DynamicStrategy {
    pub name: String,
    pub entry: ConditionGroup,
    pub exit: ConditionGroup,
    pub short_entry: Option<ConditionGroup>,
    pub short_exit: Option<ConditionGroup>,
    /// Regime filter: if non-empty, all conditions must pass for entry signals to fire.
    pub regime: ConditionGroup,
    /// Number of bars to skip before generating signals.
    pub warmup_bars: usize,
    /// Conditions that trigger a scale-in (pyramid add) while in a position.
    pub scale_in: ConditionGroup,
    /// Fraction of current portfolio equity to allocate when scaling in (0.0–1.0).
    pub scale_in_fraction: f64,
    /// Conditions that trigger a partial exit (scale-out) while in a position.
    pub scale_out: ConditionGroup,
    /// Fraction of current position quantity to close when scaling out (0.0–1.0).
    pub scale_out_fraction: f64,
    /// Entry order type for long positions.
    pub entry_order_type: LongOrderType,
    /// Price offset fraction (stop trigger) for limit/stop long entries.
    pub entry_price_offset_pct: f64,
    /// Gap above the stop price for `StopLimitAbove` orders (fraction).
    /// `limit_price = stop_price * (1 + gap)`. Unused for other order types.
    pub entry_stop_limit_gap_pct: f64,
    /// Bars until a pending long entry order expires. None = GTC.
    pub entry_expires_bars: Option<usize>,
    /// Per-trade stop-loss override for long entries.
    pub entry_bracket_sl: Option<f64>,
    /// Per-trade take-profit override for long entries.
    pub entry_bracket_tp: Option<f64>,
    /// Per-trade trailing-stop override for long entries.
    pub entry_bracket_trail: Option<f64>,
    /// Entry order type for short positions.
    pub short_order_type: ShortOrderType,
    /// Price offset fraction for limit/stop short entries.
    pub short_price_offset_pct: f64,
    /// Bars until a pending short entry order expires. None = GTC.
    pub short_expires_bars: Option<usize>,
    /// Per-trade stop-loss override for short entries.
    pub short_bracket_sl: Option<f64>,
    /// Per-trade take-profit override for short entries.
    pub short_bracket_tp: Option<f64>,
    /// Per-trade trailing-stop override for short entries.
    pub short_bracket_trail: Option<f64>,
}

impl DynamicStrategy {
    pub fn new(name: String, entry: ConditionGroup, exit: ConditionGroup) -> Self {
        Self {
            name,
            entry,
            exit,
            short_entry: None,
            short_exit: None,
            regime: ConditionGroup::default(),
            warmup_bars: 0,
            scale_in: ConditionGroup::default(),
            scale_in_fraction: 0.25,
            scale_out: ConditionGroup::default(),
            scale_out_fraction: 0.50,
            entry_order_type: LongOrderType::Market,
            entry_price_offset_pct: 0.005,
            entry_stop_limit_gap_pct: 0.002,
            entry_expires_bars: None,
            entry_bracket_sl: None,
            entry_bracket_tp: None,
            entry_bracket_trail: None,
            short_order_type: ShortOrderType::Market,
            short_price_offset_pct: 0.005,
            short_expires_bars: None,
            short_bracket_sl: None,
            short_bracket_tp: None,
            short_bracket_trail: None,
        }
    }
}

impl Strategy for DynamicStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn required_indicators(&self) -> Vec<(String, Indicator)> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        let groups: [Option<&ConditionGroup>; 7] = [
            Some(&self.entry),
            Some(&self.exit),
            self.short_entry.as_ref(),
            self.short_exit.as_ref(),
            Some(&self.regime),
            Some(&self.scale_in),
            Some(&self.scale_out),
        ];

        for group in groups.into_iter().flatten() {
            for cond in &group.conditions {
                if cond.htf_interval.is_none() {
                    collect_indicator(&cond.indicator, &mut result, &mut seen);
                    if let CompareTarget::Indicator(ref other) = cond.target {
                        collect_indicator(other, &mut result, &mut seen);
                    }
                }
            }
        }

        result
    }

    fn htf_requirements(&self) -> Vec<HtfIndicatorSpec> {
        let mut requirements = Vec::new();
        let mut seen = HashSet::new();

        let groups: [Option<&ConditionGroup>; 7] = [
            Some(&self.entry),
            Some(&self.exit),
            self.short_entry.as_ref(),
            self.short_exit.as_ref(),
            Some(&self.regime),
            Some(&self.scale_in),
            Some(&self.scale_out),
        ];

        for group in groups.into_iter().flatten() {
            for cond in &group.conditions {
                if let Some(interval) = cond.htf_interval {
                    collect_htf_indicator(interval, &cond.indicator, &mut requirements, &mut seen);
                    if let CompareTarget::Indicator(ref other) = cond.target {
                        collect_htf_indicator(interval, other, &mut requirements, &mut seen);
                    }
                }
            }
        }

        requirements.sort_by(|a, b| a.htf_key.cmp(&b.htf_key));
        requirements
    }

    fn warmup_period(&self) -> usize {
        self.warmup_bars
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let price = ctx.close();
        let ts = ctx.timestamp();
        let has_pos = ctx.has_position();

        // Check regime filter — suppresses ALL entry signals when it fails.
        let regime_ok = self.regime.conditions.is_empty() || eval_group(&self.regime, ctx);

        // Priority 1: full exit (not gated by regime filter).
        if has_pos {
            if ctx.is_long() && eval_group(&self.exit, ctx) {
                return Signal::exit(ts, price).tag("exit");
            }
            if ctx.is_short()
                && let Some(ref sx) = self.short_exit
                && eval_group(sx, ctx)
            {
                return Signal::exit(ts, price).tag("short_exit");
            }

            // Priority 2: partial exit (scale-out).
            if !self.scale_out.conditions.is_empty() && eval_group(&self.scale_out, ctx) {
                return Signal::scale_out(self.scale_out_fraction, ts, price).tag("scale_out");
            }

            // Priority 3: add to position (scale-in).
            if !self.scale_in.conditions.is_empty() && eval_group(&self.scale_in, ctx) {
                return Signal::scale_in(self.scale_in_fraction, ts, price).tag("scale_in");
            }
        } else if regime_ok {
            if eval_group(&self.entry, ctx) {
                let offset = self.entry_price_offset_pct;
                let mut sig = match self.entry_order_type {
                    LongOrderType::Market => Signal::long(ts, price),
                    LongOrderType::LimitBelow => {
                        Signal::buy_limit(ts, price, price * (1.0 - offset))
                    }
                    LongOrderType::StopAbove => Signal::buy_stop(ts, price, price * (1.0 + offset)),
                    LongOrderType::StopLimitAbove => {
                        let stop_price = price * (1.0 + offset);
                        let limit_price = stop_price * (1.0 + self.entry_stop_limit_gap_pct);
                        Signal::buy_stop_limit(ts, price, stop_price, limit_price)
                    }
                }
                .tag("entry");
                if let Some(n) = self.entry_expires_bars {
                    sig = sig.expires_in_bars(n);
                }
                if let Some(sl) = self.entry_bracket_sl {
                    sig = sig.stop_loss(sl);
                }
                if let Some(tp) = self.entry_bracket_tp {
                    sig = sig.take_profit(tp);
                }
                if let Some(tr) = self.entry_bracket_trail {
                    sig = sig.trailing_stop(tr);
                }
                return sig;
            }
            if let Some(ref se) = self.short_entry
                && eval_group(se, ctx)
            {
                let offset = self.short_price_offset_pct;
                let mut sig = match self.short_order_type {
                    ShortOrderType::Market => Signal::short(ts, price),
                    ShortOrderType::LimitAbove => {
                        Signal::sell_limit(ts, price, price * (1.0 + offset))
                    }
                    ShortOrderType::StopBelow => {
                        Signal::sell_stop(ts, price, price * (1.0 - offset))
                    }
                }
                .tag("short_entry");
                if let Some(n) = self.short_expires_bars {
                    sig = sig.expires_in_bars(n);
                }
                if let Some(sl) = self.short_bracket_sl {
                    sig = sig.stop_loss(sl);
                }
                if let Some(tp) = self.short_bracket_tp {
                    sig = sig.take_profit(tp);
                }
                if let Some(tr) = self.short_bracket_trail {
                    sig = sig.trailing_stop(tr);
                }
                return sig;
            }
        }

        Signal::hold()
    }
}

// ─── Condition evaluation ────────────────────────────────────────────────────

fn eval_group(group: &ConditionGroup, ctx: &StrategyContext) -> bool {
    if group.conditions.is_empty() {
        return false;
    }

    // Evaluate with proper operator precedence: AND binds tighter than OR.
    // Split conditions into OR-separated groups of AND-chains, then any
    // AND-chain being fully true makes the whole group true.
    //
    // Example: A AND B OR C AND D  =>  (A AND B) OR (C AND D)
    let results: Vec<bool> = group
        .conditions
        .iter()
        .map(|c| eval_condition(c, ctx))
        .collect();

    // Walk through combining AND-chains; short-circuit on OR boundaries.
    // We pair each result[i] (i >= 1) with conditions[i - 1].next_op — the
    // *preceding* condition's operator that joins result[i-1] to result[i].
    let mut and_accum = results[0];
    for (prev_cond, &result) in group.conditions.iter().zip(results.iter().skip(1)) {
        match prev_cond.next_op {
            LogicalOp::And => {
                and_accum = and_accum && result;
            }
            LogicalOp::Or => {
                if and_accum {
                    return true;
                }
                and_accum = result;
            }
        }
    }

    and_accum
}

fn eval_condition(cond: &BuiltCondition, ctx: &StrategyContext) -> bool {
    let Some(current) = indicator_value(&cond.indicator, cond.htf_interval, ctx) else {
        return false;
    };

    match &cond.target {
        CompareTarget::Value(threshold) => {
            eval_cmp_scalar(cond.comparison, current, *threshold, || {
                indicator_prev(&cond.indicator, cond.htf_interval, ctx)
            })
        }
        CompareTarget::Range(low, high) => {
            matches!(cond.comparison, ComparisonType::Between)
                && current >= *low
                && current <= *high
        }
        CompareTarget::Indicator(other) => {
            let Some(other_val) = indicator_value(other, cond.htf_interval, ctx) else {
                return false;
            };
            eval_cmp_ref(
                cond.comparison,
                current,
                other_val,
                || indicator_prev(&cond.indicator, cond.htf_interval, ctx),
                || indicator_prev(other, cond.htf_interval, ctx),
            )
        }
    }
}

fn eval_cmp_scalar(
    comp: ComparisonType,
    current: f64,
    threshold: f64,
    prev: impl Fn() -> Option<f64>,
) -> bool {
    match comp {
        ComparisonType::Above => current > threshold,
        ComparisonType::Below => current < threshold,
        ComparisonType::CrossesAbove => {
            prev().is_some_and(|p| p <= threshold && current > threshold)
        }
        ComparisonType::CrossesBelow => {
            prev().is_some_and(|p| p >= threshold && current < threshold)
        }
        ComparisonType::Between => false, // handled by Range branch above
        ComparisonType::Equals => {
            (current - threshold).abs() / threshold.abs().max(1.0) < FLOAT_EPSILON
        }
    }
}

fn eval_cmp_ref(
    comp: ComparisonType,
    current: f64,
    other: f64,
    prev_self: impl Fn() -> Option<f64>,
    prev_other: impl Fn() -> Option<f64>,
) -> bool {
    match comp {
        ComparisonType::Above => current > other,
        ComparisonType::Below => current < other,
        ComparisonType::CrossesAbove => match (prev_self(), prev_other()) {
            (Some(ps), Some(po)) => ps <= po && current > other,
            _ => false,
        },
        ComparisonType::CrossesBelow => match (prev_self(), prev_other()) {
            (Some(ps), Some(po)) => ps >= po && current < other,
            _ => false,
        },
        ComparisonType::Between => false, // Between requires a range, not a single indicator
        ComparisonType::Equals => (current - other).abs() / other.abs().max(1.0) < FLOAT_EPSILON,
    }
}

// ─── Indicator key / value helpers ──────────────────────────────────────────

/// Try to resolve a price-action code directly from the candle context.
/// Returns `None` for non-price-action codes (they use the indicator HashMap path).
fn price_action_value(code: &str, ctx: &StrategyContext) -> Option<f64> {
    match code {
        "close" => Some(ctx.close()),
        "open" => Some(ctx.open()),
        "high" => Some(ctx.high()),
        "low" => Some(ctx.low()),
        "volume" => Some(ctx.volume() as f64),
        "typical_price" => {
            let c = ctx.current_candle();
            Some((c.high + c.low + c.close) / 3.0)
        }
        "median_price" => {
            let c = ctx.current_candle();
            Some((c.high + c.low) / 2.0)
        }
        "candle_range" => {
            let c = ctx.current_candle();
            Some(c.high - c.low)
        }
        "candle_body" => {
            let c = ctx.current_candle();
            Some((c.close - c.open).abs())
        }
        "is_bullish" => Some(if ctx.close() > ctx.open() { 1.0 } else { 0.0 }),
        "is_bearish" => Some(if ctx.close() < ctx.open() { 1.0 } else { 0.0 }),
        "price_change_pct" => ctx.previous_candle().map(|prev| {
            if prev.close != 0.0 {
                ((ctx.close() - prev.close) / prev.close) * 100.0
            } else {
                0.0
            }
        }),
        "gap_pct" => ctx.previous_candle().map(|prev| {
            if prev.close != 0.0 {
                ((ctx.open() - prev.close) / prev.close) * 100.0
            } else {
                0.0
            }
        }),
        _ => None,
    }
}

/// Previous-bar value for price-action codes.
fn price_action_prev(code: &str, ctx: &StrategyContext) -> Option<f64> {
    let prev = ctx.previous_candle()?;
    match code {
        "close" => Some(prev.close),
        "open" => Some(prev.open),
        "high" => Some(prev.high),
        "low" => Some(prev.low),
        "volume" => Some(prev.volume as f64),
        "typical_price" => Some((prev.high + prev.low + prev.close) / 3.0),
        "median_price" => Some((prev.high + prev.low) / 2.0),
        "candle_range" => Some(prev.high - prev.low),
        "candle_body" => Some((prev.close - prev.open).abs()),
        "is_bullish" => Some(if prev.close > prev.open { 1.0 } else { 0.0 }),
        "is_bearish" => Some(if prev.close < prev.open { 1.0 } else { 0.0 }),
        "price_change_pct" => {
            // Requires the candle two bars ago
            let idx = ctx.index;
            if idx >= 2 {
                let two_prev = &ctx.candles[idx - 2];
                Some(if two_prev.close != 0.0 {
                    ((prev.close - two_prev.close) / two_prev.close) * 100.0
                } else {
                    0.0
                })
            } else {
                None
            }
        }
        "gap_pct" => {
            let idx = ctx.index;
            if idx >= 2 {
                let two_prev = &ctx.candles[idx - 2];
                Some(if two_prev.close != 0.0 {
                    ((prev.open - two_prev.close) / two_prev.close) * 100.0
                } else {
                    0.0
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn indicator_value(
    ind: &BuiltIndicator,
    htf_interval: Option<Interval>,
    ctx: &StrategyContext,
) -> Option<f64> {
    if let Some(interval) = htf_interval {
        ctx.indicator(&htf_indicator_key(interval, ind))
            .or_else(|| price_action_value(ind.indicator.code, ctx))
    } else {
        price_action_value(ind.indicator.code, ctx).or_else(|| ctx.indicator(&indicator_key(ind)))
    }
}

fn indicator_prev(
    ind: &BuiltIndicator,
    htf_interval: Option<Interval>,
    ctx: &StrategyContext,
) -> Option<f64> {
    if let Some(interval) = htf_interval {
        ctx.indicator_prev(&htf_indicator_key(interval, ind))
            .or_else(|| price_action_prev(ind.indicator.code, ctx))
    } else {
        price_action_prev(ind.indicator.code, ctx)
            .or_else(|| ctx.indicator_prev(&indicator_key(ind)))
    }
}

fn htf_indicator_key(interval: Interval, ind: &BuiltIndicator) -> String {
    format!("htf_{}_{}", interval.as_str(), indicator_key(ind))
}

/// Compute the context key string for a `BuiltIndicator`.
///
/// These key formats must exactly match what the library's `IndicatorRef`
/// implementations produce (see `src/backtesting/refs/computed.rs` and engine.rs).
fn indicator_key(ind: &BuiltIndicator) -> String {
    let p = |i: usize| ind.param_values.get(i).copied().unwrap_or(0.0) as usize;
    let pf = |i: usize| ind.param_values.get(i).copied().unwrap_or(0.0);
    let out = ind.output.as_deref().unwrap_or("");

    match ind.indicator.code {
        // Moving averages — single-value
        "sma" => format!("sma_{}", p(0)),
        "ema" => format!("ema_{}", p(0)),
        "wma" => format!("wma_{}", p(0)),
        "dema" => format!("dema_{}", p(0)),
        "tema" => format!("tema_{}", p(0)),
        "hma" => format!("hma_{}", p(0)),
        "vwma" => format!("vwma_{}", p(0)),
        "mcginley" => format!("mcginley_{}", p(0)),
        "alma" => format!("alma_{}_{}_{}", p(0), pf(1), pf(2)),
        // Momentum — single-value
        "rsi" => format!("rsi_{}", p(0)),
        "cci" => format!("cci_{}", p(0)),
        "williams_r" => format!("williams_r_{}", p(0)),
        "cmo" => format!("cmo_{}", p(0)),
        "momentum" => format!("momentum_{}", p(0)),
        "roc" => format!("roc_{}", p(0)),
        // Trend — single-value
        "adx" => format!("adx_{}", p(0)),
        // Volatility — single-value
        "atr" => format!("atr_{}", p(0)),
        "true_range" => "true_range".to_string(),
        // Volume — single-value
        "mfi" => format!("mfi_{}", p(0)),
        "cmf" => format!("cmf_{}", p(0)),
        "obv" => "obv".to_string(),
        "vwap" => "vwap".to_string(),
        "chaikin_osc" => "chaikin_osc".to_string(),
        // Multi-output: MACD (default = MACD line)
        "macd" => match out {
            "signal" => format!("macd_signal_{}_{}_{}", p(0), p(1), p(2)),
            "histogram" => format!("macd_histogram_{}_{}_{}", p(0), p(1), p(2)),
            _ => format!("macd_line_{}_{}_{}", p(0), p(1), p(2)),
        },
        // Multi-output: Bollinger Bands (default = middle band)
        "bollinger" => match out {
            "upper" => format!("bollinger_upper_{}_{}", p(0), pf(1)),
            "lower" => format!("bollinger_lower_{}_{}", p(0), pf(1)),
            _ => format!("bollinger_middle_{}_{}", p(0), pf(1)),
        },
        // Multi-output: Donchian Channels (default = middle)
        "donchian" => match out {
            "upper" => format!("donchian_upper_{}", p(0)),
            "lower" => format!("donchian_lower_{}", p(0)),
            _ => format!("donchian_middle_{}", p(0)),
        },
        // SuperTrend — single value track (trend value line)
        "supertrend" => format!("supertrend_value_{}_{}", p(0), pf(1)),
        // Stochastic (default = %K)
        "stochastic" => match out {
            "d" => format!("stochastic_d_{}_{}_{}", p(0), p(1), p(2)),
            _ => format!("stochastic_k_{}_{}_{}", p(0), p(1), p(2)),
        },
        // Stochastic RSI (default = %K)
        "stochastic_rsi" => match out {
            "d" => format!("stoch_rsi_d_{}_{}_{}_{}", p(0), p(1), p(2), p(3)),
            _ => format!("stoch_rsi_k_{}_{}_{}_{}", p(0), p(1), p(2), p(3)),
        },
        // Aroon (default = up line)
        "aroon" => match out {
            "down" => format!("aroon_down_{}", p(0)),
            _ => format!("aroon_up_{}", p(0)),
        },
        // Oscillators — single-value
        "awesome_oscillator" => format!("ao_{}_{}", p(0), p(1)),
        "coppock_curve" => format!("coppock_{}_{}_{}", p(0), p(1), p(2)),
        // Trend — single-value
        "parabolic_sar" => format!("psar_{}_{}", pf(0), pf(1)),
        // Multi-output: Ichimoku Cloud (default = conversion line)
        "ichimoku" => {
            let suffix = format!("{}_{}_{}_{}", p(0), p(1), p(2), p(3));
            match out {
                "base" => format!("ichimoku_base_{}", suffix),
                "leading_a" => format!("ichimoku_leading_a_{}", suffix),
                "leading_b" => format!("ichimoku_leading_b_{}", suffix),
                "lagging" => format!("ichimoku_lagging_{}", suffix),
                _ => format!("ichimoku_conversion_{}", suffix),
            }
        }
        // Volatility — single-value
        "choppiness_index" => format!("chop_{}", p(0)),
        // Multi-output: Keltner Channels (default = middle)
        "keltner" => match out {
            "upper" => format!("keltner_upper_{}_{}_{}", p(0), pf(1), p(2)),
            "lower" => format!("keltner_lower_{}_{}_{}", p(0), pf(1), p(2)),
            _ => format!("keltner_middle_{}_{}_{}", p(0), pf(1), p(2)),
        },
        // Volume — single-value
        "accumulation_distribution" => "ad".to_string(),
        "balance_of_power" => {
            let period = p(0);
            if period > 0 {
                format!("bop_{}", period)
            } else {
                "bop".to_string()
            }
        }
        // Momentum — single-value (BullBearPower computes both; use code to select line)
        "bull_power" => format!("bull_power_{}", p(0)),
        "bear_power" => format!("bear_power_{}", p(0)),
        "elder_bull" => format!("elder_bull_{}", p(0)),
        "elder_bear" => format!("elder_bear_{}", p(0)),
        // Fallback: best-effort (also handles relative_volume_{n})
        other => format!("{}_{}", other, p(0)),
    }
}

/// Map a `BuiltIndicator` to the `(key, Indicator)` pair needed for
/// `required_indicators()`. Returns `None` for price-action codes (no pre-computation
/// needed) and for unrecognised codes.
fn indicator_to_lib(ind: &BuiltIndicator) -> Option<(String, Indicator)> {
    let p = |i: usize| ind.param_values.get(i).copied().unwrap_or(0.0) as usize;
    let pf = |i: usize| ind.param_values.get(i).copied().unwrap_or(0.0);
    let key = indicator_key(ind);

    let lib_ind = match ind.indicator.code {
        // Price action — computed directly from ctx, no pre-computation needed
        "close" | "open" | "high" | "low" | "volume" | "typical_price" | "median_price"
        | "candle_range" | "candle_body" | "is_bullish" | "is_bearish" | "price_change_pct"
        | "gap_pct" => return None,
        // Moving averages
        "sma" => Indicator::Sma(p(0)),
        "ema" => Indicator::Ema(p(0)),
        "wma" => Indicator::Wma(p(0)),
        "dema" => Indicator::Dema(p(0)),
        "tema" => Indicator::Tema(p(0)),
        "hma" => Indicator::Hma(p(0)),
        "vwma" => Indicator::Vwma(p(0)),
        "mcginley" => Indicator::McginleyDynamic(p(0)),
        "alma" => Indicator::Alma {
            period: p(0),
            offset: pf(1),
            sigma: pf(2),
        },
        // Momentum oscillators
        "rsi" => Indicator::Rsi(p(0)),
        "cci" => Indicator::Cci(p(0)),
        "williams_r" => Indicator::WilliamsR(p(0)),
        "cmo" => Indicator::Cmo(p(0)),
        "momentum" => Indicator::Momentum(p(0)),
        "roc" => Indicator::Roc(p(0)),
        "awesome_oscillator" => Indicator::AwesomeOscillator {
            fast: p(0),
            slow: p(1),
        },
        "coppock_curve" => Indicator::CoppockCurve {
            wma_period: p(0),
            long_roc: p(1),
            short_roc: p(2),
        },
        // Trend
        "adx" => Indicator::Adx(p(0)),
        "aroon" => Indicator::Aroon(p(0)),
        "macd" => Indicator::Macd {
            fast: p(0),
            slow: p(1),
            signal: p(2),
        },
        "supertrend" => Indicator::Supertrend {
            period: p(0),
            multiplier: pf(1),
        },
        "parabolic_sar" => Indicator::ParabolicSar {
            step: pf(0),
            max: pf(1),
        },
        "ichimoku" => Indicator::Ichimoku {
            conversion: p(0),
            base: p(1),
            lagging: p(2),
            displacement: p(3),
        },
        // Volatility
        "atr" => Indicator::Atr(p(0)),
        "true_range" => Indicator::TrueRange,
        "bollinger" => Indicator::Bollinger {
            period: p(0),
            std_dev: pf(1),
        },
        "donchian" => Indicator::DonchianChannels(p(0)),
        "keltner" => Indicator::KeltnerChannels {
            period: p(0),
            multiplier: pf(1),
            atr_period: p(2),
        },
        "choppiness_index" => Indicator::ChoppinessIndex(p(0)),
        // Volume
        "mfi" => Indicator::Mfi(p(0)),
        "cmf" => Indicator::Cmf(p(0)),
        "obv" => Indicator::Obv,
        "vwap" => Indicator::Vwap,
        "chaikin_osc" => Indicator::ChaikinOscillator,
        "accumulation_distribution" => Indicator::AccumulationDistribution,
        "balance_of_power" => {
            let period = p(0);
            Indicator::BalanceOfPower(if period > 0 { Some(period) } else { None })
        }
        // Stochastic
        "stochastic" => Indicator::Stochastic {
            k_period: p(0),
            k_slow: p(1),
            d_period: p(2),
        },
        "stochastic_rsi" => Indicator::StochasticRsi {
            rsi_period: p(0),
            stoch_period: p(1),
            k_period: p(2),
            d_period: p(3),
        },
        // Power indicators — BullBearPower computes both bull and bear lines
        "bull_power" | "bear_power" => Indicator::BullBearPower(p(0)),
        // Elder Ray — same computation, different storage keys
        "elder_bull" | "elder_bear" => Indicator::ElderRay(p(0)),
        _ => return None,
    };

    Some((key, lib_ind))
}

fn collect_indicator(
    ind: &BuiltIndicator,
    out: &mut Vec<(String, Indicator)>,
    seen: &mut HashSet<String>,
) {
    if let Some((key, lib_ind)) = indicator_to_lib(ind)
        && seen.insert(key.clone())
    {
        out.push((key, lib_ind));
    }
}

fn collect_htf_indicator(
    interval: Interval,
    ind: &BuiltIndicator,
    out: &mut Vec<HtfIndicatorSpec>,
    seen: &mut HashSet<String>,
) {
    if let Some((base_key, indicator)) = indicator_to_lib(ind) {
        let htf_key = format!("htf_{}_{}", interval.as_str(), base_key);
        if seen.insert(htf_key.clone()) {
            out.push(HtfIndicatorSpec {
                interval,
                htf_key,
                base_key,
                indicator,
                utc_offset_secs: 0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use finance_query::Interval;
    use finance_query::backtesting::Strategy;

    use crate::backtest::indicators::IndicatorDef;

    use super::{
        BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup,
        DynamicStrategy, LogicalOp,
    };

    #[test]
    fn htf_requirements_deduplicate_by_scoped_key() {
        let rsi = BuiltIndicator {
            indicator: IndicatorDef::find("rsi"),
            param_values: vec![14.0],
            output: None,
        };
        let cond = BuiltCondition {
            indicator: rsi.clone(),
            comparison: ComparisonType::Above,
            target: CompareTarget::Value(50.0),
            htf_interval: Some(Interval::OneWeek),
            next_op: LogicalOp::And,
        };

        let strategy = DynamicStrategy {
            entry: ConditionGroup {
                conditions: vec![cond.clone(), cond],
            },
            ..DynamicStrategy::new(
                "HTF Test".to_string(),
                ConditionGroup::default(),
                ConditionGroup::default(),
            )
        };

        let reqs = strategy.htf_requirements();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].htf_key, "htf_1wk_rsi_14");
    }

    #[test]
    fn scoped_conditions_only_emit_htf_requirements() {
        let cond = BuiltCondition {
            indicator: BuiltIndicator {
                indicator: IndicatorDef::find("rsi"),
                param_values: vec![14.0],
                output: None,
            },
            comparison: ComparisonType::Above,
            target: CompareTarget::Value(50.0),
            htf_interval: Some(Interval::OneWeek),
            next_op: LogicalOp::And,
        };

        let strategy = DynamicStrategy::new(
            "Scoped RSI".to_string(),
            ConditionGroup {
                conditions: vec![cond],
            },
            ConditionGroup::default(),
        );

        let base_requirements = strategy.required_indicators();
        let htf_requirements = strategy.htf_requirements();

        assert!(base_requirements.is_empty());
        assert_eq!(htf_requirements.len(), 1);
        assert_eq!(htf_requirements[0].base_key, "rsi_14");
        assert_eq!(htf_requirements[0].htf_key, "htf_1wk_rsi_14");
    }
}
