//! Shared per-symbol error type for batch GraphQL root fields (`quotes`,
//! `charts`, `dividendsBatch`, `indicatorsBatch`, `financialsBatch`).

use async_graphql::SimpleObject;

/// A single symbol's fetch failure within a batch root field.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlBatchError {
    pub symbol: String,
    pub message: String,
}
