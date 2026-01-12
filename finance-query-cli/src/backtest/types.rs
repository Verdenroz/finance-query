use super::indicators::IndicatorDef;
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

    pub fn code_string(&self) -> String {
        let params_str = self
            .param_values
            .iter()
            .map(|v| {
                if v.fract() == 0.0 {
                    format!("{:.0}", v)
                } else {
                    format!("{:.2}", v)
                }
            })
            .collect::<Vec<_>>()
            .join(",");

        if let Some(ref output) = self.output {
            format!("{}({}).{}", self.indicator.code, params_str, output)
        } else if params_str.is_empty() {
            self.indicator.code.to_string()
        } else {
            format!("{}({})", self.indicator.code, params_str)
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
#[derive(Debug, Clone)]
pub struct ConditionGroup {
    pub conditions: Vec<BuiltCondition>,
}

impl ConditionGroup {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
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

impl Default for ConditionGroup {
    fn default() -> Self {
        Self::new()
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
        Self {
            name: "Custom Strategy".to_string(),
            entry_conditions: ConditionGroup::new(),
            exit_conditions: ConditionGroup::new(),
            short_entry_conditions: None,
            short_exit_conditions: None,
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self::new()
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
    pub slippage: f64,
    pub allow_short: bool,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub position_size: f64,
    pub strategy: StrategyConfig,
}

impl Default for BacktestConfiguration {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            interval: Interval::OneDay,
            range: TimeRange::OneYear,
            capital: 10_000.0,
            commission: 0.001,
            slippage: 0.001,
            allow_short: false,
            stop_loss: Some(0.05),
            take_profit: Some(0.10),
            position_size: 1.0,
            strategy: StrategyConfig::new(),
        }
    }
}
