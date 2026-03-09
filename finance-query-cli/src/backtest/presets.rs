// Allow field reassignment pattern since we need to build strategy configs
// incrementally after Default::default()
#![allow(clippy::field_reassign_with_default)]

use super::indicators::IndicatorDef;
use super::types::{
    BacktestConfiguration, BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType,
    ConditionGroup, LogicalOp, bars_per_year_for_interval,
};
use finance_query::{Interval, TimeRange};

// ── Standard indicator thresholds used across presets ───────────────────────
const RSI_OVERSOLD: f64 = 30.0;
const RSI_OVERBOUGHT: f64 = 70.0;
const ADX_STRONG_TREND: f64 = 25.0;
const ADX_WEAK_TREND: f64 = 20.0;
const STOCH_OVERBOUGHT: f64 = 80.0;
const STOCH_OVERSOLD: f64 = 20.0;
/// Standard MACD parameters: [fast, slow, signal].
const MACD_PARAMS: [f64; 3] = [12.0, 26.0, 9.0];

// ── Helper ───────────────────────────────────────────────────────────────────

fn ind(code: &str, params: Vec<f64>) -> BuiltIndicator {
    BuiltIndicator {
        indicator: IndicatorDef::find(code),
        param_values: params,
        output: None,
    }
}

fn ind_out(code: &str, params: Vec<f64>, output: &str) -> BuiltIndicator {
    BuiltIndicator {
        indicator: IndicatorDef::find(code),
        param_values: params,
        output: Some(output.to_string()),
    }
}

fn entry(
    indicator: BuiltIndicator,
    comparison: ComparisonType,
    target: CompareTarget,
) -> BuiltCondition {
    BuiltCondition {
        indicator,
        comparison,
        target,
        htf_interval: None,
        next_op: LogicalOp::And,
    }
}

// ── Preset definition ────────────────────────────────────────────────────────

/// A preset strategy configuration
#[derive(Debug, Clone)]
pub struct StrategyPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub config: fn() -> BacktestConfiguration,
}

impl StrategyPreset {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                name: "Swing Trader",
                description: "SMA crossover for medium-term positions (days to weeks)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::OneYear;
                    cfg.stop_loss = Some(0.05);
                    cfg.take_profit = Some(0.10);
                    cfg.strategy.name = "SMA Crossover".to_string();

                    let sma20 = ind("sma", vec![20.0]);
                    let sma50 = ind("sma", vec![50.0]);

