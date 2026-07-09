use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_QUOTE_DEFAULT_FIELDS, GQL_QUOTE_VALID_FIELDS,
    GQL_RECOMMENDATION_VALID_FIELDS, GQL_SPLIT_DEFAULT_FIELDS, GQL_SPLIT_VALID_FIELDS,
    RECOMMENDATION_COMPOSITE_FIELDS, build_connection_selection, build_selection_or_default,
    build_type_spec_selection, escape_gql_string, execute_query, gql_string_list_literal,
    parse_fields, unwrap_field, unwrap_ticker_field, wrap_nested_connection,
};
use crate::tools::helpers::range_to_gql;

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat quote shape, multiple symbols return the batch `{quotes, errors}` shape.
pub async fn get_quote(
    schema: &FinanceSchema,
    symbols: String,
    lang: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms: Vec<String> = symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if syms.len() == 1 {
        get_one_quote(schema, syms.into_iter().next().unwrap(), lang, fields).await
    } else {
        get_many_quotes(schema, syms, lang, fields, limit, cursor).await
    }
}

async fn get_one_quote(
    schema: &FinanceSchema,
    symbol: String,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);

    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_QUOTE_VALID_FIELDS,
        GQL_QUOTE_DEFAULT_FIELDS,
    );

    let query = format!(
        "query GetQuote($symbol: String!, $logo: Boolean, $lang: String) {{ ticker(symbol: $symbol) {{ quote(logo: $logo, lang: $lang) {selection} }} }}"
    );

    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    variables.insert(async_graphql::Name::new("logo"), false.into());

    if let Some(l) = crate::lang::normalize(lang.as_deref()) {
        variables.insert(async_graphql::Name::new("lang"), l.into());
    }

    let json = execute_query(schema, &query, variables).await?;

    // Unwrap the GraphQL envelope: data.ticker.quote
    let quote = unwrap_ticker_field(json, "quote");

    let text = serde_json::to_string(&quote).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

async fn get_many_quotes(
    schema: &FinanceSchema,
    syms: Vec<String>,
    lang: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);

    let inner_selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_QUOTE_VALID_FIELDS,
        GQL_QUOTE_DEFAULT_FIELDS,
    );
    let selection = build_connection_selection(&inner_selection);

    let lang_arg = match crate::lang::normalize(lang.as_deref()) {
        Some(l) => format!("lang: \"{}\"", l),
        None => String::new(),
    };

    // Build the symbols array literal inline (GraphQL list arguments don't
    // support Variables well in all async-graphql versions).
    let syms_literal = gql_string_list_literal(&syms);

    let args = if lang_arg.is_empty() {
        format!("symbols: [{}]", syms_literal)
    } else {
        format!("symbols: [{}], {}", syms_literal, lang_arg)
    };

    let mut conn_args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor.as_deref() {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let conn_args = conn_args.join(", ");

    let query = format!(
        "query {{ quotes({}) {{ quotes({}) {} errors {{ symbol message }} }} }}",
        args, conn_args, selection
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;

    // Unwrap: data.quotes is { quotes: Connection<GqlQuote>, errors: [GqlBatchError] }
    let quotes = wrap_nested_connection(unwrap_field(json, "quotes"), "quotes");

    let text = serde_json::to_string(&quotes).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

pub async fn get_recommendations(
    schema: &FinanceSchema,
    symbol: String,
    limit: Option<u32>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_RECOMMENDATION_VALID_FIELDS,
        GQL_RECOMMENDATION_VALID_FIELDS,
        RECOMMENDATION_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetRecs($symbol: String!) {{ ticker(symbol: $symbol) {{ recommendations(limit: {}) {selection} }} }}",
        limit.unwrap_or(5)
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "recommendations");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

pub async fn get_splits(
    schema: &FinanceSchema,
    symbol: String,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_range = range_to_gql(range.as_deref().unwrap_or("max"));

    let field_list = parse_fields(fields);
    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_SPLIT_VALID_FIELDS,
        GQL_SPLIT_DEFAULT_FIELDS,
    );

    let query = format!(
        "query GetSplits($symbol: String!) {{ ticker(symbol: $symbol) {{ splits(range: {gql_range}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());

    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "splits");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
