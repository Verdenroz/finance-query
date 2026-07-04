use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_OPTIONS_VALID_FIELDS, OPTIONS_COMPOSITE_FIELDS,
    build_connection_selection, escape_gql_string, execute_query, parse_fields,
    unwrap_ticker_field, wrap_nested_connection,
};

/// Build the options `{ ... }` selection, expanding `calls`/`puts` as paginated
/// Connections sharing the same `first`/`after` args — mirrors
/// `build_options_selection` in `server/src/handlers/options.rs`.
fn build_options_selection(fields: Option<&[String]>, limit: u32, cursor: Option<&str>) -> String {
    let chosen: Vec<&str> = match fields {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_OPTIONS_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_OPTIONS_VALID_FIELDS.to_vec(),
    };
    let mut args = vec![format!("first: {limit}")];
    if let Some(cursor) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
    }
    let args_str = format!("({})", args.join(", "));
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

pub async fn get_options(
    schema: &FinanceSchema,
    symbol: String,
    expiration: Option<i64>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    // Parens must be omitted entirely when there's no argument — `options()`
    // with empty parens is invalid GraphQL syntax, not "no arguments".
    let date_arg = match expiration {
        Some(ts) => format!("(date: {ts})"),
        None => String::new(),
    };
    let field_list = parse_fields(fields);
    let selection = build_options_selection(
        field_list.as_deref(),
        limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        cursor.as_deref(),
    );

    let query = format!(
        "query GetOpts($symbol: String!) {{ ticker(symbol: $symbol) {{ options{} {} }} }}",
        date_arg, selection
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let mut data = unwrap_ticker_field(json, "options");
    data = wrap_nested_connection(data, "calls");
    data = wrap_nested_connection(data, "puts");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
