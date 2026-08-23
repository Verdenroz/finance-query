use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_NEWS_DEFAULT_FIELDS, GQL_NEWS_VALID_FIELDS,
    GQL_PRESS_RELEASE_DEFAULT_FIELDS, GQL_PRESS_RELEASE_VALID_FIELDS, NEWS_COMPOSITE_FIELDS,
    build_connection_selection, build_selection_or_default, build_type_spec_selection,
    escape_gql_string, execute_query, parse_fields, unwrap_field, unwrap_ticker_field,
    wrap_connection,
};

/// Build the `, lang: "xx"` argument fragment, or an empty string when the
/// language normalizes away (unset, English, or unrecognized).
fn build_lang_arg(lang: Option<&str>) -> String {
    match crate::lang::normalize(lang) {
        Some(l) => format!(", lang: \"{}\"", l),
        None => String::new(),
    }
}

/// Build the `news(...)` connection args: the fixed upstream-fetch `count`,
/// the page-size `first`, and an optional `after` cursor.
fn build_conn_args(limit: Option<u32>, cursor: Option<&str>) -> String {
    let mut args = vec![
        "count: 10".to_string(),
        format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
    ];
    if let Some(c) = cursor {
        args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    args.join(", ")
}

pub async fn get_news(
    schema: &FinanceSchema,
    symbol: Option<String>,
    lang: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);

    let inner_selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_NEWS_VALID_FIELDS,
        GQL_NEWS_DEFAULT_FIELDS,
        NEWS_COMPOSITE_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);

    let lang_arg = build_lang_arg(lang.as_deref());
    // `count` bounds the overall upstream fetch (kept at its historical value
    // of 10, unchanged); `first`/`after` paginate a page out of that fetched
    // pool — the two are independent GraphQL args on the same field.
    let conn_args = build_conn_args(limit, cursor.as_deref());

    // Per-symbol and general news hit different root fields, so unwrap
    // inside each branch rather than probing both shapes afterward.
    let result = if let Some(sym) = symbol {
        let query = format!(
            "query GetNews($symbol: String!) {{ ticker(symbol: $symbol) {{ news({conn_args}{lang_arg}) {selection} }} }}"
        );
        let mut variables = async_graphql::Variables::default();
        variables.insert(async_graphql::Name::new("symbol"), sym.into());
        let json = execute_query(schema, &query, variables).await?;
        unwrap_ticker_field(json, "news")
    } else {
        let query = format!("query {{ news({conn_args}{lang_arg}) {selection} }}");
        let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
        unwrap_field(json, "news")
    };
    let result = wrap_connection(result);

    let text = serde_json::to_string(&result).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

pub async fn get_press_releases(
    schema: &FinanceSchema,
    symbol: String,
    limit: Option<u32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_PRESS_RELEASE_VALID_FIELDS,
        GQL_PRESS_RELEASE_DEFAULT_FIELDS,
    );

    let query = format!(
        "query GetPressReleases($symbol: String!) {{ ticker(symbol: $symbol) {{ pressReleases(limit: {}) {selection} }} }}",
        limit.unwrap_or(10)
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "pressReleases");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_lang_arg_is_empty_when_lang_is_none() {
        assert_eq!(build_lang_arg(None), "");
    }

    #[test]
    fn build_lang_arg_is_empty_for_english() {
        assert_eq!(build_lang_arg(Some("en")), "");
    }

    #[test]
    fn build_lang_arg_is_empty_for_unparseable_input() {
        assert_eq!(build_lang_arg(Some("1234")), "");
        assert_eq!(build_lang_arg(Some("")), "");
    }

    #[test]
    fn build_lang_arg_includes_canonical_code_for_supported_language() {
        assert_eq!(build_lang_arg(Some("ja")), ", lang: \"ja\"");
    }

    #[test]
    fn build_conn_args_has_fixed_count_and_default_limit_without_cursor() {
        let args = build_conn_args(None, None);
        assert_eq!(args, format!("count: 10, first: {DEFAULT_MCP_PAGE_SIZE}"));
    }

    #[test]
    fn build_conn_args_uses_given_limit() {
        let args = build_conn_args(Some(3), None);
        assert_eq!(args, "count: 10, first: 3");
    }

    #[test]
    fn build_conn_args_includes_cursor_when_present() {
        let args = build_conn_args(Some(3), Some("cur1"));
        assert_eq!(args, "count: 10, first: 3, after: \"cur1\"");
    }

    #[test]
    fn build_conn_args_escapes_special_characters_in_cursor() {
        let args = build_conn_args(None, Some("has\"quote"));
        assert!(args.contains("after: \"has\\\"quote\""));
    }
}
