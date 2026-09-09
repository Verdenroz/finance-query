//! REST handlers for US Treasury securities auctions, served keylessly by
//! FiscalData through `Capability::ECONOMIC`.

use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{GQL_TREASURY_AUCTION_VALID_FIELDS, GQL_UPCOMING_AUCTION_VALID_FIELDS, unwrap_field},
    pagination::build_connection_selection,
};
use finance_query_server::params::{TreasuryAuctionsQuery, UpcomingAuctionsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_selection, connection_args, execute_gql_rest, unwrap_connection,
};

/// GET /v2/treasury/auctions
///
/// Query: `securityType`, `securityTerm`, `from`, `to` (`YYYY-MM-DD`),
/// `count` (overall cap fetched from Treasury), plus `fields`/`limit`/`cursor`.
pub(crate) async fn get_treasury_auctions(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<TreasuryAuctionsQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_TREASURY_AUCTION_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!(", {}", conn_args.join(", "))
    };
    let query = format!(
        "query TreasuryAuctions($securityType: String, $securityTerm: String, \
         $from: String, $to: String, $count: Int) {{ \
         treasuryAuctions(securityType: $securityType, securityTerm: $securityTerm, \
         from: $from, to: $to, count: $count{conn_args_str}) {selection} }}"
    );

    let mut vars = Variables::default();
    vars.insert(Name::new("securityType"), params.security_type.into());
    vars.insert(Name::new("securityTerm"), params.security_term.into());
    vars.insert(Name::new("from"), params.from.into());
    vars.insert(Name::new("to"), params.to.into());
    vars.insert(Name::new("count"), params.count.into());

    info!("Fetching Treasury auctions");

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "treasuryAuctions"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}

/// GET /v2/treasury/auctions/upcoming
pub(crate) async fn get_upcoming_auctions(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<UpcomingAuctionsQuery>,
) -> impl IntoResponse {
    let inner_selection =
        build_rest_selection(params.fields.as_deref(), GQL_UPCOMING_AUCTION_VALID_FIELDS);
    let selection = build_connection_selection(&inner_selection);
    let conn_args = connection_args(params.limit, params.cursor.as_deref());
    let args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };
    let query = format!("query {{ upcomingAuctions{args_str} {selection} }}");

    info!("Fetching upcoming Treasury auctions");

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let result = unwrap_connection(unwrap_field(data, "upcomingAuctions"), paginated);
    (StatusCode::OK, Json(result)).into_response()
}
