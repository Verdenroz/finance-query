//! GraphQL types for holders / ownership data.

use super::formatted::GqlFormattedValue;
use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Json, Object, Result, SimpleObject};
use serde::Deserialize;

// ── Major Holders Breakdown ────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMajorHoldersBreakdown {
    pub insiders_percent_held: Option<Json<GqlFormattedValue>>,
    pub institutions_count: Option<Json<GqlFormattedValue>>,
    pub institutions_float_percent_held: Option<Json<GqlFormattedValue>>,
    pub institutions_percent_held: Option<Json<GqlFormattedValue>>,
    pub max_age: Option<i64>,
}

// ── Institution Ownership ───────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInstitutionOwnership {
    pub max_age: Option<i64>,
    #[graphql(skip)]
    pub ownership_list: Vec<GqlInstitutionOwner>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlInstitutionOwnership {
    /// Institutional ownership positions.
    async fn ownership_list(
        &self,
        #[graphql(desc = "Max entries to return; omitted = every matching entry in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlInstitutionOwner>> {
        pagination::paginate(&self.ownership_list, first, after).await
    }
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInstitutionOwner {
    pub max_age: Option<i64>,
    pub organization: Option<String>,
    pub pct_held: Option<Json<GqlFormattedValue>>,
    pub position: Option<Json<GqlFormattedValue>>,
    pub value: Option<Json<GqlFormattedValue>>,
    pub pct_change: Option<Json<GqlFormattedValue>>,
    pub report_date: Option<Json<GqlFormattedValue>>,
}

// ── Fund Ownership ──────────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlFundOwnership {
    pub max_age: Option<i64>,
    #[graphql(skip)]
    pub ownership_list: Vec<GqlFundOwner>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlFundOwnership {
    /// Mutual fund ownership positions.
    async fn ownership_list(
        &self,
        #[graphql(desc = "Max entries to return; omitted = every matching entry in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlFundOwner>> {
        pagination::paginate(&self.ownership_list, first, after).await
    }
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlFundOwner {
    pub max_age: Option<i64>,
    pub organization: Option<String>,
    pub pct_held: Option<Json<GqlFormattedValue>>,
    pub position: Option<Json<GqlFormattedValue>>,
    pub value: Option<Json<GqlFormattedValue>>,
    pub pct_change: Option<Json<GqlFormattedValue>>,
    pub report_date: Option<Json<GqlFormattedValue>>,
}

// ── Insider Transactions ────────────────────────────────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderTransactions {
    pub max_age: Option<i64>,
    #[graphql(skip)]
    pub transactions: Vec<GqlInsiderTransaction>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlInsiderTransactions {
    /// Insider transaction history.
    async fn transactions(
        &self,
        #[graphql(desc = "Max entries to return; omitted = every matching entry in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlInsiderTransaction>> {
        pagination::paginate(&self.transactions, first, after).await
    }
}

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderTransaction {
    pub max_age: Option<i64>,
    pub shares: Option<Json<GqlFormattedValue>>,
    pub value: Option<Json<GqlFormattedValue>>,
    pub filer_name: Option<String>,
    pub filer_relation: Option<String>,
    pub filer_url: Option<String>,
    pub money_text: Option<String>,
    pub start_date: Option<Json<GqlFormattedValue>>,
    pub ownership: Option<String>,
    pub transaction_text: Option<String>,
}

// ── Net Share Purchase Activity (insider purchases) ─────────────────────────

#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlNetSharePurchaseActivity {
    pub period: Option<String>,
    pub buy_info_count: Option<Json<GqlFormattedValue>>,
    pub buy_info_shares: Option<Json<GqlFormattedValue>>,
    pub buy_percent_insider_shares: Option<Json<GqlFormattedValue>>,
    pub sell_info_count: Option<Json<GqlFormattedValue>>,
    pub sell_info_shares: Option<Json<GqlFormattedValue>>,
    pub sell_percent_insider_shares: Option<Json<GqlFormattedValue>>,
    pub net_info_count: Option<Json<GqlFormattedValue>>,
    pub net_info_shares: Option<Json<GqlFormattedValue>>,
    pub net_percent_insider_shares: Option<Json<GqlFormattedValue>>,
    pub total_insider_shares: Option<Json<GqlFormattedValue>>,
    pub max_age: Option<i64>,
}

// ── Insider Holders (insider roster) ────────────────────────────────────────

/// Not a `SimpleObject`: its only field is paginated, so a plain `#[Object]`
/// resolver replaces what would otherwise be a `SimpleObject` with zero
/// remaining (non-skipped) fields.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GqlInsiderHolders {
    pub holders: Vec<GqlInsiderHolder>,
}

#[Object(rename_fields = "camelCase")]
impl GqlInsiderHolders {
    /// Insider roster.
    async fn holders(
        &self,
        #[graphql(desc = "Max entries to return; omitted = every matching entry in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlInsiderHolder>> {
        pagination::paginate(&self.holders, first, after).await
    }
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
    pub latest_trans_date: Option<Json<GqlFormattedValue>>,
    pub position_direct: Option<Json<GqlFormattedValue>>,
    pub position_direct_date: Option<Json<GqlFormattedValue>>,
    pub position_indirect: Option<Json<GqlFormattedValue>>,
    pub position_indirect_date: Option<Json<GqlFormattedValue>>,
}
