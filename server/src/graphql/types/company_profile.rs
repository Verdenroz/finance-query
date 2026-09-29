//! GraphQL type for a company's identity/classification profile.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::CompanyProfile`, which has no serde rename of its
/// own — this deserializes snake_case keys while its GraphQL name stays
/// camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlCompanyProfile {
    pub isin: Option<String>,
    pub cusip: Option<String>,
    pub active: Option<bool>,
    pub is_etf: Option<bool>,
    pub is_adr: Option<bool>,
    pub is_fund: Option<bool>,
    pub cik: Option<String>,
    pub ipo_date: Option<String>,
    pub provider_id: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_capitalization: Option<f64>,
}
