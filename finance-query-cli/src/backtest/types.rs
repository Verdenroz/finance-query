use super::indicators::IndicatorDef;
use finance_query::backtesting::OptimizeMetric;
use finance_query::{Interval, TimeRange};

/// Types of comparisons available for conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonType {
    Above,
    Below,
    CrossesAbove,
    CrossesBelow,
    Between,
    Equals,
}

impl ComparisonType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Above,
            Self::Below,
            Self::CrossesAbove,
            Self::CrossesBelow,
            Self::Between,
            Self::Equals,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Above => "Above",
            Self::Below => "Below",
            Self::CrossesAbove => "Crosses Above",
            Self::CrossesBelow => "Crosses Below",
            Self::Between => "Between",
            Self::Equals => "Equals",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Above => "Value is greater than threshold",
            Self::Below => "Value is less than threshold",
            Self::CrossesAbove => "Value crosses from below to above threshold",
            Self::CrossesBelow => "Value crosses from above to below threshold",
            Self::Between => "Value is between two thresholds",
            Self::Equals => "Value equals threshold (within tolerance)",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Above => ">",
            Self::Below => "<",
            Self::CrossesAbove => "↗",
            Self::CrossesBelow => "↘",
            Self::Between => "↔",
            Self::Equals => "=",
        }
    }

    /// Number of threshold values needed for this comparison type
    pub fn threshold_count(&self) -> usize {
        match self {
            Self::Between => 2,
            _ => 1,
        }
    }

    /// Whether this comparison requires two threshold values
    pub fn needs_range(&self) -> bool {
        self.threshold_count() == 2
    }
}

/// What to compare the indicator against
#[derive(Debug, Clone)]
pub enum CompareTarget {
    /// Compare against a fixed value
    Value(f64),
    /// Compare against two values (for Between)
    Range(f64, f64),
    /// Compare against another indicator
    Indicator(BuiltIndicator),
}

/// A configured indicator with parameter values
#[derive(Debug, Clone)]
pub struct BuiltIndicator {
    pub indicator: IndicatorDef,
    pub param_values: Vec<f64>,
    /// For multi-output indicators like MACD, which output to use
    pub output: Option<String>,
}

impl BuiltIndicator {
    pub fn display_name(&self) -> String {
        let params_str = self
            .param_values
            .iter()
            .map(|v| {
                if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else {
                    format!("{:.1}", v)
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        if params_str.is_empty() {
            self.indicator.name.to_string()
        } else {
            format!("{}({})", self.indicator.name, params_str)
        }
    }
}

/// A single condition in a strategy
#[derive(Debug, Clone)]
pub struct BuiltCondition {
    pub indicator: BuiltIndicator,
    pub comparison: ComparisonType,
    pub target: CompareTarget,
    /// Optional higher-timeframe scope for this condition.
    /// When set, indicator lookups use precomputed stretched HTF arrays.
    pub htf_interval: Option<Interval>,
    /// Operator to use when combining with the NEXT condition (ignored for last condition)
    pub next_op: LogicalOp,
}

impl BuiltCondition {
    pub fn display(&self) -> String {
        let ind = self.indicator.display_name();
        let scope = self
            .htf_interval
            .map(|interval| format!("[{}] ", interval.as_str()))
            .unwrap_or_default();
        match &self.target {
            CompareTarget::Value(v) => {
                format!("{}{} {} {:.2}", scope, ind, self.comparison.symbol(), v)
            }
            CompareTarget::Range(low, high) => {
                format!("{}{:.2} < {} < {:.2}", scope, low, ind, high)
            }
            CompareTarget::Indicator(other) => {
                format!(
                    "{}{} {} {}",
                    scope,
                    ind,
                    self.comparison.symbol(),
                    other.display_name()
                )
            }
        }
    }
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

impl LogicalOp {
    pub fn name(&self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::And => Self::Or,
            Self::Or => Self::And,
        }
    }
}

/// Entry order type for long positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LongOrderType {
    /// Standard market order — fills at next bar's open (default).
    #[default]
    Market,
    /// Limit buy — fills if price dips X% below the signal close.
    LimitBelow,
    /// Stop buy (breakout) — fills if price breaks X% above the signal close.
    StopAbove,
    /// Stop-limit buy — triggers at stop price (X% above close) and fills
    /// only if the fill price does not exceed the limit (stop + gap%).
    StopLimitAbove,
}

impl LongOrderType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Market => "Market",
            Self::LimitBelow => "Limit Below",
            Self::StopAbove => "Stop Above",
            Self::StopLimitAbove => "Stop-Limit Above",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Market => Self::LimitBelow,
            Self::LimitBelow => Self::StopAbove,
            Self::StopAbove => Self::StopLimitAbove,
            Self::StopLimitAbove => Self::Market,
        }
    }

    pub fn needs_offset(self) -> bool {
        !matches!(self, Self::Market)
    }

    /// Whether this order type requires the stop-limit gap field.
    pub fn needs_gap(self) -> bool {
        matches!(self, Self::StopLimitAbove)
    }
}

