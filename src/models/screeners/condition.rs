//! Typed screener query condition types.
//!
//! This module defines the core traits and types for building type-safe screener
//! filter conditions. The key design is:
//!
//! - [`ScreenerField`] trait — implemented by [`EquityField`](super::fields::EquityField) and
//!   [`FundField`](super::fields::FundField)
//! - [`ScreenerFieldExt`] blanket trait — fluent condition builders on any `ScreenerField`
//! - [`QueryCondition<F>`] — a typed condition with serialization matching Yahoo's API format
//! - [`QueryGroup<F>`] and [`QueryOperand<F>`] — for composing nested AND/OR logic
use serde::Serialize;
use serde::ser::SerializeStruct;

// ============================================================================
// Operator
// ============================================================================

/// Comparison operator for screener query conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operator {
    /// Equal to (`"eq"`)
    #[serde(rename = "eq")]
    Eq,
    /// Greater than (`"gt"`)
    #[serde(rename = "gt")]
    Gt,
    /// Greater than or equal to (`"gte"`)
    #[serde(rename = "gte")]
    Gte,
    /// Less than (`"lt"`)
    #[serde(rename = "lt")]
    Lt,
    /// Less than or equal to (`"lte"`)
    #[serde(rename = "lte")]
    Lte,
    /// Between two values, inclusive (`"btwn"`)
    #[serde(rename = "btwn")]
    Between,
}

impl std::str::FromStr for Operator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eq" | "=" | "==" => Ok(Operator::Eq),
            "gt" | ">" => Ok(Operator::Gt),
            "gte" | ">=" => Ok(Operator::Gte),
            "lt" | "<" => Ok(Operator::Lt),
            "lte" | "<=" => Ok(Operator::Lte),
            "btwn" | "between" => Ok(Operator::Between),
            _ => Err(()),
        }
    }
}

// ============================================================================
// LogicalOperator
// ============================================================================

/// Logical operator for combining multiple screener conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogicalOperator {
    /// All conditions must match (AND logic)
    #[default]
    And,
    /// Any condition can match (OR logic)
    Or,
}

impl std::str::FromStr for LogicalOperator {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "and" | "&&" => Ok(LogicalOperator::And),
            "or" | "||" => Ok(LogicalOperator::Or),
            _ => Err(()),
        }
    }
}

// ============================================================================
// ScreenerField trait
// ============================================================================

/// A typed screener field usable in custom query conditions, sorting, and
/// response field selection.
///
/// Both [`EquityField`](super::fields::EquityField) and
/// [`FundField`](super::fields::FundField) implement this trait.
///
/// The `Serialize` bound ensures field values can be included in the JSON body
/// sent to Yahoo Finance (e.g., in `sortField` and `includeFields`). The
/// serialization always produces the raw Yahoo API field name string.
pub trait ScreenerField: Clone + Serialize + 'static {
    /// Returns the Yahoo Finance API field name string.
    ///
    /// For example, `EquityField::PeRatio.as_str()` returns
    /// `"peratio.lasttwelvemonths"`.
    fn as_str(&self) -> &'static str;
}

// ============================================================================
// ConditionValue
// ============================================================================

/// The value portion of a typed screener condition.
///
/// This cleanly separates the filter value from the field name, replacing the
/// old `Vec<QueryValue>` approach where the field string was mixed into the same
/// array as the comparison values.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ConditionValue {
    /// A single numeric value.
    ///
    /// Used with `Gt`, `Lt`, `Gte`, `Lte`, and `Eq` operators on numeric fields.
    Number(f64),
    /// A numeric range for `Between` conditions (inclusive).
    Between(f64, f64),
    /// A single string equality value.
    ///
    /// Used with `Eq` on categorical fields like `Region`, `Sector`, `Industry`.
    StringEq(String),
}

// ============================================================================
// QueryCondition<F>
// ============================================================================

/// A typed filter condition for a screener query.
///
/// Created via [`ScreenerFieldExt`] methods on a field enum variant. The custom
/// [`Serialize`] impl produces the exact format Yahoo Finance expects:
/// `{"operator": "gt", "operands": ["fieldname", value]}`.
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, ScreenerFieldExt};
///
/// let volume_filter = EquityField::AvgDailyVol3M.gt(200_000.0);
/// let region_filter = EquityField::Region.eq_str("us");
/// let pe_filter     = EquityField::PeRatio.between(10.0, 25.0);
/// ```
#[derive(Debug, Clone)]
pub struct QueryCondition<F: ScreenerField> {
    /// The field to filter on.
    pub field: F,
    /// The comparison operator.
    pub operator: Operator,
    /// The filter value(s).
    pub value: ConditionValue,
}

impl<F: ScreenerField> Serialize for QueryCondition<F> {
    /// Serializes to Yahoo Finance's format:
    /// `{"operator": "gt", "operands": ["fieldname", val]}`
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("QueryCondition", 2)?;
        s.serialize_field("operator", &self.operator)?;

        let field_str = self.field.as_str();
        let operands: serde_json::Value = match &self.value {
            ConditionValue::Number(v) => serde_json::json!([field_str, v]),
            ConditionValue::Between(min, max) => serde_json::json!([field_str, min, max]),
            ConditionValue::StringEq(v) => serde_json::json!([field_str, v]),
        };

        s.serialize_field("operands", &operands)?;
        s.end()
    }
}

