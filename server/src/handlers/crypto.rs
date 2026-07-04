use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_COIN_VALID_FIELDS, unwrap_field},
    pagination::build_connection_selection,
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, unwrap_connection};

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
    let mut conn_args = Vec::new();
    if let Some(limit) = params.limit {
        conn_args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.cursor.as_deref() {
        conn_args.push(format!(
            "after: \"{}\"",
            cursor.replace('\\', "\\\\").replace('"', "\\\"")
        ));
    }
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
