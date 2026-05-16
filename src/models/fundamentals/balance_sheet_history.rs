use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Balance sheet history (annual statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetHistory {
    /// List of annual balance sheet statements
    #[serde(default)]
    pub balance_sheet_statements: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Balance sheet history (quarterly statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetHistoryQuarterly {
    /// List of quarterly balance sheet statements
    #[serde(default)]
    pub balance_sheet_statements: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}
