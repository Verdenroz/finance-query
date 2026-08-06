use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_COIN_VALID_FIELDS, GQL_CRYPTO_SYMBOL_MATCH_VALID_FIELDS,
        GQL_GLOBAL_CRYPTO_STATS_VALID_FIELDS, GQL_TRENDING_COIN_VALID_FIELDS, unwrap_field,
    },
    pagination::build_connection_selection,
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{
    build_rest_selection, connection_args, execute_gql_rest, unwrap_connection,
};

fn default_vs_currency() -> String {
    "usd".to_string()
}

fn default_crypto_count() -> usize {
    50
}

/// Query parameters for /v2/crypto/coins
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CryptoCoinsQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    vs_currency: String,
    /// Number of coins to return (default: 50)
    #[serde(default = "default_crypto_count")]
    count: usize,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max coins per page; omitted (with cursor also omitted) = every fetched
    /// coin (up to `count`) as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// Query parameters for /v2/crypto/coins/{id}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CryptoCoinQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    vs_currency: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/crypto/coins
///
/// Query: `vs_currency` (str, default "usd"), `count` (u32, default 50)
pub(crate) async fn get_crypto_coins(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CryptoCoinsQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_selection(params.fields.as_deref(), GQL_COIN_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!(", {}", conn_args.join(", "))
    };
    let query = format!(
        "query {{ cryptoCoins(vsCurrency: \"{}\", count: {}{}) {} }}",
        params.vs_currency, params.count, conn_args_str, selection
    );

    info!(
        "Fetching top {} crypto coins (vs {})",
        params.count, params.vs_currency
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "cryptoCoins"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/crypto/coins/{id}
///
/// Query: `vs_currency` (str, default "usd")
pub(crate) async fn get_crypto_coin(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(coin_id): Path<String>,
    Query(params): Query<CryptoCoinQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_COIN_VALID_FIELDS);
    let query = format!(
        "query GetCoin($id: String!) {{ cryptoCoin(id: $id, vsCurrency: \"{}\") {} }}",
        params.vs_currency, selection
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("id"), coin_id.clone().into());

    info!(
        "Fetching crypto coin: {} (vs {})",
        coin_id, params.vs_currency
    );

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "cryptoCoin"))).into_response()
}

/// Query parameters for /v2/crypto/trending
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CryptoTrendingQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max coins per page; omitted (with cursor also omitted) = bare array
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// Query parameters for /v2/crypto/global
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CryptoGlobalQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/crypto/trending
pub(crate) async fn get_crypto_trending(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CryptoTrendingQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_TRENDING_COIN_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!("query {{ cryptoTrending{conn_args_str} {selection} }}");

    info!("Fetching trending crypto coins");

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "cryptoTrending"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// Query parameters for /v2/crypto/search
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CryptoSearchQuery {
    /// Free-text query (coin name, symbol, or CoinGecko id)
    query: String,
    /// Max matches to fetch (default: 25)
    #[serde(default = "default_crypto_search_limit")]
    limit_results: u32,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max matches per page; omitted (with cursor also omitted) = bare array
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

fn default_crypto_search_limit() -> u32 {
    25
}

/// GET /v2/crypto/search
pub(crate) async fn get_crypto_search(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CryptoSearchQuery>,
) -> impl IntoResponse {
    let inner_selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_CRYPTO_SYMBOL_MATCH_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!(", {}", conn_args.join(", "))
    };
    let query = format!(
        "query Search($q: String!) {{ cryptoSearch(query: $q, limit: {}{}) {} }}",
        params.limit_results, conn_args_str, selection
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("q"), params.query.clone().into());

    info!("Searching CoinGecko for: {}", params.query);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "cryptoSearch"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/crypto/global
pub(crate) async fn get_crypto_global(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CryptoGlobalQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(
        params.fields.as_deref(),
        GQL_GLOBAL_CRYPTO_STATS_VALID_FIELDS,
    );
    let query = format!("query {{ cryptoGlobal {selection} }}");

    info!("Fetching global crypto market stats");

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "cryptoGlobal"))).into_response()
}
