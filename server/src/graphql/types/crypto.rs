//! GraphQL types for CoinGecko cryptocurrency quotes.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// A cryptocurrency quote from CoinGecko, mirroring `finance_query::CoinQuote`,
/// which has no serde rename of its own (plain snake_case JSON keys) — must
/// not rename for deserialization either, even though GraphQL field names
/// are camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
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
