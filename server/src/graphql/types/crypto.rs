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
