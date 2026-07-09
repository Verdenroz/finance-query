//! Per-symbol corporate-events and quant fields: dividends, splits, capital
//! gains, technical indicators, and risk analytics.

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::exec_gql;
use crate::graphql::types::{
    enums::{GqlInterval, GqlTimeRange},
    events::{GqlCapitalGain, GqlDividends, GqlSplit},
    indicators::GqlIndicatorsSummary,
    risk::GqlRiskSummary,
};

pub(super) struct TickerEventsQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerEventsQuery {
    async fn dividends(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<GqlDividends> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::events::get_dividends(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        ))
        .await
    }

    async fn splits(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<Vec<GqlSplit>> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::events::get_splits(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        ))
        .await
    }

    async fn capital_gains(
        &self,
        ctx: &Context<'_>,
        #[graphql(default_with = "GqlTimeRange::Max")] range: GqlTimeRange,
    ) -> Result<Vec<GqlCapitalGain>> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::events::get_capital_gains(
            &state.cache,
            &self.symbol,
            range.into(),
            range.as_str(),
        ))
        .await
    }

    async fn indicators(
        &self,
        ctx: &Context<'_>,
        interval: GqlInterval,
        range: GqlTimeRange,
    ) -> Result<GqlIndicatorsSummary> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::indicators::get_indicators(
            &state.cache,
            &self.symbol,
            interval.into(),
            interval.as_str(),
            range.into(),
            range.as_str(),
        ))
        .await
    }

    async fn risk(
        &self,
        ctx: &Context<'_>,
        interval: GqlInterval,
        range: GqlTimeRange,
        benchmark: Option<String>,
    ) -> Result<GqlRiskSummary> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::risk::get_risk(
            &state.cache,
            &self.symbol,
            interval.into(),
            interval.as_str(),
            range.into(),
            range.as_str(),
            benchmark.as_deref(),
        ))
        .await
    }
}
