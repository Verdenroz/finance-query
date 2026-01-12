use super::indicators::{IndicatorCategory, IndicatorDef};
use super::presets::StrategyPreset;
use super::types::{
    BacktestConfiguration, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup, LogicalOp,
};
use crate::error::Result;
use finance_query::{Interval, TimeRange};
use ratatui::style::Color;

/// Main TUI screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Welcome screen with options
    Welcome,
    /// Load a preset strategy
    PresetSelect,
    /// Main configuration editor
    ConfigEditor,
    /// Strategy builder - select entry/exit
    StrategyBuilder,
    /// Indicator category browser
    IndicatorBrowser,
    /// Configure indicator parameters
    IndicatorConfig,
    /// Configure comparison
    ComparisonConfig,
    /// Configure target value or indicator
    TargetConfig,
    /// Review and confirm
    Confirmation,
}

/// What we're currently building a condition for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionTarget {
    Entry,
    Exit,
    ShortEntry,
    ShortExit,
}

impl ConditionTarget {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Entry => "Entry",
            Self::Exit => "Exit",
            Self::ShortEntry => "Short Entry",
            Self::ShortExit => "Short Exit",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Entry => Color::Green,
            Self::Exit => Color::Yellow,
            Self::ShortEntry => Color::Red,
            Self::ShortExit => Color::Magenta,
        }
    }
}

/// Configuration fields that can be edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Symbol,
    Interval,
    Range,
    Capital,
    Commission,
    Slippage,
    AllowShort,
    StopLoss,
    TakeProfit,
    PositionSize,
}

impl ConfigField {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Symbol,
            Self::Interval,
            Self::Range,
            Self::Capital,
            Self::Commission,
            Self::Slippage,
            Self::AllowShort,
            Self::StopLoss,
            Self::TakeProfit,
            Self::PositionSize,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::Interval => "Interval",
            Self::Range => "Time Range",
            Self::Capital => "Capital",
            Self::Commission => "Commission",
            Self::Slippage => "Slippage",
            Self::AllowShort => "Allow Short",
            Self::StopLoss => "Stop Loss",
            Self::TakeProfit => "Take Profit",
            Self::PositionSize => "Position Size",
        }
    }

    pub fn help(&self) -> &'static str {
        match self {
            Self::Symbol => "Stock ticker symbol (e.g., AAPL, TSLA, MSFT)",
            Self::Interval => "Candle interval: 1m, 5m, 15m, 1h, 1d, 1wk, 1mo",
            Self::Range => "Historical range: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, max",
            Self::Capital => "Starting capital in dollars",
            Self::Commission => "Commission per trade (e.g., 0.1 for 0.1%)",
            Self::Slippage => "Slippage per trade (e.g., 0.1 for 0.1%)",
            Self::AllowShort => "Enable short selling (true/false)",
            Self::StopLoss => "Stop loss percentage (empty for none)",
            Self::TakeProfit => "Take profit percentage (empty for none)",
            Self::PositionSize => "Position size as % of capital (e.g., 100)",
        }
    }
}

/// Which condition panel is active in strategy builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionPanel {
    Entry,
    Exit,
}

/// Main application state
pub struct App {
    // Navigation
    pub screen: Screen,
    pub prev_screens: Vec<Screen>,

    // Configuration
    pub config: BacktestConfiguration,

    // Editing state
    pub editing: bool,
    pub edit_buffer: String,
    pub edit_error: Option<String>,

    // List selections
    pub config_field_idx: usize,
    pub category_idx: usize,
    pub indicator_idx: usize,
    pub preset_idx: usize,
    pub comparison_idx: usize,
    pub param_idx: usize,

    // Condition building state
    pub condition_target: ConditionTarget,
    pub building_indicator: Option<BuiltIndicator>,
    pub building_comparison: Option<ComparisonType>,
    pub param_values: Vec<f64>,
    pub target_is_indicator: bool,
    pub target_value: f64,
    pub target_value2: f64,         // For Between comparison
    pub editing_target_value: bool, // true = editing primary, false = editing secondary (for Between)
    pub target_input_mode: bool,    // true when user is typing a number for target
    pub target_edit_buffer: String,