/// Entry order type for short positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShortOrderType {
    /// Standard market order — fills at next bar's open (default).
    #[default]
    Market,
    /// Limit sell — fills if price rallies X% above the signal close.
    LimitAbove,
    /// Stop sell (breakdown) — fills if price breaks X% below the signal close.
    StopBelow,
}

impl ShortOrderType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Market => "Market",
            Self::LimitAbove => "Limit Above",
            Self::StopBelow => "Stop Below",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Market => Self::LimitAbove,
            Self::LimitAbove => Self::StopBelow,
            Self::StopBelow => Self::Market,
        }
    }

    pub fn needs_offset(self) -> bool {
        !matches!(self, Self::Market)
    }
}

/// A group of conditions - each condition stores its own operator for combining with the next
#[derive(Debug, Clone, Default)]
pub struct ConditionGroup {
    pub conditions: Vec<BuiltCondition>,
}

impl ConditionGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display(&self) -> String {
        if self.conditions.is_empty() {
            return "(no conditions)".to_string();
        }

        let mut result = String::new();
        for (i, cond) in self.conditions.iter().enumerate() {
            if i > 0 {
                // Use the previous condition's next_op
                result.push_str(&format!(" {} ", self.conditions[i - 1].next_op.name()));
            }
            result.push_str(&cond.display());
        }
        result
    }

    /// Toggle the operator for a specific condition (affects how it combines with the next)
    pub fn toggle_op_at(&mut self, idx: usize) {
        if idx < self.conditions.len() {
            self.conditions[idx].next_op = self.conditions[idx].next_op.toggle();
        }
    }

    /// Delete a condition at a specific index
    pub fn remove_at(&mut self, idx: usize) {
        if idx < self.conditions.len() {
            self.conditions.remove(idx);
        }
    }
}

/// Full strategy configuration built by the TUI
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub name: String,
    pub entry_conditions: ConditionGroup,
    pub exit_conditions: ConditionGroup,
    pub short_entry_conditions: Option<ConditionGroup>,
    pub short_exit_conditions: Option<ConditionGroup>,
    /// Regime filter: entry signals are suppressed unless ALL regime conditions pass.
    pub regime_conditions: ConditionGroup,
    /// Number of bars to skip at the start before generating signals.
    pub warmup_bars: usize,
    /// Conditions that trigger a scale-in (pyramid) while in a position.
    pub scale_in_conditions: ConditionGroup,
    /// Fraction of current portfolio equity to add when scale-in fires (0.0–1.0).
    pub scale_in_fraction: f64,
    /// Conditions that trigger a partial exit (scale-out) while in a position.
    pub scale_out_conditions: ConditionGroup,
    /// Fraction of the current position quantity to close when scale-out fires (0.0–1.0).
    pub scale_out_fraction: f64,
    /// Entry order type for long positions.
    pub entry_order_type: LongOrderType,
    /// Price offset as a fraction (e.g. 0.005 = 0.5%) for limit/stop long entry orders.
    /// The limit price = close * (1 - offset) for LimitBelow,
    /// or close * (1 + offset) for StopAbove / StopLimitAbove (stop trigger).
    pub entry_price_offset_pct: f64,
    /// Gap above the stop trigger for StopLimitAbove orders (fraction).
    /// limit_price = stop_price * (1 + gap). Ignored for other order types.
    pub entry_stop_limit_gap_pct: f64,
    /// Bars a pending long entry order stays alive. None = Good-Till-Cancelled.
    pub entry_expires_bars: Option<usize>,
    /// Per-trade stop-loss override for long entries (fraction, e.g. 0.05 = 5%).
    /// When Some, overrides BacktestConfig::stop_loss_pct for this trade.
    pub entry_bracket_sl: Option<f64>,
    /// Per-trade take-profit override for long entries.
    pub entry_bracket_tp: Option<f64>,
    /// Per-trade trailing-stop override for long entries.
    pub entry_bracket_trail: Option<f64>,
    /// Entry order type for short positions.
    pub short_order_type: ShortOrderType,
    /// Price offset for short limit/stop entry orders.
    pub short_price_offset_pct: f64,
    /// Bars a pending short entry order stays alive. None = GTC.
    pub short_expires_bars: Option<usize>,
    /// Per-trade stop-loss override for short entries.
    pub short_bracket_sl: Option<f64>,
    /// Per-trade take-profit override for short entries.
    pub short_bracket_tp: Option<f64>,
    /// Per-trade trailing-stop override for short entries.
    pub short_bracket_trail: Option<f64>,
}