                    // Entry: SMA(20) crosses above SMA(50)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        sma20.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(sma50.clone()),
                    ));
                    // Exit: SMA(20) crosses below SMA(50)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        sma20,
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(sma50),
                    ));
                    cfg
                },
            },
            Self {
                name: "RSI Mean Reversion",
                description: "Buy oversold, sell overbought using RSI",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::OneYear;
                    cfg.stop_loss = Some(0.03);
                    cfg.take_profit = Some(0.06);
                    cfg.strategy.name = "RSI Reversal".to_string();

                    let rsi = ind("rsi", vec![14.0]);

                    // Entry: RSI crosses above oversold threshold (reversal confirmation)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        rsi.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Value(RSI_OVERSOLD),
                    ));
                    // Exit: RSI crosses above overbought threshold
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        rsi,
                        ComparisonType::CrossesAbove,
                        CompareTarget::Value(RSI_OVERBOUGHT),
                    ));
                    cfg
                },
            },
            Self {
                name: "MACD Momentum",
                description: "Trade MACD line crossovers with signal line",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::TwoYears;
                    cfg.strategy.name = "MACD Signal".to_string();

                    let macd_line = ind_out("macd", MACD_PARAMS.to_vec(), "line");
                    let macd_signal = ind_out("macd", MACD_PARAMS.to_vec(), "signal");

                    cfg.strategy.entry_conditions.conditions.push(entry(
                        macd_line.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(macd_signal.clone()),
                    ));
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        macd_line,
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(macd_signal),
                    ));
                    cfg
                },
            },
            Self {
                name: "Bollinger Breakout",
                description: "Enter on volatility breakout above/below Bollinger Bands, exit at midline",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::TwoYears;
                    cfg.allow_short = true;
                    cfg.stop_loss = Some(0.05);
                    cfg.take_profit = Some(0.15);
                    cfg.strategy.name = "Bollinger Breakout".to_string();

                    let close = ind("close", vec![]);
                    let bb_upper = ind_out("bollinger", vec![20.0, 2.0], "upper");
                    let bb_middle = ind("bollinger", vec![20.0, 2.0]); // default = middle
                    let bb_lower = ind_out("bollinger", vec![20.0, 2.0], "lower");

                    // Long entry: Close crosses above upper band (upside breakout)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(bb_upper),
                    ));
                    // Long exit: Close crosses below middle band (momentum fades)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(bb_middle.clone()),
                    ));

                    // Short entry: Close crosses below lower band (downside breakout)
                    let mut short_entry = ConditionGroup::new();
                    short_entry.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(bb_lower),
                    ));
                    cfg.strategy.short_entry_conditions = Some(short_entry);

                    // Short exit: Close crosses above middle band
                    let mut short_exit = ConditionGroup::new();
                    short_exit.conditions.push(entry(
                        close,
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(bb_middle),
                    ));
                    cfg.strategy.short_exit_conditions = Some(short_exit);

                    cfg
                },
            },
            Self {
                name: "Trend Following",
                description: "Follow strong trends using ADX strength filter (ADX > 25 entry, < 20 exit)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::TwoYears;
                    cfg.capital = 50_000.0;
                    cfg.strategy.name = "Trend Follower".to_string();

                    let adx = ind("adx", vec![14.0]);

                    // Entry: ADX > 25 (strong trend confirmed)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        adx.clone(),
                        ComparisonType::Above,
                        CompareTarget::Value(ADX_STRONG_TREND),
                    ));
                    // Exit: ADX falls below weak-trend threshold (trend fading)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        adx,
                        ComparisonType::Below,
                        CompareTarget::Value(ADX_WEAK_TREND),
                    ));
                    cfg
                },
            },
            Self {
                name: "Ichimoku TK Cross",
                description: "Tenkan/Kijun crossover — the classic Ichimoku trend signal (daily, 2yr)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::TwoYears;
                    cfg.stop_loss = Some(0.04);
                    cfg.take_profit = Some(0.12);
                    cfg.strategy.name = "Ichimoku TK Cross".to_string();

                    // Standard Ichimoku params: tenkan=9, kijun=26, lagging=26, displacement=26
                    let ichimoku_params = vec![9.0, 26.0, 26.0, 26.0];
                    let tenkan = ind("ichimoku", ichimoku_params.clone()); // default output = tenkan-sen
                    let kijun = ind_out("ichimoku", ichimoku_params, "base"); // "base" = kijun-sen

                    // Entry: Tenkan-sen crosses above Kijun-sen (bullish TK cross)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        tenkan.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(kijun.clone()),
                    ));
                    // Exit: Tenkan-sen crosses below Kijun-sen (bearish TK cross)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        tenkan,
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(kijun),
                    ));
                    cfg
                },
            },
            Self {
                name: "Volume Flow",
                description: "MFI oversold bounce above VWAP — volume-confirmed mean reversion (daily, 1yr)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::OneYear;
                    cfg.stop_loss = Some(0.04);
                    cfg.strategy.name = "MFI + VWAP".to_string();

                    let mfi = ind("mfi", vec![14.0]);
                    let close = ind("close", vec![]);
                    let vwap = ind("vwap", vec![]);

                    // Entry: MFI crosses above 20 (money flowing back in) AND close above VWAP
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        mfi.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Value(20.0),
                    ));
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        close,
                        ComparisonType::Above,
                        CompareTarget::Indicator(vwap),
                    ));
                    // Exit: MFI crosses above 80 (overbought — take profit)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        mfi,
                        ComparisonType::CrossesAbove,
                        CompareTarget::Value(80.0),
                    ));
                    cfg
                },
            },
            Self {
                name: "Keltner Scalper",
                description: "Keltner channel breakout with short side on 1-hour bars (1h, 3mo)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneHour;
                    cfg.bars_per_year = bars_per_year_for_interval(cfg.interval);
                    cfg.range = TimeRange::ThreeMonths;
                    cfg.capital = 25_000.0;
                    cfg.slippage = 0.001;
                    cfg.allow_short = true;
                    cfg.stop_loss = Some(0.02);
                    cfg.take_profit = Some(0.06);
                    cfg.strategy.name = "Keltner Breakout".to_string();

                    // period=20, multiplier=2.0, atr_period=10
                    let kc_params = vec![20.0, 2.0, 10.0];
                    let close = ind("close", vec![]);
                    let kc_upper = ind_out("keltner", kc_params.clone(), "upper");
                    let kc_middle = ind("keltner", kc_params.clone()); // default = middle (EMA)
                    let kc_lower = ind_out("keltner", kc_params, "lower");

                    // Long entry: close crosses above upper band (upside momentum breakout)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(kc_upper),
                    ));
                    // Long exit: close crosses below middle band (momentum stalls)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(kc_middle.clone()),
                    ));

                    // Short entry: close crosses below lower band (downside momentum breakout)
                    let mut short_entry = ConditionGroup::new();
                    short_entry.conditions.push(entry(
                        close.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(kc_lower),
                    ));
                    cfg.strategy.short_entry_conditions = Some(short_entry);

                    // Short exit: close crosses above middle band
                    let mut short_exit = ConditionGroup::new();
                    short_exit.conditions.push(entry(
                        close,
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(kc_middle),
                    ));
                    cfg.strategy.short_exit_conditions = Some(short_exit);

                    cfg
                },
            },
            Self {
                name: "EMA Momentum",
                description: "EMA(9/21) crossover filtered by Chande Momentum Oscillator > 0 (daily, 1yr)",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::OneYear;
                    cfg.stop_loss = Some(0.04);
                    cfg.strategy.name = "EMA + CMO".to_string();

                    let ema_fast = ind("ema", vec![9.0]);
                    let ema_slow = ind("ema", vec![21.0]);
                    let cmo = ind("cmo", vec![14.0]);

                    // Entry: EMA(9) crosses above EMA(21) AND CMO(14) > 0 (positive momentum)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        ema_fast.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(ema_slow.clone()),
                    ));
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        cmo,
                        ComparisonType::Above,
                        CompareTarget::Value(0.0),
                    ));
                    // Exit: EMA(9) crosses below EMA(21)
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        ema_fast,
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(ema_slow),
                    ));
                    cfg
                },
            },
            Self {
                name: "Day Trader",
                description: "Stochastic %K/%D crossover on 15-minute chart with overbought/oversold filter",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::FifteenMinutes;
                    cfg.bars_per_year = bars_per_year_for_interval(cfg.interval);
                    cfg.range = TimeRange::OneMonth;
                    cfg.capital = 25_000.0;
                    cfg.slippage = 0.002;
                    cfg.allow_short = true;
                    cfg.stop_loss = Some(0.015);
                    cfg.take_profit = Some(0.03);
                    cfg.strategy.name = "Stochastic Crossover".to_string();

                    let stoch_k = ind("stochastic", vec![14.0, 3.0, 3.0]);
                    let stoch_d = ind_out("stochastic", vec![14.0, 3.0, 3.0], "d");

                    // Entry: %K crosses above %D AND %K < 80 (avoid overbought)
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        stoch_k.clone(),
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(stoch_d.clone()),
                    ));
                    cfg.strategy.entry_conditions.conditions.push(entry(
                        stoch_k.clone(),
                        ComparisonType::Below,
                        CompareTarget::Value(STOCH_OVERBOUGHT),
                    ));

                    // Exit: %K crosses below %D
                    cfg.strategy.exit_conditions.conditions.push(entry(
                        stoch_k.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(stoch_d.clone()),
                    ));

                    // Short entry: %K crosses below %D AND %K > 20 (not yet oversold)
                    let mut short_entry = ConditionGroup::new();
                    short_entry.conditions.push(entry(
                        stoch_k.clone(),
                        ComparisonType::CrossesBelow,
                        CompareTarget::Indicator(stoch_d.clone()),
                    ));
                    short_entry.conditions.push(entry(
                        stoch_k.clone(),
                        ComparisonType::Above,
                        CompareTarget::Value(STOCH_OVERSOLD),
                    ));
                    cfg.strategy.short_entry_conditions = Some(short_entry);

                    // Short exit: %K crosses above %D
                    let mut short_exit = ConditionGroup::new();
                    short_exit.conditions.push(entry(
                        stoch_k,
                        ComparisonType::CrossesAbove,
                        CompareTarget::Indicator(stoch_d),
                    ));
                    cfg.strategy.short_exit_conditions = Some(short_exit);

                    cfg
                },
            },
        ]
    }
}
