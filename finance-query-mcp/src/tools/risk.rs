use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_RISK_DEFAULT_FIELDS, GQL_RISK_VALID_FIELDS, build_selection_or_default, execute_query,
    parse_fields, unwrap_ticker_field,
};
use crate::tools::helpers::{interval_to_gql, range_to_gql};

pub async fn get_risk(
    schema: &FinanceSchema,
    symbol: String,
    interval: Option<String>,
    range: Option<String>,
    benchmark: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.as_deref().unwrap_or("1d"));
    let gql_range = range_to_gql(range.as_deref().unwrap_or("1y"));
    let has_benchmark = benchmark.as_deref().is_some_and(|b| !b.is_empty());
    let bench_arg = if has_benchmark {
        ", benchmark: $benchmark"
    } else {
        ""
    };
    // GraphQL rejects a declared operation variable that's never referenced
    // in the query body, so $benchmark can only be declared when it's used.
    let benchmark_decl = if has_benchmark {
        ", $benchmark: String"
    } else {
        ""
    };

    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_RISK_VALID_FIELDS,
        GQL_RISK_DEFAULT_FIELDS,
    );

    let query = format!(
        "query GetRisk($symbol: String!{benchmark_decl}) {{ ticker(symbol: $symbol) {{ risk(interval: {gql_interval}, range: {gql_range}{bench_arg}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    if let Some(b) = benchmark.filter(|b| !b.is_empty()) {
        variables.insert(async_graphql::Name::new("benchmark"), b.into());
    }

    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "risk");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
