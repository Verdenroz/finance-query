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
