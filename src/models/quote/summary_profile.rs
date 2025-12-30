/// Summary Profile module data
///
/// Contains company profile information including address, sector, industry, and business summary.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Company profile information
///
/// Contains address, contact information, sector, industry, and business description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryProfile {
    /// Street address line 1
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,

    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// Company officers (typically populated in assetProfile instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_officers: Option<Vec<Value>>,

    /// Country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Executive team (typically populated in assetProfile instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executive_team: Option<Vec<Value>>,

    /// Number of full-time employees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_time_employees: Option<i64>,

    /// Industry name (legacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,

    /// Industry display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_disp: Option<String>,

    /// Industry key/slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry_key: Option<String>,

    /// Investor relations website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ir_website: Option<String>,

    /// Detailed business summary/description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_business_summary: Option<String>,

    /// Maximum age of data in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<i64>,

    /// Company phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Sector name (legacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,

    /// Sector display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_disp: Option<String>,

    /// Sector key/slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sector_key: Option<String>,

    /// State/province code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Company website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// ZIP/postal code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_summary_profile() {
        let json = r#"{
            "address1": "One Apple Park Way",
            "city": "Cupertino",
            "companyOfficers": [],
            "country": "United States",
            "fullTimeEmployees": 166000,
            "industry": "Consumer Electronics",
            "industryDisp": "Consumer Electronics",
            "industryKey": "consumer-electronics",
            "phone": "(408) 996-1010",
            "sector": "Technology",
            "sectorDisp": "Technology",
            "sectorKey": "technology",
            "state": "CA",
            "website": "https://www.apple.com",
            "zip": "95014"
        }"#;

        let profile: SummaryProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.city.as_deref(), Some("Cupertino"));
        assert_eq!(profile.sector_disp.as_deref(), Some("Technology"));
        assert_eq!(profile.full_time_employees, Some(166000));
    }
}
