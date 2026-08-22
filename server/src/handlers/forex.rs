use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_NEWS_VALID_FIELDS, NEWS_COMPOSITE_FIELDS, unwrap_field},
    pagination::build_connection_selection,
};
use finance_query_server::params::MarketNewsQuery;
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, connection_args, execute_gql_rest, unwrap_connection,
};

/// GET /v2/forex/news
///
/// Market-wide forex news. Currently FMP only — requires `FMP_API_KEY`.
pub(crate) async fn get_forex_news(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<MarketNewsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.page_limit, params.page_cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!(", {}", conn_args.join(", "))
    };
    let query = format!(
        "query {{ forexNews(limit: {}{}) {} }}",
        params.limit, conn_args_str, selection
    );

    info!("Fetching forex news (limit={})", params.limit);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.page_limit.is_some() || params.page_cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "forexNews"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}
