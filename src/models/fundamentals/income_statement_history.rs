use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Income statement history (annual statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatementHistory {
    /// List of annual income statements
    #[serde(default)]
    pub income_statement_history: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Income statement history (quarterly statements)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatementHistoryQuarterly {
    /// List of quarterly income statements
    #[serde(default)]
    pub income_statement_history: Option<Vec<Value>>,

    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}
