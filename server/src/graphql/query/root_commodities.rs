//! Market-wide root query fields for commodities, futures, index
//! constituents, the market-wide event calendar, and sector performance/PE.
//! Split out of `root_market.rs` to keep each `MergedObject` piece's
//! async-graphql dispatch fn small (see the module doc on `QueryRoot`).

use async_graphql::{Context, Object, Result};

use crate::AppState;
use crate::graphql::error::{from_gql_json, to_gql_error};
use crate::graphql::types::{
    commodity_futures::{GqlCommodityQuote, GqlFuturesQuote, GqlIndexConstituent},
    market::{GqlMarketSectorPe, GqlMarketSectorPerformance},
    market_calendar::{GqlCalendarKind, GqlMarketCalendarEntry},
};

#[derive(Default)]
pub(super) struct RootCommodityQuery;

#[Object]
impl RootCommodityQuery {
    /// Market-wide event calendar (earnings/IPO/dividend/split/economic/
    /// holiday/status) over `[from, to]` (`YYYY-MM-DD`). `from`/`to` are
    /// ignored by `MARKET_HOLIDAY`/`MARKET_STATUS`, which return providers'
    /// upcoming/live sets regardless.
    async fn market_calendar(
        &self,
        ctx: &Context<'_>,
        kind: GqlCalendarKind,
        #[graphql(default)] from: String,
        #[graphql(default)] to: String,
    ) -> Result<Vec<GqlMarketCalendarEntry>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market_calendar::get_market_calendar(
            &state.cache,
            &state.providers,
            kind.into(),
            &from,
            &to,
        )
        .await
        .map_err(to_gql_error)?;
        let entries: Vec<finance_query::MarketCalendarEntry> = from_gql_json(json)?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    /// A commodity's current quote (e.g. gold, silver, crude oil),
    /// provider-routed via `Providers::commodity()` (Yahoo, keyless).
    async fn commodity(&self, ctx: &Context<'_>, symbol: String) -> Result<GqlCommodityQuote> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::commodity_futures::get_commodity_quote(
            &state.cache,
            &state.providers,
            &symbol,
        )
        .await
        .map_err(to_gql_error)?;
        from_gql_json(json)
    }

    /// A futures contract's current quote, provider-routed via
    /// `Providers::futures()` (Yahoo, keyless).
    async fn futures(&self, ctx: &Context<'_>, symbol: String) -> Result<GqlFuturesQuote> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::commodity_futures::get_futures_quote(
            &state.cache,
            &state.providers,
            &symbol,
        )
        .await
        .map_err(to_gql_error)?;
        from_gql_json(json)
    }

    /// An index's current constituent list, provider-routed via
    /// `Providers::index()` (Wikipedia, S&P 500 only).
    async fn index_constituents(
        &self,
        ctx: &Context<'_>,
        symbol: String,
    ) -> Result<Vec<GqlIndexConstituent>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::commodity_futures::get_index_constituents(
            &state.cache,
            &state.providers,
            &symbol,
        )
        .await
        .map_err(to_gql_error)?;
        from_gql_json(json)
    }

    /// Aggregate performance for every sector, provider-routed via
    /// `Providers::market()` (Yahoo screener fan-out, keyless). Distinct from
    /// `sector`, a per-sector Yahoo-only shortcut.
    async fn sector_performance(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<GqlMarketSectorPerformance>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market::get_sector_performance(&state.cache, &state.providers)
            .await
            .map_err(to_gql_error)?;
        from_gql_json(json)
    }

    /// Price/earnings ratios by sector, provider-routed via
    /// `Providers::market()` (Yahoo screener fan-out, keyless).
    async fn sector_pe(&self, ctx: &Context<'_>) -> Result<Vec<GqlMarketSectorPe>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::market::get_sector_pe(&state.cache, &state.providers)
            .await
            .map_err(to_gql_error)?;
        from_gql_json(json)
    }
}
