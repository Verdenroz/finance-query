use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_FOREX_QUOTE_VALID_FIELDS, GQL_NEWS_VALID_FIELDS, NEWS_COMPOSITE_FIELDS, unwrap_field,
    },
    pagination::build_connection_selection,
};
use finance_query_server::params::{ForexQuery, MarketNewsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, build_rest_selection, connection_args, execute_gql_rest,
    unwrap_connection,
};

/// GET /v2/forex/{from}/{to}
///
/// A currency pair's current exchange rate, provider-routed via
/// `Providers::forex()`.
pub(crate) async fn get_forex(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((from, to)): Path<(String, String)>,
    Query(params): Query<ForexQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_FOREX_QUOTE_VALID_FIELDS);
    let query = format!(
        "query GetForex($from: String!, $to: String!) {{ forex(from: $from, to: $to) {selection} }}"
    );
    info!(
        "Fetching forex quote for {}/{} (fields={:?})",
        from, to, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("from"), from.clone().into());
    vars.insert(Name::new("to"), to.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "forex"))).into_response()
}

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
