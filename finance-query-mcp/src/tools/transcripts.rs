use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_TRANSCRIPT_DEFAULT_FIELDS, GQL_TRANSCRIPT_VALID_FIELDS,
    escape_gql_string, execute_query, parse_fields, unwrap_ticker_field, wrap_nested_connection,
};

/// Same `transcript` sub-selection as `TRANSCRIPT_COMPOSITE_FIELDS` in the
/// shared `fields.rs`, but requesting paginated `paragraphs` instead of the
/// unbounded whole-call `text` blob — a full transcript can run tens of
/// thousands of tokens, far past MCP's response-size budget.
fn transcript_selection(paragraph_limit: u32, paragraph_cursor: Option<&str>) -> String {
    let mut args = vec![format!("first: {paragraph_limit}")];
    if let Some(c) = paragraph_cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    format!(
        "{{ transcriptContent {{ companyId eventId version speakerMapping {{ speaker speakerData {{ company name role }} }} transcript {{ numberOfSpeakers paragraphs({}) {{ edges {{ node {{ speaker start end text }} }} pageInfo {{ hasNextPage hasPreviousPage startCursor endCursor }} }} }} }} transcriptMetadata {{ date eventId eventType fiscalPeriod fiscalYear isLatest s3Url title transcriptId transcriptType updated }} }}",
        args.join(", ")
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn get_transcripts(
    schema: &FinanceSchema,
    symbol: String,
    limit: Option<u32>,
    lang: Option<String>,
    fields: Option<String>,
    paragraph_limit: Option<u32>,
    paragraph_cursor: Option<String>,
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
    let mut chosen: Vec<&str> = match field_list.as_deref() {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_TRANSCRIPT_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_TRANSCRIPT_DEFAULT_FIELDS.to_vec(),
    };
    if chosen.is_empty() {
        chosen = GQL_TRANSCRIPT_DEFAULT_FIELDS.to_vec();
    }
    let transcript_nested = transcript_selection(
        paragraph_limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        paragraph_cursor.as_deref(),
    );
    let mut selection = String::from("{ ");
    for f in chosen {
        selection.push_str(f);
        if f == "transcript" {
            selection.push(' ');
            selection.push_str(&transcript_nested);
        }
        selection.push(' ');
    }
    selection.push('}');

    let query = format!(
        "query GetTranscripts($symbol: String!) {{ ticker(symbol: $symbol) {{ transcripts{args_str} {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let mut data = unwrap_ticker_field(json, "transcripts");
    if let Some(list) = data.as_array_mut() {
        for item in list.iter_mut() {
            if let Some(inner) = item
                .get_mut("transcript")
                .and_then(|t| t.get_mut("transcriptContent"))
                .and_then(|c| c.get_mut("transcript"))
            {
                *inner = wrap_nested_connection(inner.take(), "paragraphs");
            }
        }
    }
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
