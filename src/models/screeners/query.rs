use crate::models::screeners::condition::{
    LogicalOperator, QueryCondition, QueryGroup, QueryOperand, ScreenerField, ScreenerFieldExt,
};
use crate::models::screeners::fields::{EquityField, FundField};
use serde::{Deserialize, Serialize};

// ============================================================================
// QuoteType
// ============================================================================

/// Quote type for custom screener queries.
///
/// Yahoo Finance only supports `EQUITY` and `MUTUALFUND` for custom screener queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuoteType {
    /// Equity (stocks) — use [`EquityScreenerQuery`] with [`EquityField`] conditions.
    #[default]
    #[serde(rename = "EQUITY")]
    Equity,
    /// Mutual funds — use [`FundScreenerQuery`] with [`FundField`] conditions.
    #[serde(rename = "MUTUALFUND")]
    MutualFund,
}

impl std::str::FromStr for QuoteType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "equity" | "stock" | "stocks" => Ok(QuoteType::Equity),
            "mutualfund" | "fund" | "funds" => Ok(QuoteType::MutualFund),
            _ => Err(()),
        }
    }
}

// ============================================================================
// SortType
// ============================================================================

/// Sort direction for screener results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SortType {
    /// Sort ascending (smallest first) — `"ASC"`
    #[serde(rename = "ASC")]
    Asc,
    /// Sort descending (largest first) — `"DESC"`
    #[default]
    #[serde(rename = "DESC")]
    Desc,
}

impl std::str::FromStr for SortType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Ok(SortType::Asc),
            "desc" | "descending" => Ok(SortType::Desc),
            _ => Err(()),
        }
    }
}

// ============================================================================
// ScreenerQuery<F>
// ============================================================================

/// A typed custom screener query for Yahoo Finance.
///
/// The type parameter `F` determines which field set is valid for this query.
/// Use the type aliases for the common cases:
/// - [`EquityScreenerQuery`] — for stock screeners
/// - [`FundScreenerQuery`] — for mutual fund screeners
///
/// # Example
///
/// ```
/// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt};
///
/// // Find US large-cap value stocks
/// let query = EquityScreenerQuery::new()
///     .size(25)
///     .sort_by(EquityField::IntradayMarketCap, false)
///     .add_condition(EquityField::Region.eq_str("us"))
///     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0))
///     .add_condition(EquityField::PeRatio.between(10.0, 25.0))
///     .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
///     .include_fields(vec![
///         EquityField::Ticker,
///         EquityField::CompanyShortName,
///         EquityField::IntradayPrice,
///         EquityField::PeRatio,
///         EquityField::IntradayMarketCap,
///     ]);
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenerQuery<F: ScreenerField = EquityField> {
    /// Number of results to return (default: 25, max: 250).
    pub size: u32,

    /// Starting offset for pagination (default: 0).
    pub offset: u32,

    /// Sort direction.
    pub sort_type: SortType,

    /// Field to sort by.
    pub sort_field: F,

    /// Fields to include in the response.
    pub include_fields: Vec<F>,

    /// Top-level logical operator combining all conditions.
    pub top_operator: LogicalOperator,

    /// The nested condition tree.
    pub query: QueryGroup<F>,

    /// Quote type — determines which Yahoo Finance screener endpoint is used.
    pub quote_type: QuoteType,
}

/// Type alias for equity (stock) screener queries.
///
/// Use [`EquityField`] variants to build conditions.
pub type EquityScreenerQuery = ScreenerQuery<EquityField>;

/// Type alias for mutual fund screener queries.
///
/// Use [`FundField`] variants to build conditions.
pub type FundScreenerQuery = ScreenerQuery<FundField>;

// ============================================================================
// Default impls
// ============================================================================

impl Default for ScreenerQuery<EquityField> {
    fn default() -> Self {
        Self {
            size: 25,
            offset: 0,
            sort_type: SortType::Desc,
            sort_field: EquityField::IntradayMarketCap,
            include_fields: vec![
                EquityField::Ticker,
                EquityField::CompanyShortName,
                EquityField::IntradayPrice,
                EquityField::IntradayPriceChange,
                EquityField::PercentChange,
                EquityField::IntradayMarketCap,
                EquityField::DayVolume,
                EquityField::AvgDailyVol3M,
                EquityField::PeRatio,
                EquityField::FiftyTwoWkPctChange,
            ],
            top_operator: LogicalOperator::And,
            query: QueryGroup::new(LogicalOperator::And),
            quote_type: QuoteType::Equity,
        }
    }
}

