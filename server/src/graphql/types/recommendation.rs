//! GraphQL types for similar-stock recommendations.

use super::batch::GqlBatchError;
use async_graphql::SimpleObject;
use serde::Deserialize;

/// A single recommended/similar symbol with its similarity score.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSimilarSymbol {
    pub symbol: String,
    pub score: f64,
}

/// Similar-stock recommendations for one symbol, mirroring
/// `finance_query::Recommendation`, which has no serde rename of its own
/// (plain snake_case JSON keys, e.g. `provider_id`) — must not rename for
/// deserialization either, even though GraphQL field names are camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlRecommendation {
    pub symbol: String,
    pub recommendations: Vec<GqlSimilarSymbol>,
    pub provider_id: Option<String>,
}

/// Result of the batch `recommendationsBatch` root field: successfully
/// fetched recommendations plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlRecommendationsBatch {
    pub recommendations: Vec<GqlRecommendation>,
    pub errors: Vec<GqlBatchError>,
}