    // Strategy builder condition list selection
    pub active_condition_panel: ConditionPanel,
    pub entry_condition_idx: usize,
    pub exit_condition_idx: usize,

    // Available data
    pub presets: Vec<StrategyPreset>,
    pub indicators: Vec<IndicatorDef>,

    // Control
    pub should_quit: bool,
    pub confirmed: bool,
}

impl App {
    pub fn new(initial_symbol: Option<String>) -> Self {
        let mut config = BacktestConfiguration::default();
        if let Some(sym) = initial_symbol {
            config.symbol = sym.to_uppercase();
        }

        Self {
            screen: Screen::Welcome,
            prev_screens: Vec::new(),
            config,
            editing: false,
            edit_buffer: String::new(),
            edit_error: None,
            config_field_idx: 0,
            category_idx: 0,
            indicator_idx: 0,
            preset_idx: 0,
            comparison_idx: 0,
            param_idx: 0,
            condition_target: ConditionTarget::Entry,
            building_indicator: None,
            building_comparison: None,
            param_values: Vec::new(),
            target_is_indicator: false,
            target_value: 0.0,
            target_value2: 0.0,
            editing_target_value: true,
            target_input_mode: false,
            target_edit_buffer: String::new(),
            active_condition_panel: ConditionPanel::Entry,
            entry_condition_idx: 0,
            exit_condition_idx: 0,
            presets: StrategyPreset::all(),
            indicators: IndicatorDef::all(),
            should_quit: false,
            confirmed: false,
        }
    }

    pub fn push_screen(&mut self, screen: Screen) {
        self.prev_screens.push(self.screen);
        self.screen = screen;
    }

    pub fn pop_screen(&mut self) {
        if let Some(prev) = self.prev_screens.pop() {
            self.screen = prev;
        }
    }

    pub fn current_category(&self) -> IndicatorCategory {
        IndicatorCategory::all()[self.category_idx]
    }

    pub fn indicators_in_category(&self) -> Vec<&IndicatorDef> {
        // Filter from all available indicators by current category
        let category = self.current_category();
        self.indicators
            .iter()
            .filter(|i| i.category == category)
            .collect()
    }

    /// Count indicators per category
    pub fn indicator_count_by_category(&self, category: IndicatorCategory) -> usize {
        IndicatorDef::by_category(category).len()
    }

    pub fn current_indicator(&self) -> Option<&IndicatorDef> {
        self.indicators_in_category()
            .get(self.indicator_idx)
            .copied()
    }

    pub fn current_config_field(&self) -> ConfigField {
        ConfigField::all()[self.config_field_idx]
    }

