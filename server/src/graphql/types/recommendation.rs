//! GraphQL types for similar-stock recommendations.

use super::batch::GqlBatchError;
use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Result, SimpleObject};
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
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlRecommendationsBatch {
    #[graphql(skip)]
    pub recommendations: Vec<GqlRecommendation>,
    pub errors: Vec<GqlBatchError>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlRecommendationsBatch {
    /// Successfully fetched per-symbol recommendations.
    async fn recommendations(
        &self,
        #[graphql(desc = "Max symbols to return; omitted = every matching symbol in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlRecommendation>> {
        pagination::paginate(self.recommendations.clone(), first, after).await
    }
}
