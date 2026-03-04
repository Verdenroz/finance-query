use super::indicators::IndicatorDef;
use super::state::{parse_interval, parse_range};
use super::types::{
    BacktestConfiguration, BuiltCondition, BuiltIndicator, CompareTarget, ComparisonType,
    ConditionGroup, LogicalOp, StrategyConfig,
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
    /// "and" or "or" (lowercase).
    next_op: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    allow_short: bool,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
    trailing_stop: Option<f64>,
    position_size: f64,
    risk_free_rate: f64,
    reinvest_dividends: bool,
    benchmark: Option<String>,
    strategy: SerStrategyConfig,
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
            allow_short: config.allow_short,
            stop_loss: config.stop_loss,
            take_profit: config.take_profit,
            trailing_stop: config.trailing_stop,
            position_size: config.position_size,
            risk_free_rate: config.risk_free_rate,
            reinvest_dividends: config.reinvest_dividends,
            benchmark: config.benchmark.clone(),
            strategy: ser_strategy(&config.strategy),
        }
    }

    /// Returns `None` if the preset cannot be fully reconstructed (e.g. invalid
    /// interval/range strings or unknown indicator codes in strategy conditions).
    fn to_preset(&self) -> Option<UserStrategyPreset> {
        let interval = parse_interval(&self.interval).ok()?;
        let range = parse_range(&self.range).ok()?;
        let config = BacktestConfiguration {
            symbol: String::new(),
            interval,
            range,
            capital: self.capital,
            commission: self.commission,
            commission_flat: self.commission_flat,
            slippage: self.slippage,
            allow_short: self.allow_short,
            stop_loss: self.stop_loss,
            take_profit: self.take_profit,
            trailing_stop: self.trailing_stop,
            position_size: self.position_size,
            risk_free_rate: self.risk_free_rate,
            reinvest_dividends: self.reinvest_dividends,
            benchmark: self.benchmark.clone(),
            strategy: deser_strategy(&self.strategy),
            optimizer: None,
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
