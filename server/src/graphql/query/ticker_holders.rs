//! Per-symbol ownership fields: major/institutional/mutual-fund holders and
//! insider transaction/purchase/roster data.
//!
//! Backed by cache-aware per-type fns in `services::holders`, shared with the
//! REST `/v2/holders/{symbol}/{type}` handler.

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::to_gql_error;
use crate::graphql::types::holders::{
    GqlFundOwnership, GqlInsiderHolders, GqlInsiderTransactions, GqlInstitutionOwnership,
    GqlMajorHoldersBreakdown, GqlNetSharePurchaseActivity,
};

pub(super) struct TickerHoldersQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerHoldersQuery {
    async fn major_holders(&self, ctx: &Context<'_>) -> Result<GqlMajorHoldersBreakdown> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_major_holders(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn institutional_holders(&self, ctx: &Context<'_>) -> Result<GqlInstitutionOwnership> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_institutional_holders(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn mutual_fund_holders(&self, ctx: &Context<'_>) -> Result<GqlFundOwnership> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_mutual_fund_holders(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn insider_transactions(&self, ctx: &Context<'_>) -> Result<GqlInsiderTransactions> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_insider_transactions(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn insider_purchases(&self, ctx: &Context<'_>) -> Result<GqlNetSharePurchaseActivity> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_insider_purchases(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    async fn insider_roster(&self, ctx: &Context<'_>) -> Result<GqlInsiderHolders> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_insider_roster(&state.cache, &self.symbol)
            .await
            .map_err(to_gql_error)?;
        serde_json::from_value(json).map_err(|e| async_graphql::Error::new(e.to_string()))
    }
}
