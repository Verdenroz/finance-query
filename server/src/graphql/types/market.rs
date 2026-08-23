//! GraphQL types for market-level data (trending, fear & greed, market
//! summary, provider-routed sector/industry performance).

use super::formatted::GqlFormattedValue;
use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

/// A trending ticker.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTrendingQuote {
    pub symbol: Option<String>,
    pub short_name: Option<String>,
    pub regular_market_price: Option<Json<GqlFormattedValue>>,
    pub regular_market_change_percent: Option<Json<GqlFormattedValue>>,
}

/// Fear & Greed index response, mirroring `finance_query::models::sentiment::FearAndGreed`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlFearAndGreed {
    pub value: Option<i32>,
    pub classification: Option<String>,
    pub timestamp: Option<i64>,
}

/// A market summary quote (major index / currency / commodity).
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMarketSummaryQuote {
    pub symbol: Option<String>,
    pub short_name: Option<String>,
    pub exchange: Option<String>,
    pub full_exchange_name: Option<String>,
    pub quote_type: Option<String>,
    pub market_state: Option<String>,
    pub regular_market_price: Option<Json<GqlFormattedValue>>,
    pub regular_market_change: Option<Json<GqlFormattedValue>>,
    pub regular_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub regular_market_previous_close: Option<Json<GqlFormattedValue>>,
    pub regular_market_time: Option<Json<GqlFormattedValue>>,
    pub spark: Option<Json<serde_json::Value>>,
}

/// A sector's aggregate performance, provider-routed via `Providers::market()`
/// (`Capability::MARKET`). Distinct from `GqlSectorPerformance` in
/// `types/sector.rs`, which is a nested field on Yahoo-only `finance::sector`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlMarketSectorPerformance {
    pub sector: String,
    pub exchange: Option<String>,
    pub change_percent: Option<f64>,
}

/// A sector's aggregate price/earnings ratio, provider-routed via
/// `Providers::market()`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlMarketSectorPe {
    pub sector: String,
    pub exchange: Option<String>,
    pub pe: Option<f64>,
    pub date: Option<String>,
}
