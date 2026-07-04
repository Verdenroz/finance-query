use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_NEWS_VALID_FIELDS, unwrap_field, unwrap_ticker_field},
};
use finance_query_server::lang;
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest};

fn default_news_count() -> u32 {
    10
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NewsQuery {
    /// Maximum number of articles to return (default: 10)
    #[serde(default = "default_news_count")]
    count: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Target language for translated text fields (BCP 47, e.g. "ja", "zh-Hant");
    /// falls back to the Accept-Language header
    lang: Option<String>,
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
    let selection = build_rest_selection(params.fields.as_deref(), GQL_NEWS_VALID_FIELDS);
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };

    let query = format!(
        "query {{ news(count: {}{}) {} }}",
        params.count, lang_arg, selection
    );
    info!("Fetching general market news (fields={:?})", params.fields);

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "news"))).into_response()
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
    let selection = build_rest_selection(params.fields.as_deref(), GQL_NEWS_VALID_FIELDS);
    let lang_arg = match &lang {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    };

    let query = format!(
        "query GetNews($symbol: String!) {{ ticker(symbol: $symbol) {{ news(count: {}{}) {} }} }}",
        params.count, lang_arg, selection
    );
    info!("Fetching news for {} (fields={:?})", symbol, params.fields);

    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "news"))).into_response()
}
