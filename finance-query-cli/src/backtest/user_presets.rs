use super::indicators::IndicatorDef;
use super::state::{parse_interval, parse_range};
use super::types::{
    BacktestConfiguration, BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType,
    ConditionGroup, EnsembleConfig, EnsembleMemberConfig, EnsembleModeChoice, LogicalOp,
    RebalanceModeChoice, StrategyConfig, bars_per_year_for_interval,
};
use crate::error::Result;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Serializable mirror types ─────────────────────────────────────────────────
// These private types mirror the TUI-only runtime types for JSON persistence.
// They use stable string identifiers (indicator codes, comparison names) rather
// than enum discriminants so the format stays readable and survives future
// enum reordering.

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerBuiltIndicator {
    code: String,
    param_values: Vec<f64>,
    output: Option<String>,
}

/// Stores the target of a condition comparison. Tagged with `"type"` so the
/// JSON is self-describing (`{"type":"value","value":30.0}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SerCompareTarget {
    Value { value: f64 },
    Range { low: f64, high: f64 },
    Indicator { indicator: SerBuiltIndicator },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerBuiltCondition {
    indicator: SerBuiltIndicator,
    /// Human-readable name returned by `ComparisonType::name()`, e.g. "Crosses Above".
    comparison: String,
    target: SerCompareTarget,
    #[serde(default)]
    htf_interval: Option<String>,
    /// "and" or "or" (lowercase).
    next_op: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SerConditionGroup {
    conditions: Vec<SerBuiltCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerStrategyConfig {
    name: String,
    entry_conditions: SerConditionGroup,
    exit_conditions: SerConditionGroup,
    short_entry_conditions: Option<SerConditionGroup>,
    short_exit_conditions: Option<SerConditionGroup>,
    #[serde(default)]
    regime_conditions: SerConditionGroup,
    #[serde(default)]
    warmup_bars: usize,
    #[serde(default)]
    scale_in_conditions: SerConditionGroup,
    #[serde(default = "default_scale_in_fraction")]
    scale_in_fraction: f64,
    #[serde(default)]
    scale_out_conditions: SerConditionGroup,
    #[serde(default = "default_scale_out_fraction")]
    scale_out_fraction: f64,
    #[serde(default)]
    entry_order_type: String,
    #[serde(default = "default_price_offset")]
    entry_price_offset_pct: f64,
    #[serde(default = "default_stop_limit_gap")]
    entry_stop_limit_gap_pct: f64,
    #[serde(default)]
    entry_expires_bars: Option<usize>,
    #[serde(default)]
    entry_bracket_sl: Option<f64>,
    #[serde(default)]
    entry_bracket_tp: Option<f64>,
    #[serde(default)]
    entry_bracket_trail: Option<f64>,
    #[serde(default)]
    short_order_type: String,
    #[serde(default = "default_price_offset")]
    short_price_offset_pct: f64,
    #[serde(default)]
    short_expires_bars: Option<usize>,
    #[serde(default)]
    short_bracket_sl: Option<f64>,
    #[serde(default)]
    short_bracket_tp: Option<f64>,
    #[serde(default)]
    short_bracket_trail: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerEnsembleMember {
    name: String,
    strategy: SerStrategyConfig,
    weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerEnsembleConfig {
    mode: String,
    members: Vec<SerEnsembleMember>,
}

fn default_scale_in_fraction() -> f64 {
    0.25
}
fn default_scale_out_fraction() -> f64 {
    0.50
}
fn default_price_offset() -> f64 {
    0.005
}
fn default_stop_limit_gap() -> f64 {
    0.002
}

fn long_order_type_to_str(ot: super::types::LongOrderType) -> String {
    match ot {
        super::types::LongOrderType::Market => "market".to_string(),
        super::types::LongOrderType::LimitBelow => "limit_below".to_string(),
        super::types::LongOrderType::StopAbove => "stop_above".to_string(),
        super::types::LongOrderType::StopLimitAbove => "stop_limit_above".to_string(),
    }
}

fn str_to_long_order_type(s: &str) -> super::types::LongOrderType {
    match s {
        "limit_below" => super::types::LongOrderType::LimitBelow,
        "stop_above" => super::types::LongOrderType::StopAbove,
        "stop_limit_above" => super::types::LongOrderType::StopLimitAbove,
        _ => super::types::LongOrderType::Market,
    }
}

fn short_order_type_to_str(ot: super::types::ShortOrderType) -> String {
    match ot {
        super::types::ShortOrderType::Market => "market".to_string(),
        super::types::ShortOrderType::LimitAbove => "limit_above".to_string(),
        super::types::ShortOrderType::StopBelow => "stop_below".to_string(),
    }
}

fn str_to_short_order_type(s: &str) -> super::types::ShortOrderType {
    match s {
        "limit_above" => super::types::ShortOrderType::LimitAbove,
        "stop_below" => super::types::ShortOrderType::StopBelow,
        _ => super::types::ShortOrderType::Market,
    }
}

fn ensemble_mode_to_str(mode: EnsembleModeChoice) -> &'static str {
    match mode {
        EnsembleModeChoice::WeightedMajority => "weighted_majority",
        EnsembleModeChoice::Unanimous => "unanimous",
        EnsembleModeChoice::AnySignal => "any_signal",
        EnsembleModeChoice::StrongestSignal => "strongest_signal",
    }
}

fn str_to_ensemble_mode(s: &str) -> EnsembleModeChoice {
    match s {
        "unanimous" => EnsembleModeChoice::Unanimous,
        "any_signal" => EnsembleModeChoice::AnySignal,
        "strongest_signal" => EnsembleModeChoice::StrongestSignal,
        _ => EnsembleModeChoice::WeightedMajority,
    }
}

/// Full serializable representation of a saved user preset.
///
/// Note: `symbol` is intentionally omitted — the symbol is always set by the
/// user at runtime. `optimizer` is also omitted because optimizer parameters
/// are transient configuration that should be re-tuned per run.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPresetData {
    name: String,
    description: String,
    created_at: String,
    interval: String,
    range: String,
    capital: f64,
    commission: f64,
    commission_flat: f64,
    slippage: f64,
    #[serde(default)]
    spread_pct: f64,
    #[serde(default)]
    transaction_tax_pct: f64,
    allow_short: bool,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    trailing_stop: Option<f64>,
    position_size: f64,
    #[serde(default = "default_max_positions")]
    max_positions: usize,
    risk_free_rate: f64,
    #[serde(default)]
    min_signal_strength: Option<f64>,
    #[serde(default)]
    close_at_end: Option<bool>,
    #[serde(default)]
    bars_per_year: Option<f64>,
    reinvest_dividends: bool,
    benchmark: Option<String>,
    #[serde(default)]
    ensemble: Option<SerEnsembleConfig>,
    strategy: SerStrategyConfig,
    /// Comma-separated additional symbols for portfolio mode (empty = single-symbol run).
    #[serde(default)]
    portfolio_symbols: Vec<String>,
    /// "available_capital" (default) or "equal_weight".
    #[serde(default)]
    rebalance_mode: String,
    /// Max allocation fraction per symbol (0.0 = unlimited).
    #[serde(default)]
    max_allocation_per_symbol: f64,
}

fn default_max_positions() -> usize {
    1
}

// ── Serialization ─────────────────────────────────────────────────────────────

fn ser_indicator(ind: &BuiltIndicator) -> SerBuiltIndicator {
    SerBuiltIndicator {
        code: ind.indicator.code.to_string(),
        param_values: ind.param_values.clone(),
        output: ind.output.clone(),
    }
}

fn ser_target(target: &CompareTarget) -> SerCompareTarget {
    match target {
        CompareTarget::Value(v) => SerCompareTarget::Value { value: *v },
        CompareTarget::Range(low, high) => SerCompareTarget::Range {
            low: *low,
            high: *high,
        },
        CompareTarget::Indicator(ind) => SerCompareTarget::Indicator {
            indicator: ser_indicator(ind),
        },
    }
}

fn ser_condition(cond: &BuiltCondition) -> SerBuiltCondition {
    SerBuiltCondition {
        indicator: ser_indicator(&cond.indicator),
        comparison: cond.comparison.name().to_string(),
        target: ser_target(&cond.target),
        htf_interval: cond
            .htf_interval
            .map(|interval| interval.as_str().to_string()),
        next_op: cond.next_op.name().to_lowercase(),
    }
}

fn ser_group(group: &ConditionGroup) -> SerConditionGroup {
    SerConditionGroup {
        conditions: group.conditions.iter().map(ser_condition).collect(),
    }
}

fn ser_strategy(strategy: &StrategyConfig) -> SerStrategyConfig {
    SerStrategyConfig {
        name: strategy.name.clone(),
        entry_conditions: ser_group(&strategy.entry_conditions),
        exit_conditions: ser_group(&strategy.exit_conditions),
        short_entry_conditions: strategy.short_entry_conditions.as_ref().map(ser_group),
        short_exit_conditions: strategy.short_exit_conditions.as_ref().map(ser_group),
        regime_conditions: ser_group(&strategy.regime_conditions),
        warmup_bars: strategy.warmup_bars,
        scale_in_conditions: ser_group(&strategy.scale_in_conditions),
        scale_in_fraction: strategy.scale_in_fraction,
        scale_out_conditions: ser_group(&strategy.scale_out_conditions),
        scale_out_fraction: strategy.scale_out_fraction,
        entry_order_type: long_order_type_to_str(strategy.entry_order_type),
        entry_price_offset_pct: strategy.entry_price_offset_pct,
        entry_stop_limit_gap_pct: strategy.entry_stop_limit_gap_pct,
        entry_expires_bars: strategy.entry_expires_bars,
        entry_bracket_sl: strategy.entry_bracket_sl,
        entry_bracket_tp: strategy.entry_bracket_tp,
        entry_bracket_trail: strategy.entry_bracket_trail,
        short_order_type: short_order_type_to_str(strategy.short_order_type),
        short_price_offset_pct: strategy.short_price_offset_pct,
        short_expires_bars: strategy.short_expires_bars,
        short_bracket_sl: strategy.short_bracket_sl,
        short_bracket_tp: strategy.short_bracket_tp,
        short_bracket_trail: strategy.short_bracket_trail,
    }
}

fn ser_ensemble(ensemble: &EnsembleConfig) -> SerEnsembleConfig {
    SerEnsembleConfig {
        mode: ensemble_mode_to_str(ensemble.mode).to_string(),
        members: ensemble
            .members
            .iter()
            .map(|member| SerEnsembleMember {
                name: member.name.clone(),
                strategy: ser_strategy(&member.strategy),
                weight: member.weight,
            })
            .collect(),
    }
}

// ── Deserialization ───────────────────────────────────────────────────────────

/// Returns `None` if the indicator code is not found in the current indicator
/// set (e.g. after a downgrade that removed an indicator).
fn deser_indicator(ser: &SerBuiltIndicator) -> Option<BuiltIndicator> {
    let indicator = IndicatorDef::all()
        .iter()
        .find(|i| i.code == ser.code.as_str())
        .cloned()?;
    Some(BuiltIndicator {
        indicator,
        param_values: ser.param_values.clone(),
        output: ser.output.clone(),
    })
}

fn deser_target(ser: &SerCompareTarget) -> Option<CompareTarget> {
    match ser {
        SerCompareTarget::Value { value } => Some(CompareTarget::Value(*value)),
        SerCompareTarget::Range { low, high } => Some(CompareTarget::Range(*low, *high)),
        SerCompareTarget::Indicator { indicator } => {
            deser_indicator(indicator).map(CompareTarget::Indicator)
        }
    }
}

fn deser_comparison(s: &str) -> Option<ComparisonType> {
    ComparisonType::all().into_iter().find(|c| c.name() == s)
}

fn deser_next_op(s: &str) -> LogicalOp {
    if s.eq_ignore_ascii_case("or") {
        LogicalOp::Or
    } else {
        LogicalOp::And
    }
}

/// Returns `None` if any part of the condition cannot be reconstructed (e.g.
/// unknown indicator code or comparison name). The caller uses `filter_map`
/// so invalid conditions are silently dropped — this is intentional graceful
/// degradation rather than a fatal error.
fn deser_condition(ser: &SerBuiltCondition) -> Option<BuiltCondition> {
    Some(BuiltCondition {
        indicator: deser_indicator(&ser.indicator)?,
        comparison: deser_comparison(&ser.comparison)?,
        target: deser_target(&ser.target)?,
        htf_interval: ser
            .htf_interval
            .as_deref()
            .and_then(|value| parse_interval(value).ok()),
        next_op: deser_next_op(&ser.next_op),
    })
}

fn deser_group(ser: &SerConditionGroup) -> ConditionGroup {
    ConditionGroup {
        conditions: ser.conditions.iter().filter_map(deser_condition).collect(),
    }
}

fn deser_strategy(ser: &SerStrategyConfig) -> StrategyConfig {
    StrategyConfig {
        name: ser.name.clone(),
        entry_conditions: deser_group(&ser.entry_conditions),
        exit_conditions: deser_group(&ser.exit_conditions),
        short_entry_conditions: ser.short_entry_conditions.as_ref().map(deser_group),
        short_exit_conditions: ser.short_exit_conditions.as_ref().map(deser_group),
        regime_conditions: deser_group(&ser.regime_conditions),
        warmup_bars: ser.warmup_bars,
        scale_in_conditions: deser_group(&ser.scale_in_conditions),
        scale_in_fraction: ser.scale_in_fraction,
        scale_out_conditions: deser_group(&ser.scale_out_conditions),
        scale_out_fraction: ser.scale_out_fraction,
        entry_order_type: str_to_long_order_type(&ser.entry_order_type),
        entry_price_offset_pct: ser.entry_price_offset_pct,
        entry_stop_limit_gap_pct: ser.entry_stop_limit_gap_pct,
        entry_expires_bars: ser.entry_expires_bars,
        entry_bracket_sl: ser.entry_bracket_sl,
        entry_bracket_tp: ser.entry_bracket_tp,
        entry_bracket_trail: ser.entry_bracket_trail,
        short_order_type: str_to_short_order_type(&ser.short_order_type),
        short_price_offset_pct: ser.short_price_offset_pct,
        short_expires_bars: ser.short_expires_bars,
        short_bracket_sl: ser.short_bracket_sl,
        short_bracket_tp: ser.short_bracket_tp,
        short_bracket_trail: ser.short_bracket_trail,
    }
}

fn deser_ensemble(ser: &SerEnsembleConfig) -> EnsembleConfig {
    EnsembleConfig {
        mode: str_to_ensemble_mode(&ser.mode),
        members: ser
            .members
            .iter()
            .map(|member| EnsembleMemberConfig {
                name: member.name.clone(),
                strategy: deser_strategy(&member.strategy),
                weight: member.weight,
            })
            .collect(),
    }
}

// ── Public types ──────────────────────────────────────────────────────────────

/// A user-saved strategy preset (runtime representation, not persisted directly).
#[derive(Debug, Clone)]
pub struct UserStrategyPreset {
    pub name: String,
    pub description: String,
    pub config: BacktestConfiguration,
}

// ── Conversion ────────────────────────────────────────────────────────────────

impl UserPresetData {
    fn from_config(name: String, description: String, config: &BacktestConfiguration) -> Self {
        Self {
            name,
            description,
            created_at: chrono::Local::now().to_rfc3339(),
            interval: config.interval.to_string(),
            range: config.range.to_string(),
            capital: config.capital,
            commission: config.commission,
            commission_flat: config.commission_flat,
            slippage: config.slippage,
            spread_pct: config.spread_pct,
            transaction_tax_pct: config.transaction_tax_pct,
            allow_short: config.allow_short,
            stop_loss: config.stop_loss,
            take_profit: config.take_profit,
            trailing_stop: config.trailing_stop,
            position_size: config.position_size,
            max_positions: config.max_positions,
            risk_free_rate: config.risk_free_rate,
            min_signal_strength: Some(config.min_signal_strength),
            close_at_end: Some(config.close_at_end),
            bars_per_year: Some(config.bars_per_year),
            reinvest_dividends: config.reinvest_dividends,
            benchmark: config.benchmark.clone(),
            ensemble: config.ensemble.as_ref().map(ser_ensemble),
            strategy: ser_strategy(&config.strategy),
            portfolio_symbols: config.portfolio_symbols.clone(),
            rebalance_mode: match config.rebalance_mode {
                RebalanceModeChoice::AvailableCapital => "available_capital".to_string(),
                RebalanceModeChoice::EqualWeight => "equal_weight".to_string(),
            },
            max_allocation_per_symbol: config.max_allocation_per_symbol,
        }
    }

    /// Returns `None` if the preset cannot be fully reconstructed (e.g. invalid
    /// interval/range strings or unknown indicator codes in strategy conditions).
    fn to_preset(&self) -> Option<UserStrategyPreset> {
        let interval = parse_interval(&self.interval).ok()?;
        let range = parse_range(&self.range).ok()?;
        let min_signal_strength = self.min_signal_strength.unwrap_or(0.0).clamp(0.0, 1.0);
        let close_at_end = self.close_at_end.unwrap_or(true);
        let bars_per_year = self
            .bars_per_year
            .filter(|v| *v > 0.0)
            .unwrap_or_else(|| bars_per_year_for_interval(interval));

        let config = BacktestConfiguration {
            symbol: String::new(),
            interval,
            range,
            capital: self.capital,
            commission: self.commission,
            commission_flat: self.commission_flat,
            slippage: self.slippage,
            spread_pct: self.spread_pct,
            transaction_tax_pct: self.transaction_tax_pct,
            allow_short: self.allow_short,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
            trailing_stop: self.trailing_stop,
            position_size: self.position_size,
            max_positions: self.max_positions,
            risk_free_rate: self.risk_free_rate,
            min_signal_strength,
            close_at_end,
            bars_per_year,
            reinvest_dividends: self.reinvest_dividends,
            benchmark: self.benchmark.clone(),
            ensemble: self.ensemble.as_ref().map(deser_ensemble),
            strategy: deser_strategy(&self.strategy),
            optimizer: None,
            portfolio_symbols: self.portfolio_symbols.clone(),
            rebalance_mode: match self.rebalance_mode.as_str() {
                "equal_weight" => RebalanceModeChoice::EqualWeight,
                _ => RebalanceModeChoice::AvailableCapital,
            },
            max_allocation_per_symbol: self.max_allocation_per_symbol,
        };
        Some(UserStrategyPreset {
            name: self.name.clone(),
            description: self.description.clone(),
            config,
        })
    }
}

// ── Storage ───────────────────────────────────────────────────────────────────

/// Handles reading and writing user presets from `~/.config/fq/presets.json`.
///
/// Follows the same pattern as [`crate::config::ConfigStorage`].
pub struct PresetStorage {
    file_path: PathBuf,
}

impl PresetStorage {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file_path: Self::default_path()?,
        })
    }

    fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("fq");
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("presets.json"))
    }

    fn load_raw(&self) -> Result<Vec<UserPresetData>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.file_path).context("Failed to read presets file")?;
        Ok(serde_json::from_str(&content).context("Failed to parse presets file")?)
    }

    fn save_raw(&self, presets: &[UserPresetData]) -> Result<()> {
        let content =
            serde_json::to_string_pretty(presets).context("Failed to serialize presets")?;
        Ok(fs::write(&self.file_path, content).context("Failed to write presets file")?)
    }

    /// Loads all valid user presets. Entries that cannot be reconstructed
    /// (e.g. unknown indicator codes after a version change) are silently
    /// skipped to prevent blocking the TUI from launching.
    pub fn load_all(&self) -> Vec<UserStrategyPreset> {
        match self.load_raw() {
            Ok(raw) => raw.iter().filter_map(|p| p.to_preset()).collect(),
            Err(e) => {
                tracing::warn!("Failed to load user presets: {e}");
                Vec::new()
            }
        }
    }

    /// Saves (or replaces) a preset. Upserts by name.
    pub fn save(
        &self,
        name: String,
        description: String,
        config: &BacktestConfiguration,
    ) -> Result<()> {
        let mut presets = self.load_raw().unwrap_or_default();
        let data = UserPresetData::from_config(name.clone(), description, config);
        match presets.iter_mut().find(|p| p.name == name) {
            Some(existing) => *existing = data,
            None => presets.push(data),
        }
        self.save_raw(&presets)
    }

    /// Removes a preset by name. No-op if the name does not exist.
    pub fn delete(&self, name: &str) -> Result<()> {
        let mut presets = self.load_raw().unwrap_or_default();
        presets.retain(|p| p.name != name);
        self.save_raw(&presets)
    }
}