impl StrategyConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "Custom Strategy".to_string(),
            entry_conditions: ConditionGroup::default(),
            exit_conditions: ConditionGroup::default(),
            short_entry_conditions: None,
            short_exit_conditions: None,
            regime_conditions: ConditionGroup::default(),
            warmup_bars: 0,
            scale_in_conditions: ConditionGroup::default(),
            scale_in_fraction: 0.25,
            scale_out_conditions: ConditionGroup::default(),
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

/// Rebalance mode for portfolio backtesting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RebalanceModeChoice {
    /// Each symbol uses `position_size_pct` of available cash at entry time.
    #[default]
    AvailableCapital,
    /// Capital is split equally among all configured symbols.
    EqualWeight,
}

impl RebalanceModeChoice {
    pub fn name(self) -> &'static str {
        match self {
            Self::AvailableCapital => "Available Capital",
            Self::EqualWeight => "Equal Weight",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::AvailableCapital => Self::EqualWeight,
            Self::EqualWeight => Self::AvailableCapital,
        }
    }
}

/// Voting mode for composed ensemble strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnsembleModeChoice {
    #[default]
    WeightedMajority,
    Unanimous,
    AnySignal,
    StrongestSignal,
}

impl EnsembleModeChoice {
    pub fn name(self) -> &'static str {
        match self {
            Self::WeightedMajority => "Weighted Majority",
            Self::Unanimous => "Unanimous",
            Self::AnySignal => "Any Signal",
            Self::StrongestSignal => "Strongest Signal",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::WeightedMajority => Self::Unanimous,
            Self::Unanimous => Self::AnySignal,
            Self::AnySignal => Self::StrongestSignal,
            Self::StrongestSignal => Self::WeightedMajority,
        }
    }
}

/// A single member strategy in an ensemble.
#[derive(Debug, Clone)]
pub struct EnsembleMemberConfig {
    pub name: String,
    pub strategy: StrategyConfig,
    pub weight: f64,
}

/// Optional ensemble composition used instead of a single dynamic strategy.
#[derive(Debug, Clone)]
pub struct EnsembleConfig {
    pub mode: EnsembleModeChoice,
    pub members: Vec<EnsembleMemberConfig>,
}

/// Full backtest configuration
#[derive(Debug, Clone)]
pub struct BacktestConfiguration {
    pub symbol: String,
    pub interval: Interval,
    pub range: TimeRange,
    pub capital: f64,
    pub commission: f64,
    pub commission_flat: f64,
    pub slippage: f64,
    pub spread_pct: f64,
    pub transaction_tax_pct: f64,
    pub allow_short: bool,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub trailing_stop: Option<f64>,
    pub position_size: f64,
    /// Max concurrent positions. 0 = unlimited.
    pub max_positions: usize,
    pub risk_free_rate: f64,
    /// Require signal strength threshold to trigger trades (0.0 - 1.0).
    pub min_signal_strength: f64,
    /// Close any open position at the final bar.
    pub close_at_end: bool,
    /// Bars per calendar year used to annualise return/risk metrics.
    pub bars_per_year: f64,
    pub reinvest_dividends: bool,
    pub benchmark: Option<String>,
    /// Optional ensemble strategy composed from multiple preset strategies.
    pub ensemble: Option<EnsembleConfig>,
    pub strategy: StrategyConfig,
    pub optimizer: Option<OptimizeConfig>,
    /// Extra symbols for portfolio mode. When non-empty the portfolio engine
    /// runs all symbols concurrently with a shared capital pool.
    /// The primary `symbol` is included automatically.
    pub portfolio_symbols: Vec<String>,
    /// Capital allocation strategy across portfolio symbols.
    pub rebalance_mode: RebalanceModeChoice,
    /// Max fraction of initial capital allocated to a single symbol (0.0 = no limit).
    pub max_allocation_per_symbol: f64,
}

