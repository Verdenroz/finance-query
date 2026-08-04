use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_FEEDS_DEFAULT_FIELDS, GQL_FEEDS_VALID_FIELDS,
    build_connection_selection, build_selection_or_default, escape_gql_string, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field, wrap_connection,
};

/// Parse the comma-separated `sources` param, falling back to this tool's
/// historical default list when the caller omits it or supplies only blanks.
fn parse_sources(sources: Option<&str>) -> Vec<&str> {
    let list: Vec<&str> = match sources {
        Some(raw) => raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect(),
        None => vec![],
    };
    if list.is_empty() {
        vec!["marketwatch", "bloomberg", "wsj", "fortune"]
    } else {
        list
    }
}

/// Build the `feeds(...)` field argument list: sources, `first`, and an
/// optional `after` cursor.
fn build_feeds_args(sources: &[&str], limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = vec![
        format!("sources: [{}]", gql_string_list_literal(sources)),
        format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
    ];
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    format!("({})", args.join(", "))
}

pub async fn get_feeds(
    schema: &FinanceSchema,
    sources: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_FEEDS_VALID_FIELDS,
        GQL_FEEDS_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);

    // Preserve this tool's historical default sources (distinct from the
    // GraphQL field's own default) by always passing an explicit list.
    let list = parse_sources(sources.as_deref());
    let args_str = build_feeds_args(&list, limit, cursor.as_deref());

    let query = format!("query {{ feeds{args_str} {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let result = wrap_connection(unwrap_field(json, "feeds"));

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sources_falls_back_to_defaults_when_none() {
        assert_eq!(
            parse_sources(None),
            vec!["marketwatch", "bloomberg", "wsj", "fortune"]
        );
    }

    #[test]
    fn parse_sources_falls_back_to_defaults_when_empty_string() {
        assert_eq!(
            parse_sources(Some("")),
            vec!["marketwatch", "bloomberg", "wsj", "fortune"]
        );
    }

    #[test]
    fn parse_sources_falls_back_to_defaults_when_only_blanks_and_commas() {
        assert_eq!(
            parse_sources(Some(" , , ")),
            vec!["marketwatch", "bloomberg", "wsj", "fortune"]
        );
    }

    #[test]
    fn parse_sources_splits_and_trims_a_csv_list() {
        assert_eq!(
            parse_sources(Some("wsj, bloomberg ")),
            vec!["wsj", "bloomberg"]
        );
    }

    #[test]
    fn parse_sources_preserves_single_source() {
        assert_eq!(parse_sources(Some("wsj")), vec!["wsj"]);
    }

    #[test]
    fn build_feeds_args_uses_default_limit_without_cursor() {
        let args = build_feeds_args(&["wsj"], None, None);
        assert_eq!(
            args,
            format!("(sources: [\"wsj\"], first: {DEFAULT_MCP_PAGE_SIZE})")
        );
    }

    #[test]
    fn build_feeds_args_includes_given_limit_and_cursor() {
        let args = build_feeds_args(&["wsj", "fortune"], Some(5), Some("cur1"));
        assert_eq!(
            args,
            "(sources: [\"wsj\", \"fortune\"], first: 5, after: \"cur1\")"
        );
    }

    #[test]
    fn build_feeds_args_escapes_special_characters_in_cursor() {
        let args = build_feeds_args(&["wsj"], Some(1), Some("has\"quote"));
        assert!(args.contains("after: \"has\\\"quote\""));
    }
}
