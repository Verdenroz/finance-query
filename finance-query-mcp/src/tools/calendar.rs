use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    CALENDAR_EVENT_UNION_SELECTION, GQL_CALENDAR_VALID_FIELDS, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field,
};
use crate::tools::helpers::range_to_gql;

/// Build the `calendar { ... }` selection set, expanding `event` with its
/// full union inline-fragment selection.
fn build_calendar_selection(fields: Option<&[String]>) -> String {
    let chosen: Vec<&str> = match fields {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_CALENDAR_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_CALENDAR_VALID_FIELDS.to_vec(),
    };
    if !chosen.contains(&"event") {
        let mut sel = String::from("{ ");
        for f in &chosen {
            sel.push_str(f);
            sel.push(' ');
        }
        sel.push('}');
        return sel;
    }
    let mut sel = String::from("{ ");
    for f in ["timestamp", "date", "symbol"] {
        if chosen.contains(&f) {
            sel.push_str(f);
            sel.push(' ');
        }
    }
    sel.push_str("event ");
    sel.push_str(CALENDAR_EVENT_UNION_SELECTION);
    sel.push_str(" }");
    sel
}

/// Aggregate upcoming financial events (earnings, dividends, options
/// expirations, and — when `FRED_API_KEY` is set — economic releases) across
/// the given symbols into a single time-sorted list.
pub async fn get_calendar(
    schema: &FinanceSchema,
    symbols: String,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_range = range_to_gql(range.as_deref().unwrap_or("1mo"));
    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();
    let syms_literal = gql_string_list_literal(&syms);
    let field_list = parse_fields(fields);
    let selection = build_calendar_selection(field_list.as_deref());

    let query =
        format!("query {{ calendar(symbols: [{syms_literal}], range: {gql_range}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "calendar");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