    pub fn format_field_value(&self, field: ConfigField) -> String {
        match field {
            ConfigField::Symbol => {
                if self.config.symbol.is_empty() {
                    "(not set)".to_string()
                } else {
                    self.config.symbol.clone()
                }
            }
            ConfigField::Interval => interval_to_string(self.config.interval),
            ConfigField::Range => range_to_string(self.config.range),
            ConfigField::Capital => format!("${:.2}", self.config.capital),
            ConfigField::Commission => format!("{:.2}%", self.config.commission * 100.0),
            ConfigField::Slippage => format!("{:.2}%", self.config.slippage * 100.0),
            ConfigField::AllowShort => format!("{}", self.config.allow_short),
            ConfigField::StopLoss => self
                .config
                .stop_loss
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::TakeProfit => self
                .config
                .take_profit
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::PositionSize => format!("{:.0}%", self.config.position_size * 100.0),
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
        self.edit_error = None;
        let field = self.current_config_field();
        self.edit_buffer = match field {
            ConfigField::Symbol => self.config.symbol.clone(),
            ConfigField::Interval => interval_to_string(self.config.interval),
            ConfigField::Range => range_to_string(self.config.range),
            ConfigField::Capital => format!("{}", self.config.capital),
            ConfigField::Commission => format!("{}", self.config.commission * 100.0),
            ConfigField::Slippage => format!("{}", self.config.slippage * 100.0),
            ConfigField::AllowShort => format!("{}", self.config.allow_short),
            ConfigField::StopLoss => self
                .config
                .stop_loss
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::TakeProfit => self
                .config
                .take_profit
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::PositionSize => format!("{}", self.config.position_size * 100.0),
        };
    }

    pub fn finish_editing(&mut self) {
        let value = self.edit_buffer.trim();
        let field = self.current_config_field();

        let result: Result<()> = (|| {
            match field {
                ConfigField::Symbol => {
                    self.config.symbol = value.to_uppercase();
                }
                ConfigField::Interval => {
                    self.config.interval = parse_interval(value)?;
                }
                ConfigField::Range => {
                    self.config.range = parse_range(value)?;
                }
                ConfigField::Capital => {
                    let cap: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if cap <= 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Capital must be positive".into(),
                        ));
                    }
                    self.config.capital = cap;
                }
                ConfigField::Commission => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    self.config.commission = v / 100.0;
                }
                ConfigField::Slippage => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    self.config.slippage = v / 100.0;
                }
                ConfigField::AllowShort => {
                    self.config.allow_short = parse_bool(value)?;
                }
                ConfigField::StopLoss => {
                    if value.is_empty() {
                        self.config.stop_loss = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        self.config.stop_loss = Some(v / 100.0);
                    }
                }
                ConfigField::TakeProfit => {
                    if value.is_empty() {
                        self.config.take_profit = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        self.config.take_profit = Some(v / 100.0);
                    }
                }
                ConfigField::PositionSize => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v <= 0.0 || v > 100.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Position size must be 0-100%".into(),
                        ));
                    }
                    self.config.position_size = v / 100.0;
                }
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.editing = false;
                self.edit_buffer.clear();
                self.edit_error = None;
            }
            Err(e) => {
                self.edit_error = Some(e.to_string());
            }
        }
    }

    pub fn cancel_editing(&mut self) {
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_error = None;
    }

    pub fn load_preset(&mut self, idx: usize) {
        if let Some(preset) = self.presets.get(idx) {
            let mut preset_config = (preset.config)();
            // Keep the symbol if already set
            if !self.config.symbol.is_empty() {
                preset_config.symbol = self.config.symbol.clone();
            }
            self.config = preset_config;
        }
    }

    pub fn select_indicator(&mut self) {
        // Clone indicator to avoid borrowing issues
        let ind = self.current_indicator().cloned();
        if let Some(ind) = ind {
            // Initialize parameter values with defaults
            let param_values: Vec<f64> = ind.params.iter().map(|p| p.default).collect();
            self.param_values = param_values.clone();
            self.param_idx = 0;
            self.building_indicator = Some(BuiltIndicator {
                indicator: ind,
                param_values,
                output: None,
            });
            self.push_screen(Screen::IndicatorConfig);
        }
    }

    pub fn finish_indicator_config(&mut self) {
        if let Some(ref mut ind) = self.building_indicator {
            ind.param_values = self.param_values.clone();
        }
        self.comparison_idx = 0;
        self.push_screen(Screen::ComparisonConfig);
    }

    pub fn select_comparison(&mut self) {
        self.building_comparison = Some(ComparisonType::all()[self.comparison_idx]);

        // Set default target value based on indicator's typical range
        if let Some(ref ind) = self.building_indicator {
            if let Some((low, high)) = ind.indicator.typical_range {
                self.target_value = (low + high) / 2.0;
                self.target_value2 = high;
            } else {
                self.target_value = 0.0;
                self.target_value2 = 0.0;
            }
        }

        self.target_is_indicator = false;
        self.push_screen(Screen::TargetConfig);
    }

    pub fn finish_condition(&mut self) {
        use super::types::BuiltCondition;

        if let (Some(ind), Some(comp)) = (
            self.building_indicator.take(),
            self.building_comparison.take(),
        ) {
            let target = if self.target_is_indicator {
                // For simplicity, compare against same indicator with different params
                // In a full implementation, you'd allow selecting any indicator
                CompareTarget::Indicator(ind.clone())
            } else if comp.needs_range() {
                CompareTarget::Range(self.target_value, self.target_value2)
            } else {
                CompareTarget::Value(self.target_value)
            };

            let condition = BuiltCondition {
                indicator: ind,
                comparison: comp,
                target,
                next_op: LogicalOp::And, // Default to AND for new conditions
            };

            // Add to appropriate condition group
            match self.condition_target {
                ConditionTarget::Entry => {
                    self.config
                        .strategy
                        .entry_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::Exit => {
                    self.config
                        .strategy
                        .exit_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ShortEntry => {
                    if self.config.strategy.short_entry_conditions.is_none() {
                        self.config.strategy.short_entry_conditions = Some(ConditionGroup::new());
                    }
                    self.config
                        .strategy
                        .short_entry_conditions
                        .as_mut()
                        .unwrap()
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ShortExit => {
                    if self.config.strategy.short_exit_conditions.is_none() {
                        self.config.strategy.short_exit_conditions = Some(ConditionGroup::new());
                    }
                    self.config
                        .strategy
                        .short_exit_conditions
                        .as_mut()
                        .unwrap()
                        .conditions
                        .push(condition);
                }
            }

            // Return to strategy builder
            self.prev_screens.clear();
            self.screen = Screen::StrategyBuilder;
        }
    }

    pub fn can_run(&self) -> bool {
        !self.config.symbol.is_empty()
            && !self.config.strategy.entry_conditions.conditions.is_empty()
            && !self.config.strategy.exit_conditions.conditions.is_empty()
    }
}

// Helper functions for parsing and formatting

pub fn interval_to_string(interval: Interval) -> String {
    match interval {
        Interval::OneMinute => "1m",
        Interval::FiveMinutes => "5m",
        Interval::FifteenMinutes => "15m",
        Interval::ThirtyMinutes => "30m",
        Interval::OneHour => "1h",
        Interval::OneDay => "1d",
        Interval::OneWeek => "1wk",
        Interval::OneMonth => "1mo",
        Interval::ThreeMonths => "3mo",
    }
    .to_string()
}

pub fn range_to_string(range: TimeRange) -> String {
    match range {
        TimeRange::OneDay => "1d",
        TimeRange::FiveDays => "5d",
        TimeRange::OneMonth => "1mo",
        TimeRange::ThreeMonths => "3mo",
        TimeRange::SixMonths => "6mo",
        TimeRange::OneYear => "1y",
        TimeRange::TwoYears => "2y",
        TimeRange::FiveYears => "5y",
        TimeRange::TenYears => "10y",
        TimeRange::YearToDate => "ytd",
        TimeRange::Max => "max",
    }
    .to_string()
}

pub fn parse_interval(s: &str) -> Result<Interval> {
    match s.to_lowercase().as_str() {
        "1m" => Ok(Interval::OneMinute),
        "5m" => Ok(Interval::FiveMinutes),
        "15m" => Ok(Interval::FifteenMinutes),
        "30m" => Ok(Interval::ThirtyMinutes),
        "1h" => Ok(Interval::OneHour),
        "1d" => Ok(Interval::OneDay),
        "1wk" => Ok(Interval::OneWeek),
        "1mo" => Ok(Interval::OneMonth),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid interval: {}",
            s
        ))),
    }
}

pub fn parse_range(s: &str) -> Result<TimeRange> {
    match s.to_lowercase().as_str() {
        "1d" => Ok(TimeRange::OneDay),
        "5d" => Ok(TimeRange::FiveDays),
        "1mo" => Ok(TimeRange::OneMonth),
        "3mo" => Ok(TimeRange::ThreeMonths),
        "6mo" => Ok(TimeRange::SixMonths),
        "1y" => Ok(TimeRange::OneYear),
        "2y" => Ok(TimeRange::TwoYears),
        "5y" => Ok(TimeRange::FiveYears),
        "10y" => Ok(TimeRange::TenYears),
        "ytd" => Ok(TimeRange::YearToDate),
        "max" => Ok(TimeRange::Max),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid range: {}",
            s
        ))),
    }
}

pub fn parse_bool(s: &str) -> Result<bool> {
    match s.to_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Ok(true),
        "false" | "no" | "n" | "0" => Ok(false),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid boolean: {}",
            s
        ))),
    }
}
