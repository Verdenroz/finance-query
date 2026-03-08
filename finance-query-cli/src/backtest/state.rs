use super::indicators::{IndicatorCategory, IndicatorDef};
use super::presets::StrategyPreset;
use super::types::{
    BacktestConfiguration, BuiltIndicator, CompareTarget, ComparisonType, ConditionGroup,
    EnsembleConfig, EnsembleMemberConfig, EnsembleModeChoice, LogicalOp, OptimizerParamDef,
    WALK_FORWARD_IN_SAMPLE_BARS, WALK_FORWARD_OOS_BARS, bars_per_year_for_interval,
};
use super::user_presets::{self, UserStrategyPreset};
use crate::error::Result;
use finance_query::{Interval, TimeRange};
use ratatui::style::Color;
use std::collections::HashMap;

/// Optimizer field column indices (used in optimizer_field_idx)
pub const OPTIMIZER_FIELD_START: usize = 0;
pub const OPTIMIZER_FIELD_END: usize = 1;
pub const OPTIMIZER_FIELD_STEP: usize = 2;
pub const OPTIMIZER_FIELD_IN_SAMPLE: usize = 3;
pub const OPTIMIZER_FIELD_OOS: usize = 4;
pub const OPTIMIZER_FIELD_MAX: usize = 4;

/// Main TUI screens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Welcome screen with options
    Welcome,
    /// Load a preset strategy
    PresetSelect,
    /// Compose multiple presets into an ensemble strategy
    EnsembleCompose,
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
    /// Optimizer parameter configuration
    OptimizerSetup,
    /// Save current strategy as a named user preset
    SavePreset,
}

/// What we're currently building a condition for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionTarget {
    Entry,
    Exit,
    ShortEntry,
    ShortExit,
    Regime,
    ScaleIn,
    ScaleOut,
}

impl ConditionTarget {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Entry => "Entry",
            Self::Exit => "Exit",
            Self::ShortEntry => "Short Entry",
            Self::ShortExit => "Short Exit",
            Self::Regime => "Regime Filter",
            Self::ScaleIn => "Scale-In",
            Self::ScaleOut => "Scale-Out",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Entry => Color::Green,
            Self::Exit => Color::Yellow,
            Self::ShortEntry => Color::Red,
            Self::ShortExit => Color::Magenta,
            Self::Regime => Color::Cyan,
            Self::ScaleIn => Color::LightBlue,
            Self::ScaleOut => Color::LightRed,
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
    CostProfile,
    Commission,
    CommissionFlat,
    Slippage,
    SpreadPct,
    TransactionTaxPct,
    AllowShort,
    StopLoss,
    TakeProfit,
    TrailingStop,
    PositionSize,
    MaxPositions,
    WarmupBars,
    RiskFreeRate,
    MinSignalStrength,
    CloseAtEnd,
    BarsPerYear,
    ReinvestDividends,
    Benchmark,
    EntryOrderType,
    EntryPriceOffset,
    EntryStopLimitGap,
    EntryExpiryBars,
    EntryBracketSL,
    EntryBracketTP,
    EntryBracketTrail,
    ShortOrderType,
    ShortPriceOffset,
    ShortExpiryBars,
    ShortBracketSL,
    ShortBracketTP,
    ShortBracketTrail,
    /// Comma-separated extra symbols for portfolio mode (empty = single-symbol).
    PortfolioSymbols,
    /// Capital allocation strategy for portfolio mode.
    RebalanceMode,
}

