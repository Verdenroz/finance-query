//! Per-symbol consensus/ETF fields split out of `ticker_analysis.rs` to keep
//! each `GqlTicker` `MergedObject` piece's dispatch fn small (see the module
//! doc on `QueryRoot`).

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::exec_gql;
use crate::graphql::types::fundamentals_extra::{
    GqlEtfProfile, GqlPriceTargetConsensus, GqlRatingConsensus,
};

pub(super) struct TickerFundamentalsExtraQuery {
    pub(super) symbol: String,
}

#[Object]
impl TickerFundamentalsExtraQuery {
    /// Consensus rating rollup (analyst grade distribution + headline label).
    async fn rating_consensus(&self, ctx: &Context<'_>) -> Result<GqlRatingConsensus> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_rating_consensus(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }

    /// Consensus analyst price target.
    async fn price_target_consensus(&self, ctx: &Context<'_>) -> Result<GqlPriceTargetConsensus> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_price_target_consensus(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }

    /// ETF profile and holdings (only meaningful for ETF symbols).
    async fn etf_profile(&self, ctx: &Context<'_>) -> Result<GqlEtfProfile> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::analysis::get_etf_profile(
            &state.cache,
            &state.providers,
            &self.symbol,
        ))
        .await
    }
}
