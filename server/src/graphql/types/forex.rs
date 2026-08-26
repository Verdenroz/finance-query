//! GraphQL type for a currency-pair quote, provider-routed via
//! `Providers::forex()` (`Capability::FOREX`).

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::ForexQuote`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlForexQuote {
    pub symbol: String,
    pub base_currency: Option<String>,
    pub quote_currency: Option<String>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub timestamp: Option<i64>,
}
