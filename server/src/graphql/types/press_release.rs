//! GraphQL type for a company's own press releases (`Capability::CORPORATE`),
//! distinct from `news`, which returns press coverage.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::PressRelease`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlPressRelease {
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
}
