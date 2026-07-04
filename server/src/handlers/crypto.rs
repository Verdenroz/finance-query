use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_COIN_VALID_FIELDS, unwrap_field},
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest};

fn default_vs_currency() -> String {
    "usd".to_string()
}

fn default_crypto_count() -> usize {
    50
}

/// Query parameters for /v2/crypto/coins
#[derive(Deserialize)]
pub(crate) struct CryptoCoinsQuery {
    /// Currency to compare against (default: "usd")
    #[serde(default = "default_vs_currency")]
    vs_currency: String,
    /// Number of coins to return (default: 50)
    #[serde(default = "default_crypto_count")]
    count: usize,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/crypto/coins/{id}
#[derive(Deserialize)]
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
    let selection = build_rest_selection(params.fields.as_deref(), GQL_COIN_VALID_FIELDS);
    let query = format!(
        "query {{ cryptoCoins(vsCurrency: \"{}\", count: {}) {} }}",
        params.vs_currency, params.count, selection
    );

    info!(
        "Fetching top {} crypto coins (vs {})",
        params.count, params.vs_currency
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "cryptoCoins"))).into_response()
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
