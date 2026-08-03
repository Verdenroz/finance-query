//! FINRA Query API wire types.

use serde::{Deserialize, Serialize};

/// One row of `otcMarket/regShoDaily`: a symbol's short volume on one trade
/// date, **per reporting facility**. A symbol has one row per facility per
/// day, so rows must be summed by date to get the consolidated figure.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegShoDailyRow {
    pub trade_report_date: String,
    #[serde(default)]
    pub short_par_quantity: Option<f64>,
    #[serde(default)]
    pub short_exempt_par_quantity: Option<f64>,
    #[serde(default)]
    pub total_par_quantity: Option<f64>,
}

/// A `compareFilters` entry in a FINRA data request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompareFilter<'a> {
    pub field_name: &'a str,
    pub field_value: &'a str,
    pub compare_type: &'a str,
}

/// A `dateRangeFilters` entry in a FINRA data request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DateRangeFilter<'a> {
    pub field_name: &'a str,
    pub start_date: String,
    pub end_date: String,
}

/// The request body FINRA's data endpoints accept.
///
/// Sorting is deliberately absent: FINRA rejects `sortFields` unless every
/// partition key is pinned with an `EQUAL` filter, which a date *range* by
/// definition does not do. Rows are ordered locally instead.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DataRequest<'a> {
    pub limit: u32,
    pub compare_filters: Vec<CompareFilter<'a>>,
    pub date_range_filters: Vec<DateRangeFilter<'a>>,
}

/// The error body FINRA returns for a malformed request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FinraError {
    #[serde(default)]
    pub message: Option<String>,
}