/// Number of trading days in a calendar year (standard for annualised metrics).
pub const TRADING_DAYS_PER_YEAR: f64 = 252.0;

/// Returns the number of bars per calendar year for a given interval.
/// Used to annualise performance metrics (Sharpe, Sortino, etc.).
pub fn bars_per_year_for_interval(interval: Interval) -> f64 {
    match interval {
        Interval::OneMinute => TRADING_DAYS_PER_YEAR * 390.0,
        Interval::FiveMinutes => TRADING_DAYS_PER_YEAR * 78.0,
        Interval::FifteenMinutes => TRADING_DAYS_PER_YEAR * 26.0,
        Interval::ThirtyMinutes => TRADING_DAYS_PER_YEAR * 13.0,
        Interval::OneHour => TRADING_DAYS_PER_YEAR * 6.5,
        Interval::OneDay => TRADING_DAYS_PER_YEAR,
        Interval::OneWeek => 52.0,
        Interval::OneMonth => 12.0,
        Interval::ThreeMonths => 4.0,
    }
}
/// Default walk-forward in-sample window (one trading year).
pub const WALK_FORWARD_IN_SAMPLE_BARS: usize = TRADING_DAYS_PER_YEAR as usize;
/// Default walk-forward out-of-sample window (one trading quarter).
pub const WALK_FORWARD_OOS_BARS: usize = 63;
/// Default per-trade commission as a fraction of trade value.
pub const DEFAULT_COMMISSION_PCT: f64 = 0.001;
/// Default per-trade slippage as a fraction of trade value.
pub const DEFAULT_SLIPPAGE_PCT: f64 = 0.001;

impl Default for BacktestConfiguration {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            interval: Interval::OneDay,
            range: TimeRange::OneYear,
            capital: 10_000.0,
            commission: DEFAULT_COMMISSION_PCT,
            commission_flat: 0.0,
            slippage: DEFAULT_SLIPPAGE_PCT,
            spread_pct: 0.0,
            transaction_tax_pct: 0.0,
            allow_short: false,
            stop_loss: Some(0.05),
            take_profit: Some(0.10),
            trailing_stop: None,
            position_size: 1.0,
            max_positions: 1,
            risk_free_rate: 0.0,
            min_signal_strength: 0.0,
            close_at_end: true,
            bars_per_year: bars_per_year_for_interval(Interval::OneDay),
            reinvest_dividends: false,
            benchmark: None,
            ensemble: None,
            strategy: StrategyConfig::new(),
            optimizer: None,
            portfolio_symbols: Vec::new(),
            rebalance_mode: RebalanceModeChoice::default(),
            max_allocation_per_symbol: 0.0,
        }
    }
}

// ── Optimizer config ─────────────────────────────────────────────────────────

/// A single named parameter range for the grid-search optimizer.
#[derive(Debug, Clone)]
pub struct OptimizerParamDef {
    /// Display name (e.g. "SMA fast period")
    pub name: String,
    /// Condition group: 0 = entry, 1 = exit, 2 = short_entry, 3 = short_exit
    pub group: usize,
    /// Index of the condition within the group
    pub condition_idx: usize,
    /// Index within `BuiltIndicator::param_values`
    pub param_idx: usize,
    /// Range start value
    pub start: f64,
    /// Range end value (inclusive)
    pub end: f64,
    /// Step between values
    pub step: f64,
    /// Whether this param is currently enabled for optimization
    pub enabled: bool,
}

