//! GraphQL types for CoinGecko cryptocurrency quotes.

use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use serde::Deserialize;

use crate::AppState;
use crate::graphql::error::{exec_gql, from_gql_json, to_gql_error};
use crate::graphql::pagination::{self, Page};

/// A cryptocurrency quote from CoinGecko, mirroring `finance_query::CoinQuote`,
/// which has no serde rename of its own (plain snake_case JSON keys) — must
/// not rename for deserialization either, even though GraphQL field names
/// are camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(default)]
pub struct GqlCoinQuote {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub total_volume: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub image: Option<String>,
    pub market_cap_rank: Option<u32>,
}

/// Mirrors `finance_query::crypto::TrendingCoin` — plain snake_case JSON keys,
/// same no-serde-rename rule as [`GqlCoinQuote`].
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlTrendingCoin {
    pub id: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub market_cap_rank: Option<u32>,
    pub price_btc: Option<f64>,
    pub score: Option<u32>,
}

/// Mirrors `finance_query::crypto::SymbolMatch` — the provider-neutral
/// discovery shape, here populated from CoinGecko's coin universe.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlSymbolMatch {
    pub symbol: String,
    pub id: Option<String>,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub asset_type: Option<String>,
    pub currency: Option<String>,
    pub active: Option<bool>,
    pub market_cap_rank: Option<u32>,
    pub thumbnail: Option<String>,
    pub image: Option<String>,
}

/// Mirrors `finance_query::crypto::GlobalCryptoStats`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlGlobalCryptoStats {
    pub active_cryptocurrencies: Option<u32>,
    pub markets: Option<u32>,
    pub total_market_cap_usd: Option<f64>,
    pub total_volume_usd: Option<f64>,
    pub btc_dominance: Option<f64>,
    pub eth_dominance: Option<f64>,
    pub market_cap_change_percentage_24h_usd: Option<f64>,
}

/// One chain's share of a protocol's TVL.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlChainAllocation {
    pub chain: String,
    pub tvl: f64,
}

/// A DeFi protocol's total value locked, provider-routed (DefiLlama, keyless).
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlProtocolTvl {
    pub slug: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub url: Option<String>,
    pub chains: Vec<String>,
    pub tvl: Option<f64>,
    pub tvl_by_chain: Vec<GqlChainAllocation>,
    pub change_1d_percent: Option<f64>,
    pub change_7d_percent: Option<f64>,
    pub market_cap: Option<f64>,
}

/// One point on a protocol's TVL history.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlTvlPoint {
    pub timestamp: i64,
    pub tvl: f64,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlCoinQuote {
    /// Total value locked for this coin's protocol, provider-routed
    /// (DefiLlama, keyless). Resolved only when selected.
    async fn tvl(&self, ctx: &Context<'_>) -> Result<GqlProtocolTvl> {
        let state = ctx.data::<AppState>()?;
        exec_gql(crate::services::crypto::get_protocol_tvl(
            &state.cache,
            &state.providers,
            &self.id,
        ))
        .await
    }

    /// TVL history for this coin's protocol, oldest first.
    async fn tvl_history(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Max points per page; omitted = every fetched point in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlTvlPoint>> {
        let state = ctx.data::<AppState>()?;
        let json = crate::services::crypto::get_protocol_tvl_history(
            &state.cache,
            &state.providers,
            &self.id,
        )
        .await
        .map_err(to_gql_error)?;
        let points: Vec<GqlTvlPoint> = from_gql_json(json)?;
        pagination::paginate(&points, first, after).await
    }
}
