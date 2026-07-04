use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_OPTIONS_VALID_FIELDS, OPTIONS_COMPOSITE_FIELDS, escape_gql_string,
        gql_string_list_literal, unwrap_field, unwrap_ticker_field,
    },
    pagination::{
        build_connection_selection, connection_nodes, connection_page_info,
        unwrap_nested_connection,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::execute_gql_rest;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OptionsQuery {
    date: Option<i64>, // Optional expiration timestamp
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max contracts per side per page; omitted (with cursor also omitted) =
    /// every matching contract as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    /// (applied to both `calls` and `puts`)
    cursor: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BatchOptionsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Expiration date (Unix timestamp, optional)
    date: Option<i64>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
    /// Max symbols per page; omitted (with cursor also omitted) = every requested
    /// symbol's options chain as a bare array, unchanged from pre-pagination behavior
    limit: Option<u32>,
    /// Opaque continuation cursor from a previous response's `pageInfo.endCursor`
    cursor: Option<String>,
}

/// Build the options `{ ... }` selection, expanding `calls`/`puts` as paginated
/// Connections sharing the same `first`/`after` args (both sides of the chain
/// page together). `limit`/`cursor` are `None` for batch callers (which don't
/// expose pagination params for the nested chain).
fn build_options_selection(
    fields: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    let mut chosen: Vec<&str> = match fields {
        Some(raw) if !raw.trim().is_empty() => raw
            .split(',')
            .map(|s| s.trim())
            .filter(|f| !f.is_empty() && GQL_OPTIONS_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_OPTIONS_VALID_FIELDS.to_vec(),
    };
    if chosen.is_empty() {
        chosen = GQL_OPTIONS_VALID_FIELDS.to_vec();
    }
    let mut args = Vec::new();
    if let Some(limit) = limit {
        args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let mut sel = String::from("{ ");
    for f in chosen {
        sel.push_str(f);
        if f == "calls" || f == "puts" {
            let item_selection = OPTIONS_COMPOSITE_FIELDS
                .iter()
                .find(|(name, _)| *name == f)
                .map(|(_, s)| *s)
                .unwrap_or("{ }");
            sel.push_str(&args_str);
            sel.push(' ');
            sel.push_str(&build_connection_selection(item_selection));
        }
        sel.push(' ');
    }
    sel.push('}');
    sel
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
    let selection = build_options_selection(
        params.fields.as_deref(),
        params.limit,
        params.cursor.as_deref(),
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
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let mut result = unwrap_ticker_field(data, "options");
    result = unwrap_nested_connection(result, "calls", paginated);
    result = unwrap_nested_connection(result, "puts", paginated);
    (StatusCode::OK, Json(result)).into_response()
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
    let item_selection = format!(
        "{{ symbol options {} }}",
        build_options_selection(params.fields.as_deref(), None, None)
    );
    let selection = build_connection_selection(&item_selection);

    let mut conn_args = Vec::new();
    if let Some(limit) = params.limit {
        conn_args.push(format!("first: {limit}"));
    }
    if let Some(cursor) = params.cursor.as_deref() {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    let conn_args_str = if conn_args.is_empty() {
        String::new()
    } else {
        format!("({})", conn_args.join(", "))
    };

    let query = format!(
        "query {{ optionsBatch(symbols: [{}]{}) {{ options{} {} errors {{ symbol message }} }} }}",
        syms_literal, date_arg, conn_args_str, selection
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
    let mut outer = unwrap_field(data, "optionsBatch");
    let options_conn = outer
        .as_object_mut()
        .and_then(|obj| obj.get("options").cloned())
        .unwrap_or(serde_json::Value::Null);
    let mut nodes = connection_nodes(&options_conn);
    for item in nodes.iter_mut() {
        if let Some(opts) = item.get("options").cloned() {
            let opts = unwrap_nested_connection(opts, "calls", false);
            let opts = unwrap_nested_connection(opts, "puts", false);
            item["options"] = opts;
        }
    }
    let paginated = params.limit.is_some() || params.cursor.is_some();
    let options_value = if paginated {
        serde_json::json!({ "items": nodes, "pageInfo": connection_page_info(&options_conn) })
    } else {
        serde_json::Value::Array(nodes)
    };
    if let Some(obj) = outer.as_object_mut() {
        obj.insert("options".to_string(), options_value);
    }
    (StatusCode::OK, Json(outer)).into_response()
}
