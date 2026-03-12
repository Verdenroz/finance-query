//! GraphQL types for market-level data (trending, fear & greed, market summary).

use async_graphql::{Json, SimpleObject};
use serde::Deserialize;

/// A trending ticker.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlTrendingQuote {
    pub symbol: Option<String>,
    pub short_name: Option<String>,
    pub regular_market_price: Option<Json<serde_json::Value>>,
    pub regular_market_change_percent: Option<Json<serde_json::Value>>,
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
    pub full_exchange_name: Option<String>,
    pub regular_market_price: Option<Json<serde_json::Value>>,
    pub regular_market_change: Option<Json<serde_json::Value>>,
    pub regular_market_change_percent: Option<Json<serde_json::Value>>,
}
