use crate::constants::screener_query::{
    LogicalOperator, Operator, QuoteType, SortType, equity_fields, fund_fields,
};
use serde::{Deserialize, Serialize};

/// A custom screener query for Yahoo Finance
///
/// Allows building flexible queries to filter stocks/funds/ETFs based on
/// various criteria like price, volume, market cap, and more.
///
/// # Example
///
/// ```
/// use finance_query::{ScreenerQuery, QueryCondition, screener_query::{Operator, QuoteType}};
///
/// // Find US stocks with high volume and market cap > $10B
/// let query = ScreenerQuery::new()
///     .quote_type(QuoteType::Equity)
///     .size(25)
///     .sort_by("intradaymarketcap", false)  // Sort by market cap descending
///     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
///     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000))
///     .add_condition(QueryCondition::new("intradaymarketcap", Operator::Gt).value(10_000_000_000.0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerQuery {
    /// Number of results to return (default: 25, max: 250)
    pub size: u32,

    /// Starting offset for pagination (default: 0)
    pub offset: u32,

    /// Sort direction (ASC or DESC)
    pub sort_type: SortType,

    /// Field to sort by (e.g., "intradaymarketcap", "percentchange")
    pub sort_field: String,

    /// Fields to include in the response
    pub include_fields: Vec<String>,

    /// Top-level logical operator (AND or OR)
    pub top_operator: LogicalOperator,

    /// Query filter conditions
    pub query: QueryGroup,

    /// Type of quote to screen (EQUITY, ETF, MUTUALFUND, etc.)
    pub quote_type: QuoteType,
}

/// Default fields to include in screener results for equities
const DEFAULT_EQUITY_FIELDS: &[&str] = &[
    "ticker",
    "companyshortname",
    equity_fields::INTRADAY_PRICE,
    equity_fields::INTRADAY_PRICE_CHANGE,
    equity_fields::PERCENT_CHANGE,
    equity_fields::INTRADAY_MARKET_CAP,
    equity_fields::DAY_VOLUME,
    equity_fields::AVG_DAILY_VOL_3M,
    equity_fields::PE_RATIO,
    equity_fields::FIFTY_TWO_WK_PCT_CHANGE,
];

/// Default fields to include in screener results for mutual funds
const DEFAULT_FUND_FIELDS: &[&str] = &[
    "ticker",
    "companyshortname",
    fund_fields::INTRADAY_PRICE,
    fund_fields::INTRADAY_PRICE_CHANGE,
    fund_fields::CATEGORY_NAME,
    fund_fields::PERFORMANCE_RATING,
    fund_fields::RISK_RATING,
];

impl Default for ScreenerQuery {
    fn default() -> Self {
        Self {
            size: 25,
            offset: 0,
            sort_type: SortType::Desc,
            sort_field: equity_fields::INTRADAY_MARKET_CAP.to_string(),
            include_fields: DEFAULT_EQUITY_FIELDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            top_operator: LogicalOperator::And,
            query: QueryGroup::new(LogicalOperator::And),
            quote_type: QuoteType::Equity,
        }
    }
}

impl ScreenerQuery {
    /// Create a new screener query with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of results to return (max 250)
    pub fn size(mut self, size: u32) -> Self {
        self.size = size.min(250);
        self
    }

    /// Set the pagination offset
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    /// Set the sort field and direction
    ///
    /// # Arguments
    ///
    /// * `field` - Field to sort by (e.g., "intradaymarketcap", "percentchange")
    /// * `ascending` - If true, sort ascending; if false, sort descending
    pub fn sort_by(mut self, field: impl Into<String>, ascending: bool) -> Self {
        self.sort_field = field.into();
        self.sort_type = if ascending {
            SortType::Asc
        } else {
            SortType::Desc
        };
        self
    }

    /// Set the quote type (EQUITY or MUTUALFUND)
    ///
    /// This also updates the default include_fields and sort_field
    /// to use appropriate fields for the quote type.
    pub fn quote_type(mut self, quote_type: QuoteType) -> Self {
        self.quote_type = quote_type;
        // Update default fields based on quote type
        let (default_fields, default_sort) = match quote_type {
            QuoteType::Equity => (DEFAULT_EQUITY_FIELDS, equity_fields::INTRADAY_MARKET_CAP),
            QuoteType::MutualFund => (DEFAULT_FUND_FIELDS, fund_fields::INTRADAY_PRICE),
        };
        self.include_fields = default_fields.iter().map(|s| s.to_string()).collect();
        self.sort_field = default_sort.to_string();
        self
    }

    /// Set the fields to include in the response
    pub fn include_fields(mut self, fields: Vec<String>) -> Self {
        self.include_fields = fields;
        self
    }

    /// Add a field to include in the response
    pub fn add_include_field(mut self, field: impl Into<String>) -> Self {
        self.include_fields.push(field.into());
        self
    }

    /// Set the top-level operator (AND or OR)
    pub fn top_operator(mut self, op: LogicalOperator) -> Self {
        self.top_operator = op;
        self
    }

    /// Add a filter condition
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{ScreenerQuery, QueryCondition, screener_query::Operator};
    ///
    /// let query = ScreenerQuery::new()
    ///     .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"))
    ///     .add_condition(QueryCondition::new("avgdailyvol3m", Operator::Gt).value(200000));
    /// ```
    pub fn add_condition(mut self, condition: QueryCondition) -> Self {
        // Wrap in OR group (Yahoo's expected format)
        let mut or_group = QueryGroup::new(LogicalOperator::Or);
        or_group.add_operand(QueryOperand::Condition(condition));
        self.query.add_operand(QueryOperand::Group(or_group));
        self
    }

