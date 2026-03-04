use std::collections::HashSet;

use finance_query::backtesting::{Signal, Strategy, StrategyContext};
use finance_query::indicators::Indicator;

use super::types::{
    BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup, LogicalOp,
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
}

impl DynamicStrategy {
    pub fn new(name: String, entry: ConditionGroup, exit: ConditionGroup) -> Self {
        Self {
            name,
            entry,
            exit,
            short_entry: None,
            short_exit: None,
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

        let groups: [Option<&ConditionGroup>; 4] = [
            Some(&self.entry),
            Some(&self.exit),
            self.short_entry.as_ref(),
            self.short_exit.as_ref(),
        ];

        for group in groups.into_iter().flatten() {
            for cond in &group.conditions {
                collect_indicator(&cond.indicator, &mut result, &mut seen);
                if let CompareTarget::Indicator(ref other) = cond.target {
                    collect_indicator(other, &mut result, &mut seen);
                }
            }
        }

        result
    }

    fn on_candle(&self, ctx: &StrategyContext) -> Signal {
        let price = ctx.close();
        let ts = ctx.timestamp();
        let has_pos = ctx.has_position();

        // Exit takes priority over entry
        if has_pos {
            if ctx.is_long() && eval_group(&self.exit, ctx) {
                return Signal::exit(ts, price);
            }
            if ctx.is_short()
                && let Some(ref sx) = self.short_exit
                && eval_group(sx, ctx)
            {
                return Signal::exit(ts, price);
            }
        } else {
            if eval_group(&self.entry, ctx) {
                return Signal::long(ts, price);
            }
            if let Some(ref se) = self.short_entry
                && eval_group(se, ctx)
            {
                return Signal::short(ts, price);
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
    let Some(current) = indicator_value(&cond.indicator, ctx) else {
        return false;
    };

    match &cond.target {
        CompareTarget::Value(threshold) => {
            eval_cmp_scalar(cond.comparison, current, *threshold, || {
                indicator_prev(&cond.indicator, ctx)
            })
        }
        CompareTarget::Range(low, high) => {
            matches!(cond.comparison, ComparisonType::Between)
                && current >= *low
                && current <= *high
        }
        CompareTarget::Indicator(other) => {
            let Some(other_val) = indicator_value(other, ctx) else {
                return false;
            };
            eval_cmp_ref(
                cond.comparison,
                current,
                other_val,
                || indicator_prev(&cond.indicator, ctx),
                || indicator_prev(other, ctx),
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

fn indicator_value(ind: &BuiltIndicator, ctx: &StrategyContext) -> Option<f64> {
    price_action_value(ind.indicator.code, ctx).or_else(|| ctx.indicator(&indicator_key(ind)))
}

fn indicator_prev(ind: &BuiltIndicator, ctx: &StrategyContext) -> Option<f64> {
    price_action_prev(ind.indicator.code, ctx).or_else(|| ctx.indicator_prev(&indicator_key(ind)))
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
