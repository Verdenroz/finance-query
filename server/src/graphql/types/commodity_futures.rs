//! GraphQL types for per-symbol commodity and futures quotes, provider-routed
//! via `Providers::commodity()`/`Providers::futures()` (`Capability::COMMODITIES`/
//! `Capability::FUTURES`), both Yahoo keyless. Also index constituents,
//! provider-routed via `Providers::index()` (`Capability::INDICES`, Wikipedia).

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::CommodityQuote`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlCommodityQuote {
    pub symbol: String,
    pub name: Option<String>,
    pub unit: Option<String>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub timestamp: Option<i64>,
}

/// Mirrors `finance_query::FuturesQuote`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFuturesQuote {
    pub symbol: String,
    pub name: Option<String>,
    pub underlying: Option<String>,
    pub exchange: Option<String>,
    pub expiration_date: Option<String>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub open_interest: Option<u64>,
    pub volume: Option<u64>,
    pub timestamp: Option<i64>,
}

/// Mirrors `finance_query::IndexConstituent`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlIndexConstituent {
    pub symbol: String,
    pub name: Option<String>,
    pub sector: Option<String>,
    pub sub_sector: Option<String>,
    pub headquarters: Option<String>,
    pub date_first_added: Option<String>,
    pub cik: Option<String>,
    pub founded: Option<String>,
}
