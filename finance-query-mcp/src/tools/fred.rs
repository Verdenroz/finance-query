use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::{invalid_params, ser_err};
use crate::tools::gql::{
    GQL_MACRO_SERIES_VALID_FIELDS, GQL_TREASURY_YIELD_VALID_FIELDS, MACRO_SERIES_COMPOSITE_FIELDS,
    build_selection_or_default, build_type_spec_selection, execute_query, parse_fields,
    unwrap_field,
};

pub async fn get_fred_series(
    schema: &FinanceSchema,
    id: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    if std::env::var("FRED_API_KEY").is_err() {
        return Err(invalid_params(
            "FRED not configured — set the FRED_API_KEY environment variable to enable FRED tools",
        ));
    }
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_MACRO_SERIES_VALID_FIELDS,
        GQL_MACRO_SERIES_VALID_FIELDS,
        MACRO_SERIES_COMPOSITE_FIELDS,
    );
    let query = format!("query GetFredSeries($id: String!) {{ fredSeries(id: $id) {selection} }}");
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("id"), id.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_field(json, "fredSeries");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_treasury_yields(
    schema: &FinanceSchema,
    year: Option<u32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_TREASURY_YIELD_VALID_FIELDS,
        GQL_TREASURY_YIELD_VALID_FIELDS,
    );
    let year_arg = year.map(|y| format!("(year: {y})")).unwrap_or_default();
    let query = format!("query {{ treasuryYields{year_arg} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "treasuryYields");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
