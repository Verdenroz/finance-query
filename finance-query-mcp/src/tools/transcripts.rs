use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_TRANSCRIPT_DEFAULT_FIELDS, GQL_TRANSCRIPT_VALID_FIELDS, TRANSCRIPT_COMPOSITE_FIELDS,
    build_type_spec_selection, execute_query, parse_fields, unwrap_ticker_field,
};

pub async fn get_transcripts(
    schema: &FinanceSchema,
    symbol: String,
    limit: Option<u32>,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let limit_arg = limit.map(|l| format!("limit: {l}"));
    let lang_arg = crate::lang::normalize(lang.as_deref()).map(|l| format!("lang: \"{}\"", l));
    let args: Vec<String> = [limit_arg, lang_arg].into_iter().flatten().collect();
    let args_str = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_TRANSCRIPT_VALID_FIELDS,
        GQL_TRANSCRIPT_DEFAULT_FIELDS,
        TRANSCRIPT_COMPOSITE_FIELDS,
    );

    let query = format!(
        "query GetTranscripts($symbol: String!) {{ ticker(symbol: $symbol) {{ transcripts{args_str} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "transcripts");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
