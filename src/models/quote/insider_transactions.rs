//! Insider Transactions Module
//!
//! Contains recent insider transaction data (buys, sells, etc.).

use serde::{Deserialize, Serialize};

/// Insider transaction history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderTransactions {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// List of transactions
    #[serde(default)]
    pub transactions: Vec<InsiderTransaction>,
}

/// Individual insider transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderTransaction {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Number of shares involved
    #[serde(default)]
    pub shares: Option<crate::models::quote::FormattedValue<i64>>,

    /// Total value of the transaction
    #[serde(default)]
    pub value: Option<crate::models::quote::FormattedValue<i64>>,

    /// Name of the filer
    #[serde(default)]
    pub filer_name: Option<String>,

    /// Filer's relationship to the company
    #[serde(default)]
    pub filer_relation: Option<String>,

    /// URL for filer information (often empty)
    #[serde(default)]
    pub filer_url: Option<String>,

    /// Text description of the money involved
    #[serde(default)]
    pub money_text: Option<String>,

    /// Start date of the transaction
    #[serde(default)]
    pub start_date: Option<crate::models::quote::FormattedValue<i64>>,

    /// Ownership type ("D" for direct, "I" for indirect)
    #[serde(default)]
    pub ownership: Option<String>,

    /// Text description of the transaction
    #[serde(default)]
    pub transaction_text: Option<String>,
}
