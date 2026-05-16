//! Asset Profile Module
//!
//! Contains company information including address, sector, officers, and risk metrics.

use serde::{Deserialize, Serialize};

/// Company asset profile and information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProfile {
    /// Street address line 1
    #[serde(default)]
    pub address1: Option<String>,

    /// Street address line 2
    #[serde(default)]
    pub address2: Option<String>,

    /// Street address line 3
    #[serde(default)]
    pub address3: Option<String>,

    /// City
    #[serde(default)]
    pub city: Option<String>,

    /// State or province
    #[serde(default)]
    pub state: Option<String>,

    /// Postal/ZIP code
    #[serde(default)]
    pub zip: Option<String>,

    /// Country
    #[serde(default)]
    pub country: Option<String>,

    /// Phone number
    #[serde(default)]
    pub phone: Option<String>,

    /// Fax number
    #[serde(default)]
    pub fax: Option<String>,

    /// Company website
    #[serde(default)]
    pub website: Option<String>,

    /// Industry
    #[serde(default)]
    pub industry: Option<String>,

    /// Industry key (machine-readable)
    #[serde(default)]
    pub industry_key: Option<String>,

    /// Industry disp (display name)
    #[serde(default)]
    pub industry_disp: Option<String>,

    /// Sector
    #[serde(default)]
    pub sector: Option<String>,

    /// Sector key (machine-readable)
    #[serde(default)]
    pub sector_key: Option<String>,

    /// Sector disp (display name)
    #[serde(default)]
    pub sector_disp: Option<String>,

    /// Long business summary
    #[serde(default)]
    pub long_business_summary: Option<String>,

    /// Number of full-time employees
    #[serde(default)]
    pub full_time_employees: Option<i64>,

    /// List of company officers
    #[serde(default)]
    pub company_officers: Vec<CompanyOfficer>,

    /// Audit risk score (1-10)
    #[serde(default)]
    pub audit_risk: Option<i32>,

    /// Board risk score (1-10)
    #[serde(default)]
    pub board_risk: Option<i32>,

    /// Compensation risk score (1-10)
    #[serde(default)]
    pub compensation_risk: Option<i32>,

    /// Shareholder rights risk score (1-10)
    #[serde(default)]
    pub shareholder_rights_risk: Option<i32>,

    /// Overall risk score (1-10)
    #[serde(default)]
    pub overall_risk: Option<i32>,

    /// Governance epoch date
    #[serde(default)]
    pub governance_epoch_date: Option<i64>,

    /// Compensation as of epoch date
    #[serde(default)]
    pub compensation_as_of_epoch_date: Option<i64>,

    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,
}

/// Company officer information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyOfficer {
    /// Maximum age of this data in seconds
    #[serde(default)]
    pub max_age: Option<i64>,

    /// Officer's name
    #[serde(default)]
    pub name: Option<String>,

    /// Officer's age
    #[serde(default)]
    pub age: Option<i32>,

    /// Officer's title/position
    #[serde(default)]
    pub title: Option<String>,

    /// Year the officer was born
    #[serde(default)]
    pub year_born: Option<i32>,

    /// Fiscal year for compensation data
    #[serde(default)]
    pub fiscal_year: Option<i32>,

    /// Total compensation/pay
    #[serde(default)]
    pub total_pay: Option<crate::models::quote::FormattedValue<i64>>,

    /// Value of exercised options
    #[serde(default)]
    pub exercised_value: Option<crate::models::quote::FormattedValue<i64>>,

    /// Value of unexercised options
    #[serde(default)]
    pub unexercised_value: Option<crate::models::quote::FormattedValue<i64>>,
}