impl ConfigField {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Symbol,
            Self::Interval,
            Self::Range,
            Self::Capital,
            Self::CostProfile,
            Self::Commission,
            Self::CommissionFlat,
            Self::Slippage,
            Self::SpreadPct,
            Self::TransactionTaxPct,
            Self::AllowShort,
            Self::StopLoss,
            Self::TakeProfit,
            Self::TrailingStop,
            Self::PositionSize,
            Self::MaxPositions,
            Self::WarmupBars,
            Self::RiskFreeRate,
            Self::MinSignalStrength,
            Self::CloseAtEnd,
            Self::BarsPerYear,
            Self::ReinvestDividends,
            Self::Benchmark,
            Self::EntryOrderType,
            Self::EntryPriceOffset,
            Self::EntryStopLimitGap,
            Self::EntryExpiryBars,
            Self::EntryBracketSL,
            Self::EntryBracketTP,
            Self::EntryBracketTrail,
            Self::ShortOrderType,
            Self::ShortPriceOffset,
            Self::ShortExpiryBars,
            Self::ShortBracketSL,
            Self::ShortBracketTP,
            Self::ShortBracketTrail,
            Self::PortfolioSymbols,
            Self::RebalanceMode,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::Interval => "Interval",
            Self::Range => "Time Range",
            Self::Capital => "Capital",
            Self::CostProfile => "Cost Profile",
            Self::Commission => "Commission %",
            Self::CommissionFlat => "Flat Commission",
            Self::Slippage => "Slippage %",
            Self::SpreadPct => "Spread %",
            Self::TransactionTaxPct => "Transaction Tax %",
            Self::AllowShort => "Allow Short",
            Self::StopLoss => "Stop Loss",
            Self::TakeProfit => "Take Profit",
            Self::TrailingStop => "Trailing Stop",
            Self::PositionSize => "Position Size",
            Self::MaxPositions => "Max Positions",
            Self::WarmupBars => "Warmup Bars",
            Self::RiskFreeRate => "Risk-Free Rate",
            Self::MinSignalStrength => "Min Signal %",
            Self::CloseAtEnd => "Close At End",
            Self::BarsPerYear => "Bars / Year",
            Self::ReinvestDividends => "Reinvest Divs",
            Self::Benchmark => "Benchmark",
            Self::EntryOrderType => "Entry Order",
            Self::EntryPriceOffset => "Entry Offset %",
            Self::EntryStopLimitGap => "SL Gap %",
            Self::EntryExpiryBars => "Entry Expiry",
            Self::EntryBracketSL => "Entry Trade SL",
            Self::EntryBracketTP => "Entry Trade TP",
            Self::EntryBracketTrail => "Entry Trail",
            Self::ShortOrderType => "Short Order",
            Self::ShortPriceOffset => "Short Offset %",
            Self::ShortExpiryBars => "Short Expiry",
            Self::ShortBracketSL => "Short Trade SL",
            Self::ShortBracketTP => "Short Trade TP",
            Self::ShortBracketTrail => "Short Trail",
            Self::PortfolioSymbols => "Portfolio Symbols",
            Self::RebalanceMode => "Rebalance Mode",
        }
    }

    pub fn help(&self) -> &'static str {
        match self {
            Self::Symbol => "Stock ticker symbol (e.g., AAPL, TSLA, MSFT)",
            Self::Interval => "Candle interval: 1m, 5m, 15m, 1h, 1d, 1wk, 1mo",
            Self::Range => "Historical range: 1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, max",
            Self::Capital => "Starting capital in dollars",
            Self::CostProfile => {
                "Press Enter to cycle cost presets (Zero Cost, IBKR, Retail, UK Stamp Duty)"
            }
            Self::Commission => "% commission per trade (stacks with flat fee; e.g., 0.1 for 0.1%)",
            Self::CommissionFlat => "Flat $ fee per trade (stacks with % commission; e.g., 5.00)",
            Self::Slippage => "Slippage per trade (e.g., 0.1 for 0.1%)",
            Self::SpreadPct => {
                "Symmetric bid-ask spread % (e.g., 0.02 for 2 bps; half applied each side)"
            }
            Self::TransactionTaxPct => {
                "Purchase tax % on buy orders only (e.g., 0.5 for UK stamp duty)"
            }
            Self::AllowShort => "Enable short selling (true/false)",
            Self::StopLoss => "Stop loss percentage (empty for none, e.g., 5 for 5%)",
            Self::TakeProfit => "Take profit percentage (empty for none, e.g., 10 for 10%)",
            Self::TrailingStop => "Trailing stop percentage (empty for none, e.g., 3 for 3%)",
            Self::PositionSize => "Position size as % of capital (e.g., 100)",
            Self::MaxPositions => "Max concurrent positions (0 = unlimited, default 1)",
            Self::WarmupBars => {
                "Bars to skip before strategy starts trading (e.g., 200 to wait for SMA(200))"
            }
            Self::RiskFreeRate => {
                "Annual risk-free rate for Sharpe/Sortino/Calmar (e.g., 4 for 4%). Default 0% inflates Sharpe — set to current T-bill rate for accurate results."
            }
            Self::MinSignalStrength => {
                "Minimum signal strength % required to execute trades (0-100, usually 0 for dynamic strategies)"
            }
            Self::CloseAtEnd => "Close any open position on the final bar (true/false)",
            Self::BarsPerYear => {
                "Bars per calendar year for annualized metrics (e.g., 252 daily, 52 weekly, 1638 hourly)"
            }
            Self::ReinvestDividends => "Reinvest dividend income into position (true/false)",
            Self::Benchmark => {
                "Benchmark symbol for alpha/beta/info-ratio (e.g., SPY, QQQ; leave empty for none)"
            }
            Self::EntryOrderType => {
                "Long entry order: Market, Limit Below, Stop Above, Stop-Limit Above. Press Enter to cycle."
            }
            Self::EntryPriceOffset => {
                "Price offset % for limit/stop long entries (e.g. 0.5 → buy 0.5% below/above close). For Stop-Limit: this is the stop trigger offset."
            }
            Self::EntryStopLimitGap => {
                "Gap % above stop price for Stop-Limit Above orders. limit = stop * (1 + gap). Inactive for other order types."
            }
            Self::EntryExpiryBars => {
                "Bars until a pending long entry order is cancelled (0 = Good-Till-Cancelled)"
            }
            Self::EntryBracketSL => {
                "Per-trade stop-loss % for long entries — overrides global Stop Loss (empty = use global)"
            }
            Self::EntryBracketTP => {
                "Per-trade take-profit % for long entries — overrides global Take Profit (empty = use global)"
            }
            Self::EntryBracketTrail => {
                "Per-trade trailing stop % for long entries — overrides global Trailing Stop (empty = use global)"
            }
            Self::ShortOrderType => {
                "Short entry order: Market, Limit Above (sell rally), Stop Below (breakdown). Press Enter to cycle."
            }
            Self::ShortPriceOffset => "Price offset % for limit/stop short entries",
            Self::ShortExpiryBars => {
                "Bars until a pending short entry order is cancelled (0 = GTC)"
            }
            Self::ShortBracketSL => "Per-trade stop-loss % for short entries (empty = use global)",
            Self::ShortBracketTP => {
                "Per-trade take-profit % for short entries (empty = use global)"
            }
            Self::ShortBracketTrail => {
                "Per-trade trailing stop % for short entries (empty = use global)"
            }
            Self::PortfolioSymbols => {
                "Comma-separated extra symbols for portfolio mode (e.g. MSFT,GOOGL,NVDA). Leave empty for single-symbol backtesting."
            }
            Self::RebalanceMode => {
                "Portfolio capital allocation: Available Capital (position_size % of cash) or Equal Weight (capital / symbols). Press Enter to toggle."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostProfile {
    Custom,
    ZeroCost,
    InteractiveBrokersUs,
    CommissionFreeRetail,
    UkStampDuty,
}

impl CostProfile {
    pub fn name(self) -> &'static str {
        match self {
            Self::Custom => "Custom",
            Self::ZeroCost => "Zero Cost",
            Self::InteractiveBrokersUs => "Interactive Brokers (US)",
            Self::CommissionFreeRetail => "Commission-Free Retail",
            Self::UkStampDuty => "UK Shares (Stamp Duty)",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Custom => Self::ZeroCost,
            Self::ZeroCost => Self::InteractiveBrokersUs,
            Self::InteractiveBrokersUs => Self::CommissionFreeRetail,
            Self::CommissionFreeRetail => Self::UkStampDuty,
            Self::UkStampDuty => Self::ZeroCost,
        }
    }
}

/// Which condition panel is active in strategy builder
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionPanel {
    Entry,
    Exit,
    Regime,
    ScaleIn,
    ScaleOut,
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
    pub building_htf_interval: Option<Interval>,
    pub param_values: Vec<f64>,
    pub target_value: f64,
    pub target_value2: f64,         // For Between comparison
    pub editing_target_value: bool, // true = editing primary, false = editing secondary (for Between)
    pub target_input_mode: bool,    // true when user is typing a number for target
    pub target_edit_buffer: String,

    // Strategy builder condition list selection
    pub active_condition_panel: ConditionPanel,
    pub entry_condition_idx: usize,
    pub exit_condition_idx: usize,
    pub regime_condition_idx: usize,
    pub scale_in_condition_idx: usize,
    pub scale_out_condition_idx: usize,

    // Available data
    pub presets: Vec<StrategyPreset>,
    pub user_presets: Vec<UserStrategyPreset>,
    pub indicators: &'static [IndicatorDef],

    // Control
    pub should_quit: bool,
    pub confirmed: bool,

    // Save preset dialog
    pub save_preset_buffer: String,
    pub save_preset_error: Option<String>,

    // Optimizer setup state
    pub optimizer_params: Vec<OptimizerParamDef>,
    pub optimizer_param_idx: usize,
    /// Which sub-field is selected (see OPTIMIZER_FIELD_* constants)
    pub optimizer_field_idx: usize,
    pub optimizer_metric_idx: usize,
    pub optimizer_search_method: crate::backtest::types::SearchMethod,
    pub optimizer_walk_forward: bool,
    pub optimizer_in_sample: usize,
    pub optimizer_oos: usize,
    /// true = run with optimizer, false = run normal backtest
    pub run_with_optimizer: bool,

    // Ensemble compose state
    pub ensemble_cursor_idx: usize,
    pub ensemble_selected: Vec<usize>,
    pub ensemble_weights: HashMap<usize, f64>,
    pub ensemble_mode: EnsembleModeChoice,
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
            building_htf_interval: None,
            param_values: Vec::new(),
            target_value: 0.0,
            target_value2: 0.0,
            editing_target_value: true,
            target_input_mode: false,
            target_edit_buffer: String::new(),
            active_condition_panel: ConditionPanel::Entry,
            entry_condition_idx: 0,
            exit_condition_idx: 0,
            regime_condition_idx: 0,
            scale_in_condition_idx: 0,
            scale_out_condition_idx: 0,
            presets: StrategyPreset::all(),
            user_presets: user_presets::load_user_presets(),
            indicators: IndicatorDef::all(),
            should_quit: false,
            confirmed: false,
            save_preset_buffer: String::new(),
            save_preset_error: None,
            optimizer_params: Vec::new(),
            optimizer_param_idx: 0,
            optimizer_field_idx: 0,
            optimizer_metric_idx: 0,
            optimizer_search_method: crate::backtest::types::SearchMethod::Grid,
            optimizer_walk_forward: false,
            optimizer_in_sample: WALK_FORWARD_IN_SAMPLE_BARS,
            optimizer_oos: WALK_FORWARD_OOS_BARS,
            run_with_optimizer: false,
            ensemble_cursor_idx: 0,
            ensemble_selected: Vec::new(),
            ensemble_weights: HashMap::new(),
            ensemble_mode: EnsembleModeChoice::WeightedMajority,
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
        self.indicators
            .iter()
            .filter(|i| i.category == category)
            .count()
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
            ConfigField::CostProfile => detect_cost_profile(&self.config).name().to_string(),
            ConfigField::Commission => format!("{:.2}%", self.config.commission * 100.0),
            ConfigField::CommissionFlat => format!("${:.2}", self.config.commission_flat),
            ConfigField::Slippage => format!("{:.2}%", self.config.slippage * 100.0),
            ConfigField::SpreadPct => format!("{:.2}%", self.config.spread_pct * 100.0),
            ConfigField::TransactionTaxPct => {
                format!("{:.2}%", self.config.transaction_tax_pct * 100.0)
            }
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
            ConfigField::TrailingStop => self
                .config
                .trailing_stop
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::PositionSize => format!("{:.0}%", self.config.position_size * 100.0),
            ConfigField::MaxPositions => {
                if self.config.max_positions == 0 {
                    "Unlimited".to_string()
                } else {
                    self.config.max_positions.to_string()
                }
            }
            ConfigField::WarmupBars => {
                if self.config.strategy.warmup_bars == 0 {
                    "None".to_string()
                } else {
                    self.config.strategy.warmup_bars.to_string()
                }
            }
            ConfigField::RiskFreeRate => format!("{:.1}%", self.config.risk_free_rate * 100.0),
            ConfigField::MinSignalStrength => {
                format!("{:.1}%", self.config.min_signal_strength * 100.0)
            }
            ConfigField::CloseAtEnd => format!("{}", self.config.close_at_end),
            ConfigField::BarsPerYear => {
                if (self.config.bars_per_year - self.config.bars_per_year.round()).abs() < 1e-6 {
                    format!("{:.0}", self.config.bars_per_year)
                } else {
                    format!("{:.2}", self.config.bars_per_year)
                }
            }
            ConfigField::ReinvestDividends => format!("{}", self.config.reinvest_dividends),
            ConfigField::Benchmark => self
                .config
                .benchmark
                .clone()
                .unwrap_or_else(|| "None".to_string()),
            ConfigField::EntryOrderType => self.config.strategy.entry_order_type.name().to_string(),
            ConfigField::EntryPriceOffset => {
                format!(
                    "{:.2}%",
                    self.config.strategy.entry_price_offset_pct * 100.0
                )
            }
            ConfigField::EntryStopLimitGap => {
                format!(
                    "{:.2}%",
                    self.config.strategy.entry_stop_limit_gap_pct * 100.0
                )
            }
            ConfigField::EntryExpiryBars => match self.config.strategy.entry_expires_bars {
                None | Some(0) => "GTC".to_string(),
                Some(n) => format!("{} bars", n),
            },
            ConfigField::EntryBracketSL => self
                .config
                .strategy
                .entry_bracket_sl
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::EntryBracketTP => self
                .config
                .strategy
                .entry_bracket_tp
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::EntryBracketTrail => self
                .config
                .strategy
                .entry_bracket_trail
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortOrderType => self.config.strategy.short_order_type.name().to_string(),
            ConfigField::ShortPriceOffset => {
                format!(
                    "{:.2}%",
                    self.config.strategy.short_price_offset_pct * 100.0
                )
            }
            ConfigField::ShortExpiryBars => match self.config.strategy.short_expires_bars {
                None | Some(0) => "GTC".to_string(),
                Some(n) => format!("{} bars", n),
            },
            ConfigField::ShortBracketSL => self
                .config
                .strategy
                .short_bracket_sl
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortBracketTP => self
                .config
                .strategy
                .short_bracket_tp
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::ShortBracketTrail => self
                .config
                .strategy
                .short_bracket_trail
                .map(|v| format!("{:.1}%", v * 100.0))
                .unwrap_or_else(|| "Global".to_string()),
            ConfigField::PortfolioSymbols => {
                if self.config.portfolio_symbols.is_empty() {
                    "(single-symbol mode)".to_string()
                } else {
                    self.config.portfolio_symbols.join(", ")
                }
            }
            ConfigField::RebalanceMode => self.config.rebalance_mode.name().to_string(),
        }
    }

    pub fn start_editing(&mut self) {
        self.editing = true;
        self.edit_error = None;
        let field = self.current_config_field();

        // Handle enum-cycle fields that don't use text input
        match field {
            ConfigField::CostProfile => {
                let current = detect_cost_profile(&self.config);
                apply_cost_profile(&mut self.config, current.next());
                self.editing = false;
                return;
            }
            ConfigField::EntryOrderType => {
                self.config.strategy.entry_order_type =
                    self.config.strategy.entry_order_type.cycle();
                self.editing = false;
                return;
            }
            ConfigField::ShortOrderType => {
                self.config.strategy.short_order_type =
                    self.config.strategy.short_order_type.cycle();
                self.editing = false;
                return;
            }
            ConfigField::RebalanceMode => {
                self.config.rebalance_mode = self.config.rebalance_mode.cycle();
                self.editing = false;
                return;
            }
            _ => {}
        }

        self.edit_buffer = match field {
            ConfigField::Symbol => self.config.symbol.clone(),
            ConfigField::Interval => interval_to_string(self.config.interval),
            ConfigField::Range => range_to_string(self.config.range),
            ConfigField::Capital => format!("{}", self.config.capital),
            ConfigField::CostProfile => String::new(),
            ConfigField::Commission => format!("{}", self.config.commission * 100.0),
            ConfigField::CommissionFlat => format!("{}", self.config.commission_flat),
            ConfigField::Slippage => format!("{}", self.config.slippage * 100.0),
            ConfigField::SpreadPct => format!("{}", self.config.spread_pct * 100.0),
            ConfigField::TransactionTaxPct => {
                format!("{}", self.config.transaction_tax_pct * 100.0)
            }
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
            ConfigField::TrailingStop => self
                .config
                .trailing_stop
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::PositionSize => format!("{}", self.config.position_size * 100.0),
            ConfigField::MaxPositions => format!("{}", self.config.max_positions),
            ConfigField::WarmupBars => format!("{}", self.config.strategy.warmup_bars),
            ConfigField::RiskFreeRate => format!("{}", self.config.risk_free_rate * 100.0),
            ConfigField::MinSignalStrength => {
                format!("{}", self.config.min_signal_strength * 100.0)
            }
            ConfigField::CloseAtEnd => format!("{}", self.config.close_at_end),
            ConfigField::BarsPerYear => format!("{}", self.config.bars_per_year),
            ConfigField::ReinvestDividends => format!("{}", self.config.reinvest_dividends),
            ConfigField::Benchmark => self.config.benchmark.clone().unwrap_or_default(),
            ConfigField::EntryOrderType
            | ConfigField::ShortOrderType
            | ConfigField::RebalanceMode => {
                // Already handled above by enum-cycle — unreachable
                String::new()
            }
            ConfigField::EntryPriceOffset => {
                format!("{}", self.config.strategy.entry_price_offset_pct * 100.0)
            }
            ConfigField::EntryStopLimitGap => {
                format!("{}", self.config.strategy.entry_stop_limit_gap_pct * 100.0)
            }
            ConfigField::EntryExpiryBars => self
                .config
                .strategy
                .entry_expires_bars
                .map(|n| n.to_string())
                .unwrap_or_default(),
            ConfigField::EntryBracketSL => self
                .config
                .strategy
                .entry_bracket_sl
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::EntryBracketTP => self
                .config
                .strategy
                .entry_bracket_tp
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::EntryBracketTrail => self
                .config
                .strategy
                .entry_bracket_trail
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortPriceOffset => {
                format!("{}", self.config.strategy.short_price_offset_pct * 100.0)
            }
            ConfigField::ShortExpiryBars => self
                .config
                .strategy
                .short_expires_bars
                .map(|n| n.to_string())
                .unwrap_or_default(),
            ConfigField::ShortBracketSL => self
                .config
                .strategy
                .short_bracket_sl
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortBracketTP => self
                .config
                .strategy
                .short_bracket_tp
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::ShortBracketTrail => self
                .config
                .strategy
                .short_bracket_trail
                .map(|v| format!("{}", v * 100.0))
                .unwrap_or_default(),
            ConfigField::PortfolioSymbols => self.config.portfolio_symbols.join(", "),
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
                    let interval = parse_interval(value)?;
                    self.config.interval = interval;
                    // Keep annualisation assumptions aligned with the selected interval
                    // until the user explicitly overrides Bars / Year.
                    self.config.bars_per_year = bars_per_year_for_interval(interval);
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
                ConfigField::CostProfile => {
                    // Handled in start_editing by cycling
                }
                ConfigField::Commission => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Commission cannot be negative".into(),
                        ));
                    }
                    self.config.commission = v / 100.0;
                }
                ConfigField::CommissionFlat => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Flat commission cannot be negative".into(),
                        ));
                    }
                    self.config.commission_flat = v;
                }
                ConfigField::Slippage => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v < 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Slippage cannot be negative".into(),
                        ));
                    }
                    self.config.slippage = v / 100.0;
                }
                ConfigField::SpreadPct => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Spread must be 0-100%".into(),
                        ));
                    }
                    self.config.spread_pct = v / 100.0;
                }
                ConfigField::TransactionTaxPct => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Transaction tax must be 0-100%".into(),
                        ));
                    }
                    self.config.transaction_tax_pct = v / 100.0;
                }
                ConfigField::MaxPositions => {
                    let v: usize = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument(
                            "Must be a whole number (0 = unlimited)".into(),
                        )
                    })?;
                    self.config.max_positions = v;
                }
                ConfigField::WarmupBars => {
                    let v: usize = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument(
                            "Must be a whole number (0 = no warmup)".into(),
                        )
                    })?;
                    self.config.strategy.warmup_bars = v;
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
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Stop loss must be between 0 and 100%".into(),
                            ));
                        }
                        // Reject if stop-loss is at or below round-trip costs.
                        // A stop at or below break-even means every stopped-out trade
                        // is guaranteed to lose money on fees alone.
                        // Include all friction sources: commission, slippage, spread (half
                        // applied each side → full spread round-trip), and transaction tax
                        // (buy-only, so counted once).
                        let round_trip_pct = (self.config.commission * 2.0
                            + self.config.slippage * 2.0
                            + self.config.spread_pct
                            + self.config.transaction_tax_pct)
                            * 100.0;
                        if v <= round_trip_pct {
                            return Err(crate::error::CliError::InvalidArgument(format!(
                                "Stop loss {v:.2}% ≤ round-trip cost {round_trip_pct:.2}% \
                                 (2× commission + 2× slippage + spread + transaction tax). \
                                 A stopped-out trade loses {v:.2}% plus {round_trip_pct:.2}% \
                                 in fees — no trade can profit after costs at this stop level."
                            )));
                        }
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
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Take profit must be between 0 and 1000%".into(),
                            ));
                        }
                        self.config.take_profit = Some(v / 100.0);
                    }
                }
                ConfigField::TrailingStop => {
                    if value.is_empty() {
                        self.config.trailing_stop = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Trailing stop must be 0-100%".into(),
                            ));
                        }
                        self.config.trailing_stop = Some(v / 100.0);
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
                ConfigField::RiskFreeRate => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Risk-free rate must be 0-100%".into(),
                        ));
                    }
                    self.config.risk_free_rate = v / 100.0;
                }
                ConfigField::MinSignalStrength => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Min signal strength must be 0-100%".into(),
                        ));
                    }
                    self.config.min_signal_strength = v / 100.0;
                }
                ConfigField::CloseAtEnd => {
                    self.config.close_at_end = parse_bool(value)?;
                }
                ConfigField::BarsPerYear => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if v <= 0.0 {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Bars per year must be positive".into(),
                        ));
                    }
                    self.config.bars_per_year = v;
                }
                ConfigField::ReinvestDividends => {
                    self.config.reinvest_dividends = parse_bool(value)?;
                }
                ConfigField::Benchmark => {
                    let sym = value.trim().to_uppercase();
                    self.config.benchmark = if sym.is_empty() { None } else { Some(sym) };
                }
                ConfigField::EntryOrderType => {
                    // Handled in start_editing by cycling
                }
                ConfigField::EntryPriceOffset => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Offset must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.entry_price_offset_pct = v / 100.0;
                }
                ConfigField::EntryStopLimitGap => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Gap must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.entry_stop_limit_gap_pct = v / 100.0;
                }
                ConfigField::EntryExpiryBars => {
                    if value.is_empty() {
                        self.config.strategy.entry_expires_bars = None;
                    } else {
                        let v: usize = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument(
                                "Entry expiry must be a whole number of bars (0 = GTC)".into(),
                            )
                        })?;
                        self.config.strategy.entry_expires_bars =
                            if v == 0 { None } else { Some(v) };
                    }
                }
                ConfigField::EntryBracketSL => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_sl = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_sl = Some(v / 100.0);
                    }
                }
                ConfigField::EntryBracketTP => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_tp = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-1000%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_tp = Some(v / 100.0);
                    }
                }
                ConfigField::EntryBracketTrail => {
                    if value.is_empty() {
                        self.config.strategy.entry_bracket_trail = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.entry_bracket_trail = Some(v / 100.0);
                    }
                }
                ConfigField::ShortOrderType => {
                    // Handled in start_editing by cycling
                }
                ConfigField::ShortPriceOffset => {
                    let v: f64 = value.parse().map_err(|_| {
                        crate::error::CliError::InvalidArgument("Invalid number".into())
                    })?;
                    if !(0.0..=100.0).contains(&v) {
                        return Err(crate::error::CliError::InvalidArgument(
                            "Offset must be 0-100%".into(),
                        ));
                    }
                    self.config.strategy.short_price_offset_pct = v / 100.0;
                }
                ConfigField::ShortExpiryBars => {
                    if value.is_empty() {
                        self.config.strategy.short_expires_bars = None;
                    } else {
                        let v: usize = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument(
                                "Short expiry must be a whole number of bars (0 = GTC)".into(),
                            )
                        })?;
                        self.config.strategy.short_expires_bars =
                            if v == 0 { None } else { Some(v) };
                    }
                }
                ConfigField::ShortBracketSL => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_sl = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_sl = Some(v / 100.0);
                    }
                }
                ConfigField::ShortBracketTP => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_tp = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 1000.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-1000%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_tp = Some(v / 100.0);
                    }
                }
                ConfigField::ShortBracketTrail => {
                    if value.is_empty() {
                        self.config.strategy.short_bracket_trail = None;
                    } else {
                        let v: f64 = value.parse().map_err(|_| {
                            crate::error::CliError::InvalidArgument("Invalid number".into())
                        })?;
                        if v <= 0.0 || v > 100.0 {
                            return Err(crate::error::CliError::InvalidArgument(
                                "Must be 0-100%".into(),
                            ));
                        }
                        self.config.strategy.short_bracket_trail = Some(v / 100.0);
                    }
                }
                ConfigField::PortfolioSymbols => {
                    // Parse comma-separated symbols, trim whitespace, uppercase each.
                    self.config.portfolio_symbols = value
                        .split(',')
                        .map(|s| s.trim().to_uppercase())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                ConfigField::RebalanceMode => {
                    // Handled in start_editing by cycling — unreachable via text editing.
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

    pub fn total_preset_count(&self) -> usize {
        self.presets.len() + self.user_presets.len()
    }

    pub fn is_user_preset(&self, idx: usize) -> bool {
        idx >= self.presets.len()
    }

    pub fn load_preset(&mut self, idx: usize) {
        let symbol = if self.config.symbol.is_empty() {
            None
        } else {
            Some(self.config.symbol.clone())
        };

        let Some((_, _, mut preset_config)) = self.preset_entry(idx) else {
            return;
        };

        if let Some(sym) = symbol {
            preset_config.symbol = sym;
        }
        self.config = preset_config;
    }

    /// Returns (name, description, config) for a preset index in the combined
    /// built-in + user list.
    pub fn preset_entry(&self, idx: usize) -> Option<(String, String, BacktestConfiguration)> {
        if idx < self.presets.len() {
            let preset = &self.presets[idx];
            Some((
                preset.name.to_string(),
                preset.description.to_string(),
                (preset.config)(),
            ))
        } else {
            let user_idx = idx - self.presets.len();
            self.user_presets.get(user_idx).map(|preset| {
                (
                    preset.name.clone(),
                    preset.description.clone(),
                    preset.config.clone(),
                )
            })
        }
    }

    pub fn toggle_ensemble_selection(&mut self, idx: usize) {
        if let Some(pos) = self.ensemble_selected.iter().position(|v| *v == idx) {
            self.ensemble_selected.remove(pos);
            self.ensemble_weights.remove(&idx);
        } else {
            self.ensemble_selected.push(idx);
            self.ensemble_weights.entry(idx).or_insert(1.0);
            self.ensemble_selected.sort_unstable();
            self.ensemble_selected.dedup();
        }
    }

    pub fn ensemble_weight_for(&self, idx: usize) -> f64 {
        self.ensemble_weights.get(&idx).copied().unwrap_or(1.0)
    }

    pub fn adjust_ensemble_weight(&mut self, idx: usize, delta: f64) {
        if !self.ensemble_selected.contains(&idx) {
            return;
        }
        let current = self.ensemble_weight_for(idx);
        let next = (current + delta).clamp(0.0, 10.0);
        self.ensemble_weights.insert(idx, next);
    }

    pub fn set_ensemble_weight(&mut self, idx: usize, weight: f64) -> Result<()> {
        if !self.ensemble_selected.contains(&idx) {
            return Err(crate::error::CliError::InvalidArgument(
                "Select the preset first (Space) before editing weight".into(),
            ));
        }
        if !weight.is_finite() || !(0.0..=10.0).contains(&weight) {
            return Err(crate::error::CliError::InvalidArgument(
                "Weight must be between 0.0 and 10.0".into(),
            ));
        }
        self.ensemble_weights.insert(idx, weight);
        Ok(())
    }

    /// Build ensemble config from selected presets and apply it as active config.
    /// Existing execution and risk settings (capital, commissions, slippage, etc.) are preserved;
    /// only the ensemble/strategy composition fields are updated.
    pub fn apply_selected_ensemble(&mut self) -> Result<()> {
        let mut selected = self.ensemble_selected.clone();
        selected.sort_unstable();
        selected.dedup();

        let had_optimizer = self.config.optimizer.is_some();

        if selected.len() < 2 {
            return Err(crate::error::CliError::InvalidArgument(
                "Select at least 2 presets to compose an ensemble".into(),
            ));
        }

        let mut members = Vec::with_capacity(selected.len());
        for idx in selected {
            let (name, _, cfg) = self.preset_entry(idx).ok_or_else(|| {
                crate::error::CliError::InvalidArgument("Invalid ensemble preset selection".into())
            })?;
            if cfg.strategy.entry_conditions.conditions.is_empty()
                || cfg.strategy.exit_conditions.conditions.is_empty()
            {
                return Err(crate::error::CliError::InvalidArgument(format!(
                    "Preset '{}' must have both entry and exit conditions",
                    name
                )));
            }
            let weight = self.ensemble_weight_for(idx);
            members.push(EnsembleMemberConfig {
                name,
                strategy: cfg.strategy,
                weight,
            });
        }

        // Only update ensemble/strategy fields — preserve the user's existing capital,
        // commission, slippage, risk-free rate, and all other execution/risk settings.
        self.config.ensemble = Some(EnsembleConfig {
            mode: self.ensemble_mode,
            members,
        });
        self.config.strategy.name = format!("Ensemble ({})", self.ensemble_mode.name());

        // Optimizer currently tunes one StrategyConfig parameter space.
        // Ensemble optimization can be added later with explicit semantics.
        self.config.optimizer = None;
        if had_optimizer {
            self.edit_error = Some(
                "Optimizer settings were cleared because ensembles are not optimizer-compatible yet"
                    .into(),
            );
        } else {
            self.edit_error = None;
        }

        Ok(())
    }

    pub fn reload_user_presets(&mut self) {
        self.user_presets = user_presets::load_user_presets();
    }

    pub fn select_indicator(&mut self) {
        // Clone indicator to avoid borrowing issues
        let ind = self.current_indicator().cloned();
        if let Some(ind) = ind {
            // Initialize parameter values with defaults
            let param_values: Vec<f64> = ind.params.iter().map(|p| p.default).collect();
            self.param_values = param_values.clone();
            self.param_idx = 0;
            self.building_htf_interval = None;
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

        self.push_screen(Screen::TargetConfig);
    }

    pub fn finish_condition(&mut self) {
        use super::types::BuiltCondition;

        if let (Some(ind), Some(comp)) = (
            self.building_indicator.take(),
            self.building_comparison.take(),
        ) {
            let target = if comp.needs_range() {
                CompareTarget::Range(self.target_value, self.target_value2)
            } else {
                CompareTarget::Value(self.target_value)
            };

            // HTF scope is only valid when all compared values come from
            // precomputed indicators (price-action values are base-TF only).
            let htf_interval = self
                .building_htf_interval
                .filter(|_| indicator_supports_htf(&ind) && target_supports_htf(&target));

            let condition = BuiltCondition {
                indicator: ind,
                comparison: comp,
                target,
                htf_interval,
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
                ConditionTarget::Regime => {
                    self.config
                        .strategy
                        .regime_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ScaleIn => {
                    self.config
                        .strategy
                        .scale_in_conditions
                        .conditions
                        .push(condition);
                }
                ConditionTarget::ScaleOut => {
                    self.config
                        .strategy
                        .scale_out_conditions
                        .conditions
                        .push(condition);
                }
            }

            // Return to strategy builder
            self.building_htf_interval = None;
            self.prev_screens.clear();
            self.screen = Screen::StrategyBuilder;
        }
    }

    pub fn available_htf_intervals(&self) -> Vec<Option<Interval>> {
        available_htf_intervals_for_base(self.config.interval)
    }

    pub fn cycle_building_htf_interval(&mut self) {
        if self
            .building_indicator
            .as_ref()
            .is_some_and(|ind| !indicator_supports_htf(ind))
        {
            self.building_htf_interval = None;
            return;
        }

        let options = self.available_htf_intervals();
        let current = self.building_htf_interval;
        let idx = options.iter().position(|opt| *opt == current).unwrap_or(0);
        self.building_htf_interval = options[(idx + 1) % options.len()];
    }

    pub fn can_run(&self) -> bool {
        if self.config.symbol.is_empty() {
            return false;
        }

        if let Some(ensemble) = &self.config.ensemble {
            if !self.config.portfolio_symbols.is_empty() {
                return false;
            }
            return ensemble.members.len() >= 2;
        }

        !self.config.strategy.entry_conditions.conditions.is_empty()
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
        "3mo" => Ok(Interval::ThreeMonths),
        _ => Err(crate::error::CliError::InvalidArgument(format!(
            "Invalid interval: {}. Valid: 1m, 5m, 15m, 30m, 1h, 1d, 1wk, 1mo, 3mo",
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

fn interval_rank(interval: Interval) -> usize {
    match interval {
        Interval::OneMinute => 0,
        Interval::FiveMinutes => 1,
        Interval::FifteenMinutes => 2,
        Interval::ThirtyMinutes => 3,
        Interval::OneHour => 4,
        Interval::OneDay => 5,
        Interval::OneWeek => 6,
        Interval::OneMonth => 7,
        Interval::ThreeMonths => 8,
    }
}

fn available_htf_intervals_for_base(base_interval: Interval) -> Vec<Option<Interval>> {
    let all = [
        Interval::OneMinute,
        Interval::FiveMinutes,
        Interval::FifteenMinutes,
        Interval::ThirtyMinutes,
        Interval::OneHour,
        Interval::OneDay,
        Interval::OneWeek,
        Interval::OneMonth,
        Interval::ThreeMonths,
    ];

    let base_rank = interval_rank(base_interval);
    let mut options = vec![None];
    options.extend(
        all.into_iter()
            .filter(|interval| interval_rank(*interval) > base_rank)
            .map(Some),
    );
    options
}

fn indicator_supports_htf(indicator: &BuiltIndicator) -> bool {
    !matches!(indicator.indicator.category, IndicatorCategory::PriceAction)
}

fn target_supports_htf(target: &CompareTarget) -> bool {
    match target {
        CompareTarget::Indicator(other) => indicator_supports_htf(other),
        CompareTarget::Value(_) | CompareTarget::Range(_, _) => true,
    }
}

const COST_EPS: f64 = 1e-9;

// Cost profiles are intentionally conservative defaults that are easy to reason
// about in the TUI and can be tweaked manually afterward.
const PROFILE_ZERO_COST: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.0, 0.0);
const PROFILE_IBKR_US: (f64, f64, f64, f64, f64) = (0.0005, 0.35, 0.0002, 0.0001, 0.0);
const PROFILE_COMMISSION_FREE: (f64, f64, f64, f64, f64) = (0.0, 0.0, 0.0005, 0.0004, 0.0);
const PROFILE_UK_STAMP_DUTY: (f64, f64, f64, f64, f64) = (0.0005, 0.0, 0.0005, 0.0002, 0.005);

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() <= COST_EPS
}

fn matches_cost_tuple(config: &BacktestConfiguration, tuple: (f64, f64, f64, f64, f64)) -> bool {
    approx_eq(config.commission, tuple.0)
        && approx_eq(config.commission_flat, tuple.1)
        && approx_eq(config.slippage, tuple.2)
        && approx_eq(config.spread_pct, tuple.3)
        && approx_eq(config.transaction_tax_pct, tuple.4)
}

fn detect_cost_profile(config: &BacktestConfiguration) -> CostProfile {
    if matches_cost_tuple(config, PROFILE_ZERO_COST) {
        CostProfile::ZeroCost
    } else if matches_cost_tuple(config, PROFILE_IBKR_US) {
        CostProfile::InteractiveBrokersUs
    } else if matches_cost_tuple(config, PROFILE_COMMISSION_FREE) {
        CostProfile::CommissionFreeRetail
    } else if matches_cost_tuple(config, PROFILE_UK_STAMP_DUTY) {
        CostProfile::UkStampDuty
    } else {
        CostProfile::Custom
    }
}

fn apply_cost_profile(config: &mut BacktestConfiguration, profile: CostProfile) {
    let (commission, commission_flat, slippage, spread_pct, transaction_tax_pct) = match profile {
        CostProfile::Custom => {
            return;
        }
        CostProfile::ZeroCost => PROFILE_ZERO_COST,
        CostProfile::InteractiveBrokersUs => PROFILE_IBKR_US,
        CostProfile::CommissionFreeRetail => PROFILE_COMMISSION_FREE,
        CostProfile::UkStampDuty => PROFILE_UK_STAMP_DUTY,
    };

    config.commission = commission;
    config.commission_flat = commission_flat;
    config.slippage = slippage;
    config.spread_pct = spread_pct;
    config.transaction_tax_pct = transaction_tax_pct;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::types::OptimizeConfig;
    use crate::backtest::types::StrategyConfig;

    fn field_index(field: ConfigField) -> usize {
        ConfigField::all()
            .iter()
            .position(|f| *f == field)
            .expect("field should exist")
    }

    #[test]
    fn interval_edit_updates_default_bars_per_year() {
        let mut app = App::new(None);
        app.config.bars_per_year = 999.0;
        app.config_field_idx = field_index(ConfigField::Interval);
        app.edit_buffer = "1wk".to_string();

        app.finish_editing();

        assert_eq!(app.config.interval, Interval::OneWeek);
        assert!(
            (app.config.bars_per_year - bars_per_year_for_interval(Interval::OneWeek)).abs() < 1e-9
        );
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn min_signal_strength_parses_percent_to_fraction() {
        let mut app = App::new(None);
        app.config_field_idx = field_index(ConfigField::MinSignalStrength);
        app.edit_buffer = "25".to_string();

        app.finish_editing();

        assert!((app.config.min_signal_strength - 0.25).abs() < 1e-9);
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn close_at_end_parses_boolean() {
        let mut app = App::new(None);
        app.config.close_at_end = true;
        app.config_field_idx = field_index(ConfigField::CloseAtEnd);
        app.edit_buffer = "false".to_string();

        app.finish_editing();

        assert!(!app.config.close_at_end);
        assert!(app.edit_error.is_none());
    }

    #[test]
    fn bars_per_year_must_be_positive() {
        let mut app = App::new(None);
        app.config.bars_per_year = 252.0;
        app.config_field_idx = field_index(ConfigField::BarsPerYear);
        app.edit_buffer = "0".to_string();

        app.finish_editing();

        assert!((app.config.bars_per_year - 252.0).abs() < 1e-9);
        let err = app.edit_error.expect("validation error expected");
        assert!(err.contains("positive"));
    }

    #[test]
    fn htf_options_only_include_higher_intervals() {
        let options = available_htf_intervals_for_base(Interval::OneHour);
        assert_eq!(options[0], None);
        assert_eq!(
            options,
            vec![
                None,
                Some(Interval::OneDay),
                Some(Interval::OneWeek),
                Some(Interval::OneMonth),
                Some(Interval::ThreeMonths),
            ]
        );
    }

    #[test]
    fn cycle_htf_interval_wraps_to_none() {
        let mut app = App::new(None);
        app.config.interval = Interval::OneMonth;

        app.cycle_building_htf_interval();
        assert_eq!(app.building_htf_interval, Some(Interval::ThreeMonths));

        app.cycle_building_htf_interval();
        assert_eq!(app.building_htf_interval, None);
    }

    #[test]
    fn price_action_indicator_cannot_cycle_htf_scope() {
        let mut app = App::new(None);
        app.config.interval = Interval::OneDay;
        app.building_indicator = Some(BuiltIndicator {
            indicator: IndicatorDef::find("close"),
            param_values: vec![],
            output: None,
        });

        app.cycle_building_htf_interval();

        assert_eq!(app.building_htf_interval, None);
    }

    #[test]
    fn finish_condition_drops_htf_for_price_action_indicator() {
        let mut app = App::new(None);
        app.condition_target = ConditionTarget::Entry;
        app.building_indicator = Some(BuiltIndicator {
            indicator: IndicatorDef::find("close"),
            param_values: vec![],
            output: None,
        });
        app.building_comparison = Some(ComparisonType::Above);
        app.target_value = 100.0;
        app.building_htf_interval = Some(Interval::OneWeek);

        app.finish_condition();

        let cond = app
            .config
            .strategy
            .entry_conditions
            .conditions
            .first()
            .expect("condition should be created");
        assert_eq!(cond.htf_interval, None);
    }

    #[test]
    fn detect_zero_cost_profile() {
        let mut cfg = BacktestConfiguration::default();
        apply_cost_profile(&mut cfg, CostProfile::ZeroCost);
        assert_eq!(detect_cost_profile(&cfg), CostProfile::ZeroCost);
    }

    #[test]
    fn apply_uk_stamp_duty_profile_sets_expected_fields() {
        let mut cfg = BacktestConfiguration::default();
        apply_cost_profile(&mut cfg, CostProfile::UkStampDuty);

        assert!(approx_eq(cfg.commission, PROFILE_UK_STAMP_DUTY.0));
        assert!(approx_eq(cfg.commission_flat, PROFILE_UK_STAMP_DUTY.1));
        assert!(approx_eq(cfg.slippage, PROFILE_UK_STAMP_DUTY.2));
        assert!(approx_eq(cfg.spread_pct, PROFILE_UK_STAMP_DUTY.3));
        assert!(approx_eq(cfg.transaction_tax_pct, PROFILE_UK_STAMP_DUTY.4));
    }

    #[test]
    fn can_run_allows_valid_ensemble_without_manual_conditions() {
        let mut app = App::new(None);
        app.config.symbol = "AAPL".to_string();
        app.config.strategy.entry_conditions.conditions.clear();
        app.config.strategy.exit_conditions.conditions.clear();
        app.config.ensemble = Some(EnsembleConfig {
            mode: EnsembleModeChoice::WeightedMajority,
            members: vec![
                EnsembleMemberConfig {
                    name: "A".to_string(),
                    strategy: StrategyConfig::default(),
                    weight: 1.0,
                },
                EnsembleMemberConfig {
                    name: "B".to_string(),
                    strategy: StrategyConfig::default(),
                    weight: 1.0,
                },
            ],
        });

        assert!(app.can_run());
    }

    #[test]
    fn apply_selected_ensemble_rejects_member_without_conditions() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.user_presets.push(UserStrategyPreset {
            name: "Broken".to_string(),
            description: "No conditions".to_string(),
            config: BacktestConfiguration::default(),
        });

        let invalid_idx = app.total_preset_count() - 1;
        app.ensemble_selected = vec![0, invalid_idx];

        let err = app
            .apply_selected_ensemble()
            .expect_err("should reject invalid member preset");
        assert!(err.to_string().contains("Broken"));
    }

    #[test]
    fn apply_selected_ensemble_clears_optimizer_with_warning() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.config.optimizer = Some(OptimizeConfig::default());
        app.ensemble_selected = vec![0, 1];

        app.apply_selected_ensemble()
            .expect("ensemble composition should succeed");

        assert!(app.config.optimizer.is_none());
        let warning = app
            .edit_error
            .clone()
            .expect("optimizer warning should be surfaced");
        assert!(warning.contains("Optimizer settings were cleared"));
    }

    #[test]
    fn ensemble_toggle_sets_and_clears_weight_defaults() {
        let mut app = App::new(None);
        app.toggle_ensemble_selection(1);
        assert!(app.ensemble_selected.contains(&1));
        assert!((app.ensemble_weight_for(1) - 1.0).abs() < 1e-9);

        app.toggle_ensemble_selection(1);
        assert!(!app.ensemble_selected.contains(&1));
        assert!(!app.ensemble_weights.contains_key(&1));
    }

    #[test]
    fn apply_selected_ensemble_uses_custom_member_weights() {
        let mut app = App::new(Some("AAPL".to_string()));
        app.ensemble_selected = vec![0, 1];
        app.ensemble_weights.insert(0, 1.25);
        app.ensemble_weights.insert(1, 0.75);

        app.apply_selected_ensemble()
            .expect("ensemble composition should succeed");

        let ensemble = app
            .config
            .ensemble
            .as_ref()
            .expect("ensemble should be set");
        assert_eq!(ensemble.members.len(), 2);
        assert!((ensemble.members[0].weight - 1.25).abs() < 1e-9);
        assert!((ensemble.members[1].weight - 0.75).abs() < 1e-9);
    }

    #[test]
    fn set_ensemble_weight_validates_selection_and_range() {
        let mut app = App::new(None);

        let err = app
            .set_ensemble_weight(0, 1.5)
            .expect_err("selection guard should fail");
        assert!(err.to_string().contains("Select the preset first"));

        app.ensemble_selected.push(0);
        app.set_ensemble_weight(0, 2.5)
            .expect("valid weight should be accepted");
        assert!((app.ensemble_weight_for(0) - 2.5).abs() < 1e-9);

        let err = app
            .set_ensemble_weight(0, 10.5)
            .expect_err("range guard should fail");
        assert!(err.to_string().contains("between 0.0 and 10.0"));
    }
}