impl Default for ScreenerQuery<FundField> {
    fn default() -> Self {
        Self {
            size: 25,
            offset: 0,
            sort_type: SortType::Desc,
            sort_field: FundField::IntradayPrice,
            include_fields: vec![
                FundField::Ticker,
                FundField::CompanyShortName,
                FundField::IntradayPrice,
                FundField::IntradayPriceChange,
                FundField::CategoryName,
                FundField::PerformanceRating,
                FundField::RiskRating,
            ],
            top_operator: LogicalOperator::And,
            query: QueryGroup::new(LogicalOperator::And),
            quote_type: QuoteType::MutualFund,
        }
    }
}

// ============================================================================
// Shared builder methods
// ============================================================================

impl<F: ScreenerField> ScreenerQuery<F> {
    /// Create a new screener query with default settings.
    pub fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }

    /// Set the number of results to return (capped at 250).
    pub fn size(mut self, size: u32) -> Self {
        self.size = size.min(250);
        self
    }

    /// Set the pagination offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    /// Set the field to sort by and the sort direction.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{EquityField, EquityScreenerQuery};
    ///
    /// let query = EquityScreenerQuery::new()
    ///     .sort_by(EquityField::PeRatio, true);  // ascending P/E
    /// ```
    pub fn sort_by(mut self, field: F, ascending: bool) -> Self {
        self.sort_field = field;
        self.sort_type = if ascending {
            SortType::Asc
        } else {
            SortType::Desc
        };
        self
    }

    /// Set the top-level logical operator (AND or OR).
    pub fn top_operator(mut self, op: LogicalOperator) -> Self {
        self.top_operator = op;
        self
    }

    /// Set which fields to include in the response.
    pub fn include_fields(mut self, fields: Vec<F>) -> Self {
        self.include_fields = fields;
        self
    }

    /// Add a field to include in the response.
    pub fn add_include_field(mut self, field: F) -> Self {
        self.include_fields.push(field);
        self
    }

    /// Add a typed filter condition to this query (ANDed with all others).
    ///
    /// Conditions are added directly as operands of the top-level AND group,
    /// matching the format Yahoo Finance's screener API expects. Use
    /// [`add_or_conditions`](Self::add_or_conditions) when you need to match
    /// any of several values for the same field.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt};
    ///
    /// let query = EquityScreenerQuery::new()
    ///     .add_condition(EquityField::Region.eq_str("us"))
    ///     .add_condition(EquityField::PeRatio.between(10.0, 25.0))
    ///     .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0));
    /// ```
    pub fn add_condition(mut self, condition: QueryCondition<F>) -> Self {
        self.query.add_operand(QueryOperand::Condition(condition));
        self
    }

    /// Add multiple conditions that are OR'd together.
    ///
    /// # Example
    ///
    /// ```
    /// use finance_query::{EquityField, EquityScreenerQuery, ScreenerFieldExt};
    ///
    /// // Accept US or GB region
    /// let query = EquityScreenerQuery::new()
    ///     .add_or_conditions(vec![
    ///         EquityField::Region.eq_str("us"),
    ///         EquityField::Region.eq_str("gb"),
    ///     ]);
    /// ```
    pub fn add_or_conditions(mut self, conditions: Vec<QueryCondition<F>>) -> Self {
        let mut or_group = QueryGroup::new(LogicalOperator::Or);
        for condition in conditions {
            or_group.add_operand(QueryOperand::Condition(condition));
        }
        self.query.add_operand(QueryOperand::Group(or_group));
        self
    }
}

// ============================================================================
// Equity preset constructors
// ============================================================================

