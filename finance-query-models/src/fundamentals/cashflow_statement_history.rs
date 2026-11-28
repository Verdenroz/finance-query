use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Cash flow statement history (annual statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashflowStatementHistory {
    /// List of annual cash flow statements
    #[serde(default)]
    pub cashflow_statements: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Cash flow statement history (quarterly statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CashflowStatementHistoryQuarterly {
    /// List of quarterly cash flow statements
    #[serde(default)]
    pub cashflow_statements: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}