impl OptimizerParamDef {
    /// Extract all optimizable params from a strategy's conditions.
    ///
    /// Uses `ParamDef::min`, `max`, `step` from the indicator definition as
    /// default ranges, so the user has sensible starting values.
    pub fn from_strategy(strategy: &StrategyConfig) -> Vec<Self> {
        let mut params = Vec::new();

        let short_entry_ref = strategy.short_entry_conditions.as_ref();
        let short_exit_ref = strategy.short_exit_conditions.as_ref();

        let groups: [Option<(&ConditionGroup, usize)>; 7] = [
            Some((&strategy.entry_conditions, 0)),
            Some((&strategy.exit_conditions, 1)),
            short_entry_ref.map(|g| (g, 2)),
            short_exit_ref.map(|g| (g, 3)),
            Some((&strategy.scale_in_conditions, 4)),
            Some((&strategy.scale_out_conditions, 5)),
            Some((&strategy.regime_conditions, 6)),
        ];

        for entry in groups.into_iter().flatten() {
            let (group, group_idx) = entry;
            for (cond_idx, cond) in group.conditions.iter().enumerate() {
                let ind = &cond.indicator;
                for (param_idx, param_def) in ind.indicator.params.iter().enumerate() {
                    let group_label = match group_idx {
                        0 => "entry",
                        1 => "exit",
                        2 => "short_entry",
                        3 => "short_exit",
                        4 => "scale_in",
                        5 => "scale_out",
                        _ => "regime",
                    };
                    let name = format!(
                        "{} {} c{} {}",
                        group_label,
                        ind.indicator.name,
                        cond_idx + 1,
                        param_def.name,
                    );
                    params.push(OptimizerParamDef {
                        name,
                        group: group_idx,
                        condition_idx: cond_idx,
                        param_idx,
                        start: param_def.min,
                        end: param_def.max,
                        step: if param_def.step > 0.0 {
                            param_def.step
                        } else {
                            0.1
                        },
                        enabled: true,
                    });
                }
            }
        }

        params
    }
}

/// Whether to use grid search or Bayesian/SAMBO optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMethod {
    #[default]
    Grid,
    Bayesian,
}

impl SearchMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Grid => "Grid Search (exhaustive)",
            Self::Bayesian => "Bayesian/SAMBO (adaptive)",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Grid => Self::Bayesian,
            Self::Bayesian => Self::Grid,
        }
    }
}

/// Configuration for the parameter optimizer.
#[derive(Debug, Clone)]
pub struct OptimizeConfig {
    pub params: Vec<OptimizerParamDef>,
    pub metric: OptimizeMetric,
    pub search_method: SearchMethod,
    /// Max evaluations for Bayesian search (ignored by Grid).
    pub bayesian_trials: usize,
    pub walk_forward: bool,
    pub in_sample_bars: usize,
    pub out_of_sample_bars: usize,
}

impl Default for OptimizeConfig {
    fn default() -> Self {
        Self {
            params: Vec::new(),
            metric: OptimizeMetric::SharpeRatio,
            search_method: SearchMethod::Grid,
            bayesian_trials: 100,
            walk_forward: false,
            in_sample_bars: 252,
            out_of_sample_bars: 63,
        }
    }
}

pub fn all_optimize_metrics() -> Vec<OptimizeMetric> {
    vec![
        OptimizeMetric::SharpeRatio,
        OptimizeMetric::TotalReturn,
        OptimizeMetric::SortinoRatio,
        OptimizeMetric::CalmarRatio,
        OptimizeMetric::ProfitFactor,
        OptimizeMetric::WinRate,
        OptimizeMetric::MinDrawdown,
    ]
}

pub fn optimize_metric_label(m: OptimizeMetric) -> &'static str {
    match m {
        OptimizeMetric::SharpeRatio => "Sharpe Ratio",
        OptimizeMetric::TotalReturn => "Total Return",
        OptimizeMetric::SortinoRatio => "Sortino Ratio",
        OptimizeMetric::CalmarRatio => "Calmar Ratio",
        OptimizeMetric::ProfitFactor => "Profit Factor",
        OptimizeMetric::WinRate => "Win Rate",
        OptimizeMetric::MinDrawdown => "Min Drawdown",
        _ => "Unknown",
    }
}