impl ScreenerQuery<EquityField> {
    /// Preset: US stocks sorted by short interest percentage of float.
    ///
    /// Filters: US region, average daily volume > 200K.
    ///
    /// ```no_run
    /// use finance_query::{EquityScreenerQuery, finance};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = finance::custom_screener(EquityScreenerQuery::most_shorted()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn most_shorted() -> Self {
        Self::new()
            .sort_by(EquityField::ShortPctFloat, false)
            .add_condition(EquityField::Region.eq_str("us"))
            .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0))
    }

    /// Preset: US stocks with forward dividend yield > 3%, sorted by yield descending.
    ///
    /// Filters: US region, forward dividend yield > 3%, average daily volume > 100K.
    ///
    /// ```no_run
    /// use finance_query::{EquityScreenerQuery, finance};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = finance::custom_screener(EquityScreenerQuery::high_dividend()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn high_dividend() -> Self {
        Self::new()
            .sort_by(EquityField::ForwardDivYield, false)
            .add_condition(EquityField::Region.eq_str("us"))
            .add_condition(EquityField::ForwardDivYield.gt(3.0))
            .add_condition(EquityField::AvgDailyVol3M.gt(100_000.0))
    }

    /// Preset: US large-cap stocks with positive EPS growth, sorted by market cap.
    ///
    /// Filters: US region, market cap > $10B, positive EPS growth.
    ///
    /// ```no_run
    /// use finance_query::{EquityScreenerQuery, finance};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let results = finance::custom_screener(EquityScreenerQuery::large_cap_growth()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn large_cap_growth() -> Self {
        Self::new()
            .sort_by(EquityField::IntradayMarketCap, false)
            .add_condition(EquityField::Region.eq_str("us"))
            .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
            .add_condition(EquityField::EpsGrowth.gt(0.0))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::screeners::condition::ScreenerFieldExt;

    #[test]
    fn test_default_equity_query() {
        let query = EquityScreenerQuery::new();
        assert_eq!(query.size, 25);
        assert_eq!(query.offset, 0);
        assert_eq!(query.quote_type, QuoteType::Equity);
        assert_eq!(query.sort_field, EquityField::IntradayMarketCap);
    }

    #[test]
    fn test_default_fund_query() {
        let query = FundScreenerQuery::new();
        assert_eq!(query.size, 25);
        assert_eq!(query.quote_type, QuoteType::MutualFund);
        assert_eq!(query.sort_field, FundField::IntradayPrice);
    }

    #[test]
    fn test_most_shorted_preset() {
        let query = EquityScreenerQuery::most_shorted();
        assert_eq!(query.sort_field, EquityField::ShortPctFloat);
        assert_eq!(query.sort_type, SortType::Desc);
    }

    #[test]
    fn test_high_dividend_preset() {
        let query = EquityScreenerQuery::high_dividend();
        assert_eq!(query.sort_field, EquityField::ForwardDivYield);
    }

    #[test]
    fn test_large_cap_growth_preset() {
        let query = EquityScreenerQuery::large_cap_growth();
        assert_eq!(query.sort_field, EquityField::IntradayMarketCap);
    }

    #[test]
    fn test_sort_by_typed_field() {
        let query = EquityScreenerQuery::new().sort_by(EquityField::PeRatio, true);
        assert_eq!(query.sort_field, EquityField::PeRatio);
        assert_eq!(query.sort_type, SortType::Asc);
    }

    #[test]
    fn test_size_capped_at_250() {
        let query = EquityScreenerQuery::new().size(9999);
        assert_eq!(query.size, 250);
    }

    #[test]
    fn test_query_serializes_sort_field_as_string() {
        let query = EquityScreenerQuery::new().sort_by(EquityField::PeRatio, false);
        let json = serde_json::to_value(&query).unwrap();
        assert_eq!(json["sortField"], "peratio.lasttwelvemonths");
        assert_eq!(json["sortType"], "DESC");
    }

    #[test]
    fn test_query_serializes_include_fields_as_strings() {
        let query = EquityScreenerQuery::new()
            .include_fields(vec![EquityField::Ticker, EquityField::PeRatio]);
        let json = serde_json::to_value(&query).unwrap();
        let fields = json["includeFields"].as_array().unwrap();
        assert_eq!(fields[0], "ticker");
        assert_eq!(fields[1], "peratio.lasttwelvemonths");
    }

    #[test]
    fn test_add_condition_adds_directly_to_and_group() {
        let query = EquityScreenerQuery::new().add_condition(EquityField::Region.eq_str("us"));
        let json = serde_json::to_value(&query).unwrap();
        // condition is a direct operand of the AND group (no OR wrapper)
        let outer_operands = json["query"]["operands"].as_array().unwrap();
        assert_eq!(outer_operands.len(), 1);
        assert_eq!(outer_operands[0]["operator"], "eq");
        assert_eq!(outer_operands[0]["operands"][0], "region");
    }

    #[test]
    fn test_full_query_serialization() {
        let query = EquityScreenerQuery::new()
            .size(10)
            .add_condition(EquityField::Region.eq_str("us"))
            .add_condition(EquityField::AvgDailyVol3M.gt(200_000.0));

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("\"size\":10"));
        assert!(json.contains("\"region\""));
        assert!(json.contains("\"avgdailyvol3m\""));
        assert!(json.contains("\"EQUITY\""));
    }
}
