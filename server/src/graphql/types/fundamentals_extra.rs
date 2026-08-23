//! GraphQL types for the FUNDAMENTALS-capability analyst-consensus and ETF
//! profile operations — provider-routed, distinct from Yahoo's quote-module
//! `recommendationTrend`/`gradingHistory` fields.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Mirrors `finance_query::RatingConsensus`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlRatingConsensus {
    pub symbol: Option<String>,
    pub strong_buy: Option<i64>,
    pub buy: Option<i64>,
    pub hold: Option<i64>,
    pub sell: Option<i64>,
    pub strong_sell: Option<i64>,
    pub consensus: Option<String>,
}

/// Mirrors `finance_query::PriceTargetConsensus`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlPriceTargetConsensus {
    pub symbol: Option<String>,
    pub target_high: Option<f64>,
    pub target_low: Option<f64>,
    pub target_consensus: Option<f64>,
    pub target_median: Option<f64>,
}

/// Mirrors `finance_query::EtfProfile`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlEtfProfile {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub asset_type: Option<String>,
    pub net_assets: Option<f64>,
    pub net_expense_ratio: Option<f64>,
    pub portfolio_turnover: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub inception_date: Option<String>,
    pub holdings: Vec<GqlEtfHolding>,
    pub sector_weightings: Vec<GqlEtfSectorWeighting>,
    pub country_weightings: Vec<GqlEtfCountryWeighting>,
}

/// Mirrors `finance_query::EtfHolding`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlEtfHolding {
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub weight: Option<f64>,
}

/// Mirrors `finance_query::EtfSectorWeighting`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlEtfSectorWeighting {
    pub sector: Option<String>,
    pub weight: Option<f64>,
}

/// Mirrors `finance_query::EtfCountryWeighting`.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(default)]
pub struct GqlEtfCountryWeighting {
    pub country: Option<String>,
    pub weight: Option<f64>,
}