// ── Convenience helpers used by App ──────────────────────────────────────────

/// Loads all user presets. Returns an empty list on any I/O or parse error.
pub fn load_user_presets() -> Vec<UserStrategyPreset> {
    PresetStorage::new()
        .map(|s| s.load_all())
        .unwrap_or_default()
}

/// Saves a user preset, upserts by name.
pub fn save_user_preset(
    name: String,
    description: String,
    config: &BacktestConfiguration,
) -> Result<()> {
    PresetStorage::new()?.save(name, description, config)
}

/// Deletes a user preset by name.
pub fn delete_user_preset(name: &str) -> Result<()> {
    PresetStorage::new()?.delete(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::indicators::IndicatorDef;
    use finance_query::Interval;

    #[test]
    fn legacy_preset_defaults_new_fields() {
        let config = BacktestConfiguration {
            interval: Interval::OneWeek,
            min_signal_strength: 0.35,
            close_at_end: false,
            bars_per_year: 999.0,
            ..BacktestConfiguration::default()
        };

        let mut data = UserPresetData::from_config("Legacy".into(), "Test".into(), &config);
        data.min_signal_strength = None;
        data.close_at_end = None;
        data.bars_per_year = None;

        let preset = data.to_preset().expect("preset should deserialize");
        assert!((preset.config.min_signal_strength - 0.0).abs() < 1e-9);
        assert!(preset.config.close_at_end);
        assert!(
            (preset.config.bars_per_year - bars_per_year_for_interval(Interval::OneWeek)).abs()
                < 1e-9
        );
    }

    #[test]
    fn preset_round_trips_new_fields() {
        let config = BacktestConfiguration {
            min_signal_strength: 0.42,
            close_at_end: false,
            bars_per_year: 365.0,
            ..BacktestConfiguration::default()
        };

        let data = UserPresetData::from_config("RoundTrip".into(), "Test".into(), &config);
        let preset = data.to_preset().expect("preset should deserialize");

        assert!((preset.config.min_signal_strength - 0.42).abs() < 1e-9);
        assert!(!preset.config.close_at_end);
        assert!((preset.config.bars_per_year - 365.0).abs() < 1e-9);
    }

    #[test]
    fn preset_round_trips_condition_htf_interval() {
        let mut config = BacktestConfiguration::default();
        config
            .strategy
            .entry_conditions
            .conditions
            .push(BuiltCondition {
                indicator: BuiltIndicator {
                    indicator: IndicatorDef::find("rsi"),
                    param_values: vec![14.0],
                    output: None,
                },
                comparison: ComparisonType::Above,
                target: CompareTarget::Value(50.0),
                htf_interval: Some(Interval::OneWeek),
                next_op: LogicalOp::And,
            });

        let data = UserPresetData::from_config("HTF".into(), "Test".into(), &config);
        let preset = data.to_preset().expect("preset should deserialize");

        let cond = preset
            .config
            .strategy
            .entry_conditions
            .conditions
            .first()
            .expect("condition should be present");
        assert_eq!(cond.htf_interval, Some(Interval::OneWeek));
    }

    #[test]
    fn preset_round_trips_ensemble_config() {
        let config = BacktestConfiguration {
            ensemble: Some(EnsembleConfig {
                mode: EnsembleModeChoice::AnySignal,
                members: vec![
                    EnsembleMemberConfig {
                        name: "Swing Trader".to_string(),
                        strategy: StrategyConfig::default(),
                        weight: 1.0,
                    },
                    EnsembleMemberConfig {
                        name: "RSI Mean Reversion".to_string(),
                        strategy: StrategyConfig::default(),
                        weight: 0.7,
                    },
                ],
            }),
            ..BacktestConfiguration::default()
        };

        let data = UserPresetData::from_config("Ensemble".into(), "Test".into(), &config);
        let preset = data.to_preset().expect("preset should deserialize");

        let ensemble = preset
            .config
            .ensemble
            .expect("ensemble should be persisted");
        assert_eq!(ensemble.mode, EnsembleModeChoice::AnySignal);
        assert_eq!(ensemble.members.len(), 2);
        assert_eq!(ensemble.members[0].name, "Swing Trader");
        assert!((ensemble.members[1].weight - 0.7).abs() < 1e-9);
    }
}
