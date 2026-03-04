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
    /// Operator to use when combining with the NEXT condition (ignored for last condition)
    pub next_op: LogicalOp,
}

impl BuiltCondition {
    pub fn display(&self) -> String {
        let ind = self.indicator.display_name();
        match &self.target {
            CompareTarget::Value(v) => {
                format!("{} {} {:.2}", ind, self.comparison.symbol(), v)
            }
            CompareTarget::Range(low, high) => {
                format!("{:.2} < {} < {:.2}", low, ind, high)
            }
            CompareTarget::Indicator(other) => {
                format!(
                    "{} {} {}",
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
        }
    }
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
    pub allow_short: bool,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub trailing_stop: Option<f64>,
    pub position_size: f64,
    pub risk_free_rate: f64,
    pub reinvest_dividends: bool,
    pub benchmark: Option<String>,
    pub strategy: StrategyConfig,
    pub optimizer: Option<OptimizeConfig>,
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
            allow_short: false,
            stop_loss: Some(0.05),
            take_profit: Some(0.10),
            trailing_stop: None,
            position_size: 1.0,
            risk_free_rate: 0.0,
            reinvest_dividends: false,
            benchmark: None,
            strategy: StrategyConfig::new(),
            optimizer: None,
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

        let groups: [Option<(&ConditionGroup, usize)>; 4] = [
            Some((&strategy.entry_conditions, 0)),
            Some((&strategy.exit_conditions, 1)),
            short_entry_ref.map(|g| (g, 2)),
            short_exit_ref.map(|g| (g, 3)),
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
                        _ => "short_exit",
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

/// Configuration for the parameter optimizer.
#[derive(Debug, Clone)]
pub struct OptimizeConfig {
    pub params: Vec<OptimizerParamDef>,
    pub metric: OptimizeMetric,
    pub walk_forward: bool,
    pub in_sample_bars: usize,
    pub out_of_sample_bars: usize,
}

impl Default for OptimizeConfig {
    fn default() -> Self {
        Self {
            params: Vec::new(),
            metric: OptimizeMetric::SharpeRatio,
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
