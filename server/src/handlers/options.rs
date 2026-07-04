use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_OPTIONS_VALID_FIELDS, OPTIONS_COMPOSITE_FIELDS, gql_string_list_literal, unwrap_field,
        unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, execute_gql_rest};

#[derive(Deserialize)]
pub(crate) struct OptionsQuery {
    date: Option<i64>, // Optional expiration timestamp
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BatchOptionsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Expiration date (Unix timestamp, optional)
    date: Option<i64>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/options/{symbol}
///
/// Query: `date` (i64, optional expiration timestamp)
pub(crate) async fn get_options(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<OptionsQuery>,
) -> impl IntoResponse {
    // Parens must be omitted entirely when there's no argument — `options()`
    // with empty parens is invalid GraphQL syntax, not "no arguments".
    let date_arg = match params.date {
        Some(ts) => format!("(date: {ts})"),
        None => String::new(),
    };
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_OPTIONS_VALID_FIELDS,
        OPTIONS_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetOpts($symbol: String!) {{ ticker(symbol: $symbol) {{ options{} {} }} }}",
        date_arg, selection
    );
    info!(
        "Fetching options for {} (fields={:?})",
        symbol, params.fields
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());
    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "options"))).into_response()
}

/// GET /v2/options?symbols=<csv>&date=<i64>
pub(crate) async fn get_batch_options(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchOptionsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal = gql_string_list_literal(&symbols);
    let date_arg = match params.date {
        Some(ts) => format!(", date: {ts}"),
        None => String::new(),
    };
    let item_selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_OPTIONS_VALID_FIELDS,
        OPTIONS_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query {{ optionsBatch(symbols: [{}]{}) {{ options {{ symbol options {} }} errors {{ symbol message }} }} }}",
        syms_literal, date_arg, item_selection
    );

    info!(
        "Fetching batch options for {} symbols (date={:?})",
        symbols.len(),
        params.date
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "optionsBatch"))).into_response()
}
