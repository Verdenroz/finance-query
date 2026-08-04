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

/// Build the `transcripts(...)` field argument list: optional `limit` and
/// `lang`, omitted entirely (no parens) when both are absent.
fn build_transcripts_args(limit: Option<u32>, lang: Option<&str>) -> String {
    let limit_arg = limit.map(|l| format!("limit: {l}"));
    let lang_arg = crate::lang::normalize(lang).map(|l| format!("lang: \"{}\"", l));
    let args: Vec<String> = [limit_arg, lang_arg].into_iter().flatten().collect();
    if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    }
}

/// Choose the requested top-level transcript fields, validating against
/// `GQL_TRANSCRIPT_VALID_FIELDS` and falling back to the curated defaults
/// when the caller's list is absent, empty, or matches nothing valid.
fn choose_transcript_fields(field_list: Option<&[String]>) -> Vec<&str> {
    let mut chosen: Vec<&str> = match field_list {
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
    chosen
}

/// Build the top-level transcript selection set, splicing the paginated
/// `transcript_nested` sub-selection in wherever `transcript` was chosen.
fn build_transcript_field_selection(chosen: &[&str], transcript_nested: &str) -> String {
    let mut selection = String::from("{ ");
    for f in chosen {
        selection.push_str(f);
        if *f == "transcript" {
            selection.push(' ');
            selection.push_str(transcript_nested);
        }
        selection.push(' ');
    }
    selection.push('}');
    selection
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
    let args_str = build_transcripts_args(limit, lang.as_deref());
    let field_list = parse_fields(fields);
    let chosen = choose_transcript_fields(field_list.as_deref());
    let transcript_nested = transcript_selection(
        paragraph_limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE),
        paragraph_cursor.as_deref(),
    );
    let selection = build_transcript_field_selection(&chosen, &transcript_nested);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn transcript_selection_includes_first_and_no_cursor_by_default() {
        let sel = transcript_selection(25, None);
        assert!(sel.contains("paragraphs(first: 25)"));
        assert!(!sel.contains("after:"));
    }

    #[test]
    fn transcript_selection_includes_escaped_cursor_when_present() {
        let sel = transcript_selection(10, Some("has\"quote"));
        assert!(sel.contains("paragraphs(first: 10, after: \"has\\\"quote\")"));
    }

    #[test]
    fn transcript_selection_contains_expected_structure() {
        let sel = transcript_selection(25, None);
        assert!(sel.contains("transcriptContent"));
        assert!(sel.contains("speakerMapping"));
        assert!(sel.contains("transcriptMetadata"));
    }

    #[test]
    fn build_transcripts_args_is_empty_when_both_absent() {
        assert_eq!(build_transcripts_args(None, None), "");
    }

    #[test]
    fn build_transcripts_args_includes_limit_only() {
        assert_eq!(build_transcripts_args(Some(4), None), "(limit: 4)");
    }

    #[test]
    fn build_transcripts_args_includes_lang_only_for_supported_language() {
        assert_eq!(build_transcripts_args(None, Some("ja")), "(lang: \"ja\")");
    }

    #[test]
    fn build_transcripts_args_omits_english_lang() {
        assert_eq!(build_transcripts_args(Some(2), Some("en")), "(limit: 2)");
    }

    #[test]
    fn build_transcripts_args_combines_limit_and_lang() {
        assert_eq!(
            build_transcripts_args(Some(2), Some("ja")),
            "(limit: 2, lang: \"ja\")"
        );
    }

    #[test]
    fn choose_transcript_fields_falls_back_to_defaults_when_none() {
        assert_eq!(
            choose_transcript_fields(None),
            GQL_TRANSCRIPT_DEFAULT_FIELDS.to_vec()
        );
    }

    #[test]
    fn choose_transcript_fields_falls_back_to_defaults_when_empty() {
        let empty = fields(&[]);
        assert_eq!(
            choose_transcript_fields(Some(&empty)),
            GQL_TRANSCRIPT_DEFAULT_FIELDS.to_vec()
        );
    }

    #[test]
    fn choose_transcript_fields_falls_back_when_every_requested_field_is_unknown() {
        let requested = fields(&["bogus1", "bogus2"]);
        assert_eq!(
            choose_transcript_fields(Some(&requested)),
            GQL_TRANSCRIPT_DEFAULT_FIELDS.to_vec()
        );
    }

    #[test]
    fn choose_transcript_fields_keeps_only_valid_requested_fields() {
        let requested = fields(&["title", "bogus", "quarter"]);
        assert_eq!(
            choose_transcript_fields(Some(&requested)),
            vec!["title", "quarter"]
        );
    }

    #[test]
    fn build_transcript_field_selection_wraps_fields_in_braces() {
        let sel = build_transcript_field_selection(&["title", "quarter"], "{ nested }");
        assert_eq!(sel, "{ title quarter }");
    }

    #[test]
    fn build_transcript_field_selection_splices_nested_selection_after_transcript() {
        let sel = build_transcript_field_selection(&["title", "transcript"], "{ nested }");
        assert_eq!(sel, "{ title transcript { nested } }");
    }
}
