use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_INSIDER_PURCHASES_VALID_FIELDS, GQL_INSIDER_ROSTER_COMPOSITE,
        GQL_INSIDER_ROSTER_VALID_FIELDS, GQL_INSIDER_TRANSACTIONS_COMPOSITE,
        GQL_INSIDER_TRANSACTIONS_VALID_FIELDS, GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS,
        GQL_MAJOR_HOLDERS_VALID_FIELDS, GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS, GQL_OWNER_FIELDS,
        unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{RestTypeSpec, build_rest_composite_selection, execute_gql_rest};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HoldersQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Per-holder-type spec. The first element must stay in sync with every
/// `services::holders::HolderType` variant and its corresponding GraphQL field.
const HOLDER_TYPE_REST_SPECS: &[RestTypeSpec] = &[
    ("major", "majorHolders", GQL_MAJOR_HOLDERS_VALID_FIELDS, &[]),
    (
        "institutional",
        "institutionalHolders",
        GQL_INSTITUTIONAL_HOLDERS_VALID_FIELDS,
        &[("ownershipList", GQL_OWNER_FIELDS)],
    ),
    (
        "mutualfund",
        "mutualFundHolders",
        GQL_MUTUAL_FUND_HOLDERS_VALID_FIELDS,
        &[("ownershipList", GQL_OWNER_FIELDS)],
    ),
    (
        "insider-transactions",
        "insiderTransactions",
        GQL_INSIDER_TRANSACTIONS_VALID_FIELDS,
        &[("transactions", GQL_INSIDER_TRANSACTIONS_COMPOSITE)],
    ),
    (
        "insider-purchases",
        "insiderPurchases",
        GQL_INSIDER_PURCHASES_VALID_FIELDS,
        &[],
    ),
    (
        "insider-roster",
        "insiderRoster",
        GQL_INSIDER_ROSTER_VALID_FIELDS,
        &[("holders", GQL_INSIDER_ROSTER_COMPOSITE)],
    ),
];

/// GET /v2/holders/{symbol}/{holder_type}
///
/// Path params:
/// - `holder_type`: major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster
///
/// Query: `fields` (comma-separated, optional)
pub(crate) async fn get_holders(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path((symbol, holder_type)): Path<(String, String)>,
    Query(params): Query<HoldersQuery>,
) -> impl IntoResponse {
    let key = holder_type.to_lowercase();
    let Some((_, gql_field, valid_fields, composite_fields)) = HOLDER_TYPE_REST_SPECS
        .iter()
        .find(|(k, ..)| *k == key.as_str())
    else {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": format!("Invalid holder type: '{}'. Valid: major, institutional, mutualfund, insider-transactions, insider-purchases, insider-roster", holder_type),
            "status": 400
        }))).into_response();
    };
    let selection =
        build_rest_composite_selection(params.fields.as_deref(), valid_fields, composite_fields);
    let query = format!(
        "query GetHolders($symbol: String!) {{ ticker(symbol: $symbol) {{ {gql_field} {selection} }} }}"
    );
    info!(
        "Fetching {} holders for {} (fields={:?})",
        holder_type, symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, gql_field))).into_response()
}
