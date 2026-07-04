use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_OPTIONS_DEFAULT_FIELDS, GQL_OPTIONS_VALID_FIELDS, OPTIONS_COMPOSITE_FIELDS,
    build_type_spec_selection, execute_query, parse_fields, unwrap_ticker_field,
};

pub async fn get_options(
    schema: &FinanceSchema,
    symbol: String,
    expiration: Option<i64>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    // Parens must be omitted entirely when there's no argument — `options()`
    // with empty parens is invalid GraphQL syntax, not "no arguments".
    let date_arg = match expiration {
        Some(ts) => format!("(date: {ts})"),
        None => String::new(),
    };
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_OPTIONS_VALID_FIELDS,
        GQL_OPTIONS_DEFAULT_FIELDS,
        OPTIONS_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query GetOpts($symbol: String!) {{ ticker(symbol: $symbol) {{ options{} {} }} }}",
        date_arg, selection
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "options");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
