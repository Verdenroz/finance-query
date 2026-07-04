//! GraphQL types for holders / ownership data.

use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

// ── Major Holders Breakdown ────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMajorHoldersBreakdown {
    pub insiders_percent_held: Option<Json<serde_json::Value>>,
    pub institutions_count: Option<Json<serde_json::Value>>,
    pub institutions_float_percent_held: Option<Json<serde_json::Value>>,
    pub institutions_percent_held: Option<Json<serde_json::Value>>,
    pub max_age: Option<i64>,
}

// ── Institution Ownership ───────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInstitutionOwnership {
    pub max_age: Option<i64>,
    pub ownership_list: Vec<GqlInstitutionOwner>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInstitutionOwner {
    pub max_age: Option<i64>,
    pub organization: Option<String>,
    pub pct_held: Option<Json<serde_json::Value>>,
    pub position: Option<Json<serde_json::Value>>,
    pub value: Option<Json<serde_json::Value>>,
    pub pct_change: Option<Json<serde_json::Value>>,
    pub report_date: Option<Json<serde_json::Value>>,
}

// ── Fund Ownership ──────────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlFundOwnership {
    pub max_age: Option<i64>,
    pub ownership_list: Vec<GqlFundOwner>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlFundOwner {
    pub max_age: Option<i64>,
    pub organization: Option<String>,
    pub pct_held: Option<Json<serde_json::Value>>,
    pub position: Option<Json<serde_json::Value>>,
    pub value: Option<Json<serde_json::Value>>,
    pub pct_change: Option<Json<serde_json::Value>>,
    pub report_date: Option<Json<serde_json::Value>>,
}

// ── Insider Transactions ────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderTransactions {
    pub max_age: Option<i64>,
    pub transactions: Vec<GqlInsiderTransaction>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderTransaction {
    pub max_age: Option<i64>,
    pub shares: Option<Json<serde_json::Value>>,
    pub value: Option<Json<serde_json::Value>>,
    pub filer_name: Option<String>,
    pub filer_relation: Option<String>,
    pub filer_url: Option<String>,
    pub money_text: Option<String>,
    pub start_date: Option<Json<serde_json::Value>>,
    pub ownership: Option<String>,
    pub transaction_text: Option<String>,
}

// ── Net Share Purchase Activity (insider purchases) ─────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlNetSharePurchaseActivity {
    pub period: Option<String>,
    pub buy_info_count: Option<Json<serde_json::Value>>,
    pub buy_info_shares: Option<Json<serde_json::Value>>,
    pub buy_percent_insider_shares: Option<Json<serde_json::Value>>,
    pub sell_info_count: Option<Json<serde_json::Value>>,
    pub sell_info_shares: Option<Json<serde_json::Value>>,
    pub sell_percent_insider_shares: Option<Json<serde_json::Value>>,
    pub net_info_count: Option<Json<serde_json::Value>>,
    pub net_info_shares: Option<Json<serde_json::Value>>,
    pub net_percent_insider_shares: Option<Json<serde_json::Value>>,
    pub total_insider_shares: Option<Json<serde_json::Value>>,
    pub max_age: Option<i64>,
}

// ── Insider Holders (insider roster) ────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderHolders {
    pub holders: Vec<GqlInsiderHolder>,
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderHolder {
    pub max_age: Option<i64>,
    pub name: Option<String>,
    pub relation: Option<String>,
    pub url: Option<String>,
    pub transaction_description: Option<String>,
    pub latest_trans_date: Option<Json<serde_json::Value>>,
    pub position_direct: Option<Json<serde_json::Value>>,
    pub position_direct_date: Option<Json<serde_json::Value>>,
    pub position_indirect: Option<Json<serde_json::Value>>,
    pub position_indirect_date: Option<Json<serde_json::Value>>,
}
