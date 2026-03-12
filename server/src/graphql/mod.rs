//! GraphQL schema, handlers, and supporting infrastructure.
//!
//! Exposes:
//! - `POST /graphql`   — query / mutation endpoint
//! - `GET  /graphql`   — Apollo Sandbox / Playground UI
//! - `GET  /graphql/ws`— WebSocket subscription endpoint (graphql-ws protocol)

pub mod error;
pub mod query;
pub mod subscription;
pub mod types;

use async_graphql::{EmptyMutation, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    Extension, Router,
    response::{Html, IntoResponse},
    routing::post,
};

use crate::AppState;
use query::QueryRoot;
use subscription::SubscriptionRoot;

/// Alias for the fully-typed schema.
pub type FinanceSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

/// Build the GraphQL schema and attach `AppState` as shared data.
pub fn build_schema(state: AppState) -> FinanceSchema {
    Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .data(state)
        .limit_depth(20)
        .limit_complexity(2000)
        .finish()
}

/// HTTP handler: execute a GraphQL query or mutation.
async fn graphql_handler(schema: Extension<FinanceSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// HTTP handler: serve the GraphiQL IDE.
async fn graphql_playground() -> impl IntoResponse {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql/ws")
            .finish(),
    )
}

/// Build the `/graphql` sub-router. The `FinanceSchema` extension must already
/// be present on the parent router (added in `create_app`).
pub fn graphql_routes(schema: FinanceSchema) -> Router {
    Router::new()
        .route("/graphql", post(graphql_handler).get(graphql_playground))
        .route_service("/graphql/ws", GraphQLSubscription::new(schema))
}
