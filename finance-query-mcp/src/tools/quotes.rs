use finance_query::TimeRange;
use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    GQL_QUOTE_DEFAULT_FIELDS, GQL_QUOTE_VALID_FIELDS, GQL_RECOMMENDATION_VALID_FIELDS,
    GQL_SPLIT_DEFAULT_FIELDS, GQL_SPLIT_VALID_FIELDS, RECOMMENDATION_COMPOSITE_FIELDS,
    build_selection_or_default, build_type_spec_selection, execute_query, gql_string_list_literal,
    parse_fields, unwrap_field, unwrap_ticker_field,
};
use crate::tools::helpers::parse_range;

pub async fn get_quote(
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

pub async fn get_quotes(
    schema: &FinanceSchema,
    symbols: String,
    lang: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();

    let field_list = parse_fields(fields);

    let selection = build_selection_or_default(
        field_list.as_deref(),
        GQL_QUOTE_VALID_FIELDS,
        GQL_QUOTE_DEFAULT_FIELDS,
    );

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

    let query = format!(
        "query {{ quotes({}) {{ quotes {} errors {{ symbol message }} }} }}",
        args, selection
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;

    // Unwrap: data.quotes is { quotes: [GqlQuote], errors: [GqlBatchError] }
    let quotes = unwrap_field(json, "quotes");

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
    let r = parse_range(range.as_deref().unwrap_or("max"));
    let gql_range = match r {
        TimeRange::OneDay => "ONE_DAY",
        TimeRange::FiveDays => "FIVE_DAYS",
        TimeRange::OneMonth => "ONE_MONTH",
        TimeRange::ThreeMonths => "THREE_MONTHS",
        TimeRange::SixMonths => "SIX_MONTHS",
        TimeRange::OneYear => "ONE_YEAR",
        TimeRange::TwoYears => "TWO_YEARS",
        TimeRange::FiveYears => "FIVE_YEARS",
        TimeRange::TenYears => "TEN_YEARS",
        TimeRange::YearToDate => "YEAR_TO_DATE",
        TimeRange::Max => "MAX",
    };

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
