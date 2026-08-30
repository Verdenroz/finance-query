//! TVL, symbol detail, index constituent changes and sector history.

use finance_query_server::graphql::FinanceSchema;
use finance_query_server::graphql::fields::{
    GQL_INDEX_CONSTITUENT_CHANGE_VALID_FIELDS, GQL_PROTOCOL_TVL_VALID_FIELDS,
    GQL_SECTOR_PERFORMANCE_HISTORY_VALID_FIELDS, GQL_SYMBOL_DETAILS_VALID_FIELDS,
    GQL_TVL_POINT_VALID_FIELDS, PROTOCOL_TVL_COMPOSITE_FIELDS,
    SECTOR_PERFORMANCE_HISTORY_COMPOSITE_FIELDS, escape_gql_string, unwrap_field,
};
use finance_query_server::graphql::pagination::build_connection_selection;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, build_selection_or_default, execute_query, parse_fields, wrap_connection,
};

/// Flat-field selection with composite sub-selections spliced in. The shared
/// builder in `gql` only covers the paginated case.
fn composite_selection(
    fields: Option<&[String]>,
    valid: &[&str],
    composites: &[(&str, &str)],
) -> String {
    let mut chosen: Vec<&str> = match fields {
        Some(f) if !f.is_empty() => f
            .iter()
            .map(String::as_str)
            .filter(|name| valid.contains(name))
            .collect(),
        _ => valid.to_vec(),
    };
    if chosen.is_empty() {
        chosen = valid.to_vec();
    }
    let mut out = String::from("{ ");
    for name in chosen {
        out.push_str(name);
        if let Some((_, sub)) = composites.iter().find(|(n, _)| *n == name) {
            out.push(' ');
            out.push_str(sub);
        }
        out.push(' ');
    }
    out.push('}');
    out
}

fn text(data: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_protocol_tvl(
    schema: &FinanceSchema,
    id: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = composite_selection(
        field_list.as_deref(),
        GQL_PROTOCOL_TVL_VALID_FIELDS,
        PROTOCOL_TVL_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetTvl {{ cryptoCoin(id: \"{}\") {{ tvl {selection} }} }}",
        escape_gql_string(&id)
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    text(unwrap_field(unwrap_field(json, "cryptoCoin"), "tvl"))
}

pub async fn get_protocol_tvl_history(
    schema: &FinanceSchema,
    id: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner = build_selection_or_default(
        field_list.as_deref(),
        GQL_TVL_POINT_VALID_FIELDS,
        GQL_TVL_POINT_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner);
    let first = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE);
    let after = cursor
        .as_deref()
        .map(|c| format!(", after: \"{}\"", escape_gql_string(c)))
        .unwrap_or_default();
    let query = format!(
        "query GetTvlHistory {{ cryptoCoin(id: \"{}\") {{ tvlHistory(first: {first}{after}) {selection} }} }}",
        escape_gql_string(&id)
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    text(wrap_connection(unwrap_field(
        unwrap_field(json, "cryptoCoin"),
        "tvlHistory",
    )))
}

pub async fn get_symbol_details(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_SYMBOL_DETAILS_VALID_FIELDS,
        GQL_SYMBOL_DETAILS_VALID_FIELDS,
    );
    let query = format!(
        "query GetSymbolDetails($symbol: String!) {{ symbolDetails(symbol: $symbol) {selection} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    text(unwrap_field(json, "symbolDetails"))
}

pub async fn get_index_constituent_changes(
    schema: &FinanceSchema,
    symbol: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner = build_selection_or_default(
        field_list.as_deref(),
        GQL_INDEX_CONSTITUENT_CHANGE_VALID_FIELDS,
        GQL_INDEX_CONSTITUENT_CHANGE_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner);
    let first = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE);
    let after = cursor
        .as_deref()
        .map(|c| format!(", after: \"{}\"", escape_gql_string(c)))
        .unwrap_or_default();
    let query = format!(
        "query GetChanges($symbol: String!) {{ indexConstituentChanges(symbol: $symbol, first: {first}{after}) {selection} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    text(wrap_connection(unwrap_field(
        json,
        "indexConstituentChanges",
    )))
}

pub async fn get_sector_performance_history(
    schema: &FinanceSchema,
    limit: Option<u32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = composite_selection(
        field_list.as_deref(),
        GQL_SECTOR_PERFORMANCE_HISTORY_VALID_FIELDS,
        SECTOR_PERFORMANCE_HISTORY_COMPOSITE_FIELDS,
    );
    let limit = limit.unwrap_or(30);
    let query = format!(
        "query GetSectorHistory {{ sectorPerformanceHistory(limit: {limit}) {selection} }}"
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    text(unwrap_field(json, "sectorPerformanceHistory"))
}
