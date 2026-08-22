//! GraphQL types for FMP-routed congressional trading and fails-to-deliver
//! disclosures, distinct from Yahoo's `insiderTransactions`/`secFilings`
//! and SEC EDGAR.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::CongressionalTrade`, which has no serde rename of
/// its own — this deserializes snake_case keys while its GraphQL name stays
/// camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlCongressionalTrade {
    pub symbol: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub office: Option<String>,
    pub district: Option<String>,
    pub trade_type: Option<String>,
    pub amount: Option<String>,
    pub asset_description: Option<String>,
    pub transaction_date: Option<String>,
    pub disclosure_date: Option<String>,
    pub link: Option<String>,
}

/// Mirrors `finance_query::FailToDeliver`, which has no serde rename of its
/// own — this deserializes snake_case keys while its GraphQL name stays
/// camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlFailToDeliver {
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
    pub name: Option<String>,
    pub description: Option<String>,
}
