use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_NEWS_VALID_FIELDS, GQL_PRESS_RELEASE_VALID_FIELDS, NEWS_COMPOSITE_FIELDS,
        escape_gql_string, unwrap_field, unwrap_ticker_field,
    },
    pagination::build_connection_selection,
};
use finance_query_server::lang;
use finance_query_server::params::{NewsQuery, PressReleasesQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_composite_selection, build_rest_selection, execute_gql_rest, unwrap_connection,
};

fn connection_args(params: &NewsQuery) -> String {
    let mut args = Vec::new();
    if let Some(limit) = params.limit {
        args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.cursor.as_deref() {
        args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    }
}

/// GET /v2/news
///
/// Returns general market news
pub(crate) async fn get_general_news(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<NewsQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let inner_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let conn_args = connection_args(&params);

    let query = format!(
        "query {{ news(count: {}{}{}) {} }}",
        params.count, lang_arg, conn_args, selection
    );
    info!("Fetching general market news (fields={:?})", params.fields);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "news"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/news/{symbol}
///
/// Returns news articles for a specific symbol.
pub(crate) async fn get_news(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<NewsQuery>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let lang = lang::resolve_lang(params.lang.as_deref(), &headers);
    let inner_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };
    let conn_args = connection_args(&params);

    let query = format!(
        "query GetNews($symbol: String!) {{ ticker(symbol: $symbol) {{ news(count: {}{}{}) {} }} }}",
        params.count, lang_arg, conn_args, selection
    );
    info!("Fetching news for {} (fields={:?})", symbol, params.fields);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_ticker_field(data, "news"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/press-releases/{symbol}
///
/// This company's own press releases, distinct from `/v2/news/{symbol}`
/// (press coverage). Routes through EDGAR 8-K exhibits, falling back to
/// FMP/Alpha Vantage when configured.
pub(crate) async fn get_press_releases(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<PressReleasesQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_PRESS_RELEASE_VALID_FIELDS);
    let query = format!(
        "query GetPressReleases($symbol: String!) {{ ticker(symbol: $symbol) {{ pressReleases(limit: {}) {selection} }} }}",
        params.limit
    );
    info!(
        "Fetching press releases for {} (limit={}, fields={:?})",
        symbol, params.limit, params.fields
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    (
        StatusCode::OK,
        Json(unwrap_ticker_field(data, "pressReleases")),
    )
        .into_response()
}
