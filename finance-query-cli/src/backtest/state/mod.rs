mod conditions;
mod editing;
mod ensemble;
mod helpers;
mod presets;

pub use helpers::{interval_to_string, range_to_string};

use super::indicators::{IndicatorCategory, IndicatorDef};
use super::presets::StrategyPreset;
use super::types::{
    BacktestConfiguration, BuiltIndicator, ComparisonType, EnsembleModeChoice, OptimizerParamDef,
    WALK_FORWARD_IN_SAMPLE_BARS, WALK_FORWARD_OOS_BARS,
};
use super::user_presets::{self, UserStrategyPreset};
use finance_query::Interval;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::types::{EnsembleConfig, EnsembleMemberConfig, StrategyConfig};

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
}
