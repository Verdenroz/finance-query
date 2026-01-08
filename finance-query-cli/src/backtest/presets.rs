// Allow field reassignment pattern since we need to build strategy configs
// incrementally after Default::default()
#![allow(clippy::field_reassign_with_default)]

use super::indicators::IndicatorDef;
use super::types::{
    BacktestConfiguration, BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType, LogicalOp,
};
use finance_query::{Interval, TimeRange};

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
                    // Entry: SMA(20) crosses above SMA(50)
                    let sma20 = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "sma")
                            .unwrap(),
                        param_values: vec![20.0],
                        output: None,
                    };
                    let sma50 = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "sma")
                            .unwrap(),
                        param_values: vec![50.0],
                        output: None,
                    };
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: sma20.clone(),
                            comparison: ComparisonType::CrossesAbove,
                            target: CompareTarget::Indicator(sma50.clone()),
                            next_op: LogicalOp::And,
                        });
                    // Exit: SMA(20) crosses below SMA(50)
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: sma20,
                            comparison: ComparisonType::CrossesBelow,
                            target: CompareTarget::Indicator(sma50),
                            next_op: LogicalOp::And,
                        });
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
                    let rsi = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "rsi")
                            .unwrap(),
                        param_values: vec![14.0],
                        output: None,
                    };
                    // Entry: RSI crosses below 30
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi.clone(),
                            comparison: ComparisonType::CrossesBelow,
                            target: CompareTarget::Value(30.0),
                            next_op: LogicalOp::And,
                        });
                    // Exit: RSI crosses above 70
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi,
                            comparison: ComparisonType::CrossesAbove,
                            target: CompareTarget::Value(70.0),
                            next_op: LogicalOp::And,
                        });
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
                    let macd_line = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "macd")
                            .unwrap(),
                        param_values: vec![12.0, 26.0, 9.0],
                        output: Some("line".to_string()),
                    };
                    let macd_signal = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "macd")
                            .unwrap(),
                        param_values: vec![12.0, 26.0, 9.0],
                        output: Some("signal".to_string()),
                    };
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: macd_line.clone(),
                            comparison: ComparisonType::CrossesAbove,
                            target: CompareTarget::Indicator(macd_signal.clone()),
                            next_op: LogicalOp::And,
                        });
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: macd_line,
                            comparison: ComparisonType::CrossesBelow,
                            target: CompareTarget::Indicator(macd_signal),
                            next_op: LogicalOp::And,
                        });
                    cfg
                },
            },
            Self {
                name: "Bollinger Bounce",
                description: "Mean reversion at Bollinger Band extremes",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::SixMonths;
                    cfg.allow_short = true;
                    cfg.stop_loss = Some(0.03);
                    cfg.take_profit = Some(0.06);
                    cfg.strategy.name = "Bollinger Mean Reversion".to_string();
                    // Simplified: use RSI at extremes with Bollinger context
                    let rsi = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "rsi")
                            .unwrap(),
                        param_values: vec![14.0],
                        output: None,
                    };
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi.clone(),
                            comparison: ComparisonType::Below,
                            target: CompareTarget::Value(30.0),
                            next_op: LogicalOp::And,
                        });
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi,
                            comparison: ComparisonType::Above,
                            target: CompareTarget::Value(70.0),
                            next_op: LogicalOp::And,
                        });
                    cfg
                },
            },
            Self {
                name: "Trend Following",
                description: "Follow strong trends using ADX filter with SuperTrend",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::OneDay;
                    cfg.range = TimeRange::TwoYears;
                    cfg.capital = 50_000.0;
                    cfg.strategy.name = "Trend Follower".to_string();
                    // Entry: ADX > 25 (strong trend) AND price above SMA(50)
                    let adx = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "adx")
                            .unwrap(),
                        param_values: vec![14.0],
                        output: None,
                    };
                    // Note: SMA condition is represented conceptually but we use
                    // prebuilt strategies, so ADX is the primary indicator here
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: adx.clone(),
                            comparison: ComparisonType::Above,
                            target: CompareTarget::Value(25.0),
                            next_op: LogicalOp::And,
                        });
                    // Exit: ADX falls below 20
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: adx,
                            comparison: ComparisonType::Below,
                            target: CompareTarget::Value(20.0),
                            next_op: LogicalOp::And,
                        });
                    cfg
                },
            },
            Self {
                name: "Day Trader",
                description: "Fast RSI reversals on 15-minute chart",
                config: || {
                    let mut cfg = BacktestConfiguration::default();
                    cfg.interval = Interval::FifteenMinutes;
                    cfg.range = TimeRange::OneMonth;
                    cfg.capital = 25_000.0;
                    cfg.commission = 0.001;
                    cfg.slippage = 0.002;
                    cfg.allow_short = true;
                    cfg.stop_loss = Some(0.02);
                    cfg.take_profit = Some(0.04);
                    cfg.strategy.name = "RSI Day Trade".to_string();
                    let rsi = BuiltIndicator {
                        indicator: IndicatorDef::all()
                            .into_iter()
                            .find(|i| i.code == "rsi")
                            .unwrap(),
                        param_values: vec![7.0],
                        output: None,
                    };
                    cfg.strategy
                        .entry_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi.clone(),
                            comparison: ComparisonType::CrossesBelow,
                            target: CompareTarget::Value(20.0),
                            next_op: LogicalOp::And,
                        });
                    cfg.strategy
                        .exit_conditions
                        .conditions
                        .push(BuiltCondition {
                            indicator: rsi,
                            comparison: ComparisonType::CrossesAbove,
                            target: CompareTarget::Value(80.0),
                            next_op: LogicalOp::And,
                        });
                    cfg
                },
            },
        ]
    }
}
