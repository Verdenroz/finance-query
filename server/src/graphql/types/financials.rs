//! GraphQL types for financial statements.

use super::batch::GqlBatchError;
use async_graphql::SimpleObject;

/// A single line item in a financial statement (e.g., "TotalRevenue").
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFinancialLineItem {
    /// Metric name (e.g., "TotalRevenue", "NetIncome").
    pub metric: String,
    /// Time-series values: date → value pairs.
    pub values: Vec<GqlFinancialDataPoint>,
}

/// A single data point in a financial time series.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFinancialDataPoint {
    /// Date string (e.g., "2024-09-30").
    pub date: String,
    /// Numeric value.
    pub value: f64,
}

/// Wrapper for batch financials: `{symbol, statement}` — `statement` is every
/// line item in that symbol's statement, not a single one.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolFinancials {
    pub symbol: String,
    pub statement: Vec<GqlFinancialLineItem>,
}

/// Result of the batch `financialsBatch` root field: successfully fetched
/// statements plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFinancialsBatch {
    pub financials: Vec<GqlSymbolFinancials>,
    pub errors: Vec<GqlBatchError>,
}
