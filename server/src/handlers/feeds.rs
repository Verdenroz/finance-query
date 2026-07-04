use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_FEEDS_VALID_FIELDS, escape_gql_string, gql_string_list_literal, unwrap_field},
    pagination::build_connection_selection,
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, unwrap_connection};

/// Query parameters for /v2/feeds
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeedsQuery {
    /// Comma-separated source slugs (default: federal-reserve, sec, marketwatch, bloomberg)
    sources: Option<String>,
    /// SEC form type for sec-filings source (e.g., "10-K", "8-K", default: "10-K")
    form_type: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max entries to return; omitted (with cursor also omitted) = every matching
    /// entry as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// GET /v2/feeds
///
/// Query: `sources` (csv, default: all built-in), `form_type` (str, for sec-filings source)
pub(crate) async fn get_feeds(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<FeedsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_selection(params.fields.as_deref(), GQL_FEEDS_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);

    let mut args = Vec::new();
    if let Some(raw) = params.sources.as_deref() {
        let list: Vec<&str> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if !list.is_empty() {
            args.push(format!("sources: [{}]", gql_string_list_literal(&list)));
        }
    }
    if let Some(ft) = params.form_type.as_deref() {
        args.push(format!("formType: \"{}\"", escape_gql_string(ft)));
    }
    if let Some(limit) = params.limit {
        args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.cursor.as_deref() {
        args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };

    info!("Fetching feeds (sources={:?})", params.sources);

    let query = format!("query {{ feeds{args_str} {selection} }}");
    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "feeds"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}
