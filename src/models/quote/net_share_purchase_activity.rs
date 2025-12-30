//! Net Share Purchase Activity Module
//!
//! Contains insider buying and selling activity data.

use serde::{Deserialize, Serialize};

/// Net share purchase activity by insiders
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetSharePurchaseActivity {
    /// Time period for this activity (e.g., "6m")
    #[serde(default)]
    pub period: Option<String>,

    /// Number of buy transactions
    #[serde(default)]
    pub buy_info_count: Option<crate::models::quote::FormattedValue<i64>>,

    /// Total shares bought
    #[serde(default)]
    pub buy_info_shares: Option<crate::models::quote::FormattedValue<i64>>,

    /// Buy amount as percentage of insider shares
    #[serde(default)]
    pub buy_percent_insider_shares: Option<crate::models::quote::FormattedValue<f64>>,

    /// Number of sell transactions
    #[serde(default)]
    pub sell_info_count: Option<crate::models::quote::FormattedValue<i64>>,

    /// Total shares sold
    #[serde(default)]
    pub sell_info_shares: Option<crate::models::quote::FormattedValue<i64>>,

    /// Sell amount as percentage of insider shares
    #[serde(default)]
    pub sell_percent_insider_shares: Option<crate::models::quote::FormattedValue<f64>>,

    /// Net transaction count (buys - sells)
    #[serde(default)]
    pub net_info_count: Option<crate::models::quote::FormattedValue<i64>>,

    /// Net shares (bought - sold)
    #[serde(default)]
    pub net_info_shares: Option<crate::models::quote::FormattedValue<i64>>,

    /// Net amount as percentage of insider shares
    #[serde(default)]
    pub net_percent_insider_shares: Option<crate::models::quote::FormattedValue<f64>>,

    /// Total shares held by insiders
    #[serde(default)]
    pub total_insider_shares: Option<crate::models::quote::FormattedValue<i64>>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}
