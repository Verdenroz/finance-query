//! Insider Holders Module
//!
//! Contains information about insider stockholders and their holdings.

use serde::{Deserialize, Serialize};

/// Insider holders data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderHolders {
    /// List of insider holders
    #[serde(default)]
    pub holders: Vec<InsiderHolder>,
}

/// Individual insider holder information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderHolder {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Name of the insider
    #[serde(default)]
    pub name: Option<String>,

    /// Relationship to the company (e.g., "CEO", "Director")
    #[serde(default)]
    pub relation: Option<String>,

    /// URL for more information (often empty)
    #[serde(default)]
    pub url: Option<String>,

    /// Description of the latest transaction
    #[serde(default)]
    pub transaction_description: Option<String>,

    /// Date of the latest transaction
    #[serde(default)]
    pub latest_trans_date: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of shares held directly
    #[serde(default)]
    pub position_direct: Option<crate::models::quote::FormattedValue<i64>>,

    /// Date of the direct position
    #[serde(default)]
    pub position_direct_date: Option<crate::models::quote::FormattedValue<i64>>,

    /// Number of shares held indirectly
    #[serde(default)]
    pub position_indirect: Option<crate::models::quote::FormattedValue<i64>>,

    /// Date of the indirect position
    #[serde(default)]
    pub position_indirect_date: Option<crate::models::quote::FormattedValue<i64>>,
}