// ============================================================================
// QueryGroup<F> and QueryOperand<F>
// ============================================================================

/// A group of query operands combined with a logical operator.
///
/// Groups can be nested to form complex AND/OR trees.
#[derive(Debug, Clone, Serialize)]
pub struct QueryGroup<F: ScreenerField> {
    /// The logical operator combining all operands in this group.
    pub operator: LogicalOperator,
    /// The operands — each is either a [`QueryCondition`] or a nested [`QueryGroup`].
    pub operands: Vec<QueryOperand<F>>,
}

impl<F: ScreenerField> QueryGroup<F> {
    /// Create a new empty group with the given logical operator.
    pub fn new(operator: LogicalOperator) -> Self {
        Self {
            operator,
            operands: Vec::new(),
        }
    }

    /// Add an operand to this group.
    pub fn add_operand(&mut self, operand: QueryOperand<F>) {
        self.operands.push(operand);
    }
}

/// An operand within a query group — either a leaf condition or a nested group.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum QueryOperand<F: ScreenerField> {
    /// A single filter condition.
    Condition(QueryCondition<F>),
    /// A nested group of conditions.
    Group(QueryGroup<F>),
}

// ============================================================================
// ScreenerFieldExt — fluent condition builders
// ============================================================================

/// Fluent condition-building methods on any [`ScreenerField`] type.
///
/// This blanket trait is automatically implemented for all types that implement
/// [`ScreenerField`], including [`EquityField`](super::fields::EquityField) and
/// [`FundField`](super::fields::FundField).
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, ScreenerFieldExt};
///
/// // Numeric comparisons
/// let cond = EquityField::PeRatio.between(10.0, 25.0);
/// let cond = EquityField::AvgDailyVol3M.gt(500_000.0);
/// let cond = EquityField::EsgScore.gte(50.0);
///
/// // String equality
/// let cond = EquityField::Region.eq_str("us");
/// let cond = EquityField::Exchange.eq_str("NMS");
/// ```
pub trait ScreenerFieldExt: ScreenerField + Sized {
    /// Filter where this field is greater than `v`.
    fn gt(self, v: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Gt,
            value: ConditionValue::Number(v),
        }
    }

    /// Filter where this field is less than `v`.
    fn lt(self, v: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Lt,
            value: ConditionValue::Number(v),
        }
    }

    /// Filter where this field is greater than or equal to `v`.
    fn gte(self, v: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Gte,
            value: ConditionValue::Number(v),
        }
    }

    /// Filter where this field is less than or equal to `v`.
    fn lte(self, v: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Lte,
            value: ConditionValue::Number(v),
        }
    }

    /// Filter where this field equals the numeric value `v`.
    fn eq_num(self, v: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Eq,
            value: ConditionValue::Number(v),
        }
    }

    /// Filter where this field is between `min` and `max` (inclusive).
    fn between(self, min: f64, max: f64) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Between,
            value: ConditionValue::Between(min, max),
        }
    }

    /// Filter where this field equals the string value `v`.
    ///
    /// Use for categorical fields like `Region`, `Sector`, `Industry`, `Exchange`.
    fn eq_str(self, v: impl Into<String>) -> QueryCondition<Self> {
        QueryCondition {
            field: self,
            operator: Operator::Eq,
            value: ConditionValue::StringEq(v.into()),
        }
    }
}

/// Blanket implementation: every `ScreenerField` automatically gets all the
/// fluent condition-building methods from `ScreenerFieldExt`.
impl<T: ScreenerField + Sized> ScreenerFieldExt for T {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::screeners::fields::EquityField;

    #[test]
    fn test_condition_gt_serializes_correctly() {
        let condition = EquityField::AvgDailyVol3M.gt(200_000.0);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["operator"], "gt");
        assert_eq!(json["operands"][0], "avgdailyvol3m");
        assert_eq!(json["operands"][1], 200_000.0);
    }

    #[test]
    fn test_condition_lt_serializes_correctly() {
        let condition = EquityField::PeRatio.lt(30.0);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["operator"], "lt");
        assert_eq!(json["operands"][0], "peratio.lasttwelvemonths");
        assert_eq!(json["operands"][1], 30.0);
    }

    #[test]
    fn test_condition_between_serializes_correctly() {
        let condition = EquityField::PeRatio.between(10.0, 25.0);
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["operator"], "btwn");
        assert_eq!(json["operands"][0], "peratio.lasttwelvemonths");
        assert_eq!(json["operands"][1], 10.0);
        assert_eq!(json["operands"][2], 25.0);
    }

    #[test]
    fn test_condition_eq_str_serializes_correctly() {
        let condition = EquityField::Region.eq_str("us");
        let json = serde_json::to_value(&condition).unwrap();
        assert_eq!(json["operator"], "eq");
        assert_eq!(json["operands"][0], "region");
        assert_eq!(json["operands"][1], "us");
    }

    #[test]
    fn test_query_group_serializes_correctly() {
        let mut group = QueryGroup::new(LogicalOperator::And);
        group.add_operand(QueryOperand::Condition(EquityField::Region.eq_str("us")));
        group.add_operand(QueryOperand::Condition(
            EquityField::AvgDailyVol3M.gt(200_000.0),
        ));

        let json = serde_json::to_value(&group).unwrap();
        assert_eq!(json["operator"], "and");
        assert_eq!(json["operands"].as_array().unwrap().len(), 2);
    }
}
