use serde::{Deserialize, Serialize};

use super::FormattedValue;

/// Fund profile information including management, fees, and expenses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundProfile {
    /// Maximum age of the data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// URL to the fund's style box image
    #[serde(default)]
    pub style_box_url: Option<String>,

    /// Fund family/company name
    #[serde(default)]
    pub family: Option<String>,

    /// Morningstar category name
    #[serde(default)]
    pub category_name: Option<String>,

    /// Legal type (e.g., "Exchange Traded Fund", "Mutual Fund")
    #[serde(default)]
    pub legal_type: Option<String>,

    /// Fund management information
    #[serde(default)]
    pub management_info: Option<ManagementInfo>,

    /// Fees and expenses for this fund
    #[serde(default)]
    pub fees_expenses_investment: Option<FeesExpenses>,

    /// Average fees and expenses for funds in the same category
    #[serde(default)]
    pub fees_expenses_investment_cat: Option<FeesExpensesCat>,

    /// List of brokerages offering this fund
    #[serde(default)]
    pub brokerages: Option<Vec<String>>,
}

/// Fund management information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementInfo {
    /// Name of the fund manager
    #[serde(default)]
    pub manager_name: Option<String>,

    /// Biography of the fund manager
    #[serde(default)]
    pub manager_bio: Option<String>,
}

/// Fees and expenses for a fund
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeesExpenses {
    /// Annual expense ratio from the latest report
    #[serde(default)]
    pub annual_report_expense_ratio: Option<FormattedValue<f64>>,

    /// Annual holdings turnover rate (percentage of portfolio changed per year)
    #[serde(default)]
    pub annual_holdings_turnover: Option<FormattedValue<f64>>,

    /// Total net assets in millions
    #[serde(default)]
    pub total_net_assets: Option<FormattedValue<f64>>,

    /// Projection values (typically empty)
    #[serde(default)]
    pub projection_values: Option<serde_json::Value>,
}

/// Average fees and expenses for funds in the same category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeesExpensesCat {
    /// Category average annual expense ratio
    #[serde(default)]
    pub annual_report_expense_ratio: Option<FormattedValue<f64>>,

    /// Category average annual holdings turnover
    #[serde(default)]
    pub annual_holdings_turnover: Option<FormattedValue<f64>>,

    /// Category average total net assets
    #[serde(default)]
    pub total_net_assets: Option<FormattedValue<f64>>,

    /// Category projection values (typically empty)
    #[serde(default)]
    pub projection_values_cat: Option<serde_json::Value>,
}
