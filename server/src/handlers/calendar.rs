use async_graphql::Variables;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        CALENDAR_EVENT_UNION_SELECTION, GQL_CALENDAR_VALID_FIELDS, gql_string_list_literal,
        unwrap_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, range_to_gql};

fn default_calendar_range() -> String {
    "1mo".to_string()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalendarQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    /// Forward window: 1wk/5d, 1mo, 3mo, etc. (default: 1mo)
    #[serde(default = "default_calendar_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Build the `calendar { ... }` selection set, expanding `event` with its
/// full union inline-fragment selection (see `CALENDAR_EVENT_UNION_SELECTION`).
fn build_rest_calendar_selection(fields: Option<&str>) -> String {
    let top_selection = build_rest_selection(fields, GQL_CALENDAR_VALID_FIELDS);
    if !top_selection.contains("event") {
        return top_selection;
    }
    let mut sel = String::from("{ ");
    for f in ["timestamp", "date", "symbol"] {
        if top_selection.contains(f) {
            sel.push_str(f);
            sel.push(' ');
        }
    }
    sel.push_str("event ");
    sel.push_str(CALENDAR_EVENT_UNION_SELECTION);
    sel.push_str(" }");
    sel
}

/// GET /v2/calendar?symbols=<csv>&range=<str>
pub(crate) async fn get_calendar(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CalendarQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let syms_literal = gql_string_list_literal(&symbols);
    let gql_range = range_to_gql(&params.range);
    let selection = build_rest_calendar_selection(params.fields.as_deref());

    let query = format!(
        "query {{ calendar(symbols: [{}], range: {}) {} }}",
        syms_literal, gql_range, selection
    );

    info!(
        "Fetching calendar for {} symbols (range={})",
        symbols.len(),
        params.range
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "calendar"))).into_response()
}
