use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_MACRO_SERIES_VALID_FIELDS, GQL_TREASURY_YIELD_VALID_FIELDS,
    MACRO_SERIES_COMPOSITE_FIELDS, build_connection_selection, build_paginated_composite_selection,
    build_selection_or_default, escape_gql_string, execute_query, parse_fields, unwrap_field,
    wrap_connection, wrap_nested_connection,
};

pub async fn get_fred_series(
    schema: &FinanceSchema,
    id: String,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    if std::env::var("FRED_API_KEY").is_err() {
        return Err(invalid_params(
            "FRED not configured — set the FRED_API_KEY environment variable to enable FRED tools",
        ));
    }
    let field_list = parse_fields(fields);
    let observations_item_selection = MACRO_SERIES_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "observations")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ date value }");
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_MACRO_SERIES_VALID_FIELDS,
        GQL_MACRO_SERIES_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS,
        "observations",
        observations_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );
    let query = format!("query GetFredSeries($id: String!) {{ fredSeries(id: $id) {selection} }}");
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("id"), id.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_nested_connection(unwrap_field(json, "fredSeries"), "observations");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_treasury_yields(
    schema: &FinanceSchema,
    year: Option<u32>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TREASURY_YIELD_VALID_FIELDS,
        GQL_TREASURY_YIELD_VALID_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);
    let mut args = Vec::new();
    if let Some(y) = year {
        args.push(format!("year: {y}"));
    }
    args.push(format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)));
    if let Some(c) = cursor.as_deref() {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let query = format!(
        "query {{ treasuryYields({}) {selection} }}",
        args.join(", ")
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_connection(unwrap_field(json, "treasuryYields"));
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