    /// Add multiple conditions that will be OR'd together
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{ScreenerQuery, QueryCondition, screener_query::Operator};
    ///
    /// // Filter for region being US OR GB
    /// let query = ScreenerQuery::new()
    ///     .add_or_conditions(vec![
    ///         QueryCondition::new("region", Operator::Eq).value_str("us"),
    ///         QueryCondition::new("region", Operator::Eq).value_str("gb"),
    ///     ]);
    /// ```
    pub fn add_or_conditions(mut self, conditions: Vec<QueryCondition>) -> Self {
        let mut or_group = QueryGroup::new(LogicalOperator::Or);
        for condition in conditions {
            or_group.add_operand(QueryOperand::Condition(condition));
        }
        self.query.add_operand(QueryOperand::Group(or_group));
        self
    }

    /// Create a "most shorted stocks" screener preset
    ///
    /// Finds US stocks sorted by short interest percentage.
    pub fn most_shorted() -> Self {
        Self::new()
            .sort_by(equity_fields::SHORT_PCT_FLOAT, false)
            .add_condition(QueryCondition::new(equity_fields::REGION, Operator::Eq).value_str("us"))
            .add_condition(
                QueryCondition::new(equity_fields::AVG_DAILY_VOL_3M, Operator::Gt).value(200000),
            )
    }

    /// Create a "high dividend yield" screener preset
    ///
    /// Finds US stocks with dividend yield > 3%.
    pub fn high_dividend() -> Self {
        Self::new()
            .sort_by(equity_fields::FORWARD_DIV_YIELD, false)
            .add_condition(QueryCondition::new(equity_fields::REGION, Operator::Eq).value_str("us"))
            .add_condition(
                QueryCondition::new(equity_fields::FORWARD_DIV_YIELD, Operator::Gt).value(3.0),
            )
            .add_condition(
                QueryCondition::new(equity_fields::AVG_DAILY_VOL_3M, Operator::Gt).value(100000),
            )
    }

    /// Create a "large cap growth" screener preset
    ///
    /// Finds large cap stocks with positive earnings growth.
    pub fn large_cap_growth() -> Self {
        Self::new()
            .sort_by(equity_fields::INTRADAY_MARKET_CAP, false)
            .add_condition(QueryCondition::new(equity_fields::REGION, Operator::Eq).value_str("us"))
            .add_condition(
                QueryCondition::new(equity_fields::INTRADAY_MARKET_CAP, Operator::Gt)
                    .value(10_000_000_000.0f64),
            )
            .add_condition(QueryCondition::new(equity_fields::EPS_GROWTH, Operator::Gt).value(0.0))
    }
}

/// A group of query operands combined with a logical operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryGroup {
    /// The logical operator (AND or OR)
    pub operator: LogicalOperator,
    /// The operands in this group
    pub operands: Vec<QueryOperand>,
}

impl QueryGroup {
    /// Create a new empty query group
    pub fn new(operator: LogicalOperator) -> Self {
        Self {
            operator,
            operands: Vec::new(),
        }
    }

    /// Add an operand to this group
    pub fn add_operand(&mut self, operand: QueryOperand) {
        self.operands.push(operand);
    }
}

/// An operand in a query - either a condition or a nested group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryOperand {
    /// A filter condition
    Condition(QueryCondition),
    /// A nested group of conditions
    Group(QueryGroup),
}

/// A single filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCondition {
    /// The comparison operator
    pub operator: Operator,
    /// The operands: [field_name, value(s)...]
    pub operands: Vec<QueryValue>,
}

impl QueryCondition {
    /// Create a new condition for a field
    ///
    /// # Arguments
    ///
    /// * `field` - The field to filter on (e.g., "region", "avgdailyvol3m")
    /// * `operator` - The comparison operator (Eq, Gt, Lt, etc.)
    pub fn new(field: impl Into<String>, operator: Operator) -> Self {
        Self {
            operator,
            operands: vec![QueryValue::String(field.into())],
        }
    }

    /// Set a numeric value for the condition
    pub fn value<T: Into<f64>>(mut self, value: T) -> Self {
        self.operands.push(QueryValue::Number(value.into()));
        self
    }

    /// Set a string value for the condition
    pub fn value_str(mut self, value: impl Into<String>) -> Self {
        self.operands.push(QueryValue::String(value.into()));
        self
    }

    /// Set two values for a BETWEEN condition
    pub fn between<T: Into<f64>>(mut self, min: T, max: T) -> Self {
        self.operands.push(QueryValue::Number(min.into()));
        self.operands.push(QueryValue::Number(max.into()));
        self
    }
}

/// A value in a query condition (can be string or number)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryValue {
    /// A string value
    String(String),
    /// A numeric value
    Number(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_query() {
        let query = ScreenerQuery::new();
        assert_eq!(query.size, 25);
        assert_eq!(query.offset, 0);
        assert_eq!(query.quote_type, QuoteType::Equity);
    }

    #[test]
    fn test_most_shorted_preset() {
        let query = ScreenerQuery::most_shorted();
        assert_eq!(query.sort_field, equity_fields::SHORT_PCT_FLOAT);
        assert_eq!(query.sort_type, SortType::Desc);
    }

    #[test]
    fn test_condition_builder() {
        let condition = QueryCondition::new("region", Operator::Eq).value_str("us");
        assert_eq!(condition.operator, Operator::Eq);
        assert_eq!(condition.operands.len(), 2);
    }

    #[test]
    fn test_query_serialization() {
        let query = ScreenerQuery::new()
            .size(10)
            .add_condition(QueryCondition::new("region", Operator::Eq).value_str("us"));

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("\"size\":10"));
        assert!(json.contains("\"region\""));
    }
}
