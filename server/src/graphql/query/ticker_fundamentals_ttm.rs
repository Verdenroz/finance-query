//! Per-symbol short-sale, grading, compensation and trailing-twelve-month
//! fields, in their own `GqlTicker` `MergedObject` piece to keep each
//! dispatch fn small (see the module doc on `QueryRoot`).

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::{exec_gql, from_gql_json, to_gql_error};
use crate::graphql::pagination::{self, Page};
use crate::graphql::types::fundamentals_ttm::{
    GqlExecutiveCompensation, GqlFinancialRatiosTtm, GqlGradingAction, GqlKeyMetricsTtm,
    GqlPriceTargetSummary, GqlShortVolume,
};

pub(super) struct TickerFundamentalsTtmQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerFundamentalsTtmQuery {
    /// Daily FINRA short-sale volume, oldest first. Keyless.
    async fn short_volume(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max entries per page; omitted = every fetched record in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlShortVolume>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::filings::get_short_volume(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlShortVolume> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    /// Analyst upgrades and downgrades, provider-routed. Distinct from
    /// `gradingHistory`, which is Yahoo's own quote-summary module.
    async fn grading_actions(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max entries per page; omitted = every fetched record in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlGradingAction>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::analysis::get_grading_actions(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlGradingAction> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    /// Disclosed executive compensation by year.
    async fn executive_compensation(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max entries per page; omitted = every fetched record in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlExecutiveCompensation>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::holders::get_executive_compensation(
            &state.cache,
            &state.providers,
            &self.symbol,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<GqlExecutiveCompensation> = from_gql_json(json)?;
        pagination::paginate(&entries, first, after).await
    }

    /// Analyst price-target publication counts and averages per window.
    async fn price_target_summary(&self, ctx: &Context<'_>) -> Result<GqlPriceTargetSummary> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_price_target_summary(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }

    /// Trailing-twelve-month key metrics.
    async fn key_metrics_ttm(&self, ctx: &Context<'_>) -> Result<GqlKeyMetricsTtm> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::financials::get_key_metrics_ttm(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }

    /// Trailing-twelve-month financial ratios.
    async fn ratios_ttm(&self, ctx: &Context<'_>) -> Result<GqlFinancialRatiosTtm> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::financials::get_ratios_ttm(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }
}
