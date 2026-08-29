//! Root query field for strategy backtesting.
//!
//! Its own `MergedObject` piece so the simulation's argument list does not
//! widen another piece's async-graphql dispatch fn (see the module doc on
//! `QueryRoot`).

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::exec_gql;
use crate::graphql::types::backtest::{
    BACKTEST_DEFAULT_INTERVAL, BACKTEST_DEFAULT_RANGE, GqlBacktestParams, GqlBacktestResult,
    GqlStrategy,
};
use crate::graphql::types::enums::{GqlInterval, GqlTimeRange};

#[derive(Default)]
pub(super) struct RootBacktestQuery;

#[Object]
impl RootBacktestQuery {
    /// Simulate `strategy` over `symbol`'s bars and report the resulting
    /// trades, equity curve and performance metrics.
    ///
    /// `equityCurve` and `trades` paginate independently; both are ordered
    /// oldest first.
    async fn backtest(
        &self,
        ctx: &Context<'_>,
        symbol: String,
        strategy: GqlStrategy,
        #[graphql(default_with = "BACKTEST_DEFAULT_INTERVAL")] interval: GqlInterval,
        #[graphql(default_with = "BACKTEST_DEFAULT_RANGE")] range: GqlTimeRange,
        #[graphql(default)] params: GqlBacktestParams,
    ) -> Result<GqlBacktestResult> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::backtest::run_backtest(
            &state.cache,
            &symbol,
            strategy,
            interval,
            range,
            params,
        ))
        .await
    }
}
