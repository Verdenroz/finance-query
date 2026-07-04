use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, DIVIDENDS_COMPOSITE_FIELDS, GQL_DIVIDENDS_DEFAULT_FIELDS,
    GQL_DIVIDENDS_VALID_FIELDS, build_paginated_composite_selection, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field, unwrap_ticker_field,
    wrap_nested_connection,
};

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat shape with dividend `analytics` (CAGR, average payment, etc.) and
/// paginated dividend history; multiple symbols return the batch
/// `{dividends, errors}` shape (dividend entries only, no per-symbol analytics
/// — the underlying batch query doesn't compute it).
pub async fn get_dividends(
    schema: &FinanceSchema,
    symbols: String,
    range: Option<String>,
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
        get_one_dividends(
            schema,
            syms.into_iter().next().unwrap(),
            range,
            fields,
            limit,
            cursor,
        )
        .await
    } else {
        get_many_dividends(schema, syms, range, fields).await
    }
}

async fn get_one_dividends(
    schema: &FinanceSchema,
    symbol: String,
    range: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let dividends_item_selection = DIVIDENDS_COMPOSITE_FIELDS
        .iter()
        .find(|(name, _)| *name == "dividends")
        .map(|(_, sel)| *sel)
        .unwrap_or("{ timestamp amount }");
    let fields_csv = field_list.as_ref().map(|fs| fs.join(","));
    let selection = build_paginated_composite_selection(
        fields_csv.as_deref(),
        GQL_DIVIDENDS_VALID_FIELDS,
        GQL_DIVIDENDS_DEFAULT_FIELDS,
        DIVIDENDS_COMPOSITE_FIELDS,
        "dividends",
        dividends_item_selection,
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );
    let r = range.as_deref().unwrap_or("max").to_lowercase();
    let gql_range = match crate::tools::helpers::parse_range(r.as_str()) {
        finance_query::TimeRange::OneDay => "ONE_DAY",
        finance_query::TimeRange::FiveDays => "FIVE_DAYS",
        finance_query::TimeRange::OneMonth => "ONE_MONTH",
        finance_query::TimeRange::ThreeMonths => "THREE_MONTHS",
        finance_query::TimeRange::SixMonths => "SIX_MONTHS",
        finance_query::TimeRange::OneYear => "ONE_YEAR",
        finance_query::TimeRange::TwoYears => "TWO_YEARS",
        finance_query::TimeRange::FiveYears => "FIVE_YEARS",
        finance_query::TimeRange::TenYears => "TEN_YEARS",
        finance_query::TimeRange::YearToDate => "YEAR_TO_DATE",
        finance_query::TimeRange::Max => "MAX",
    };

    let query = format!(
        "query GetDivs($symbol: String!) {{ ticker(symbol: $symbol) {{ dividends(range: {gql_range}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());

    let json = execute_query(schema, &query, variables).await?;
    let data = wrap_nested_connection(unwrap_ticker_field(json, "dividends"), "dividends");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

async fn get_many_dividends(
    schema: &FinanceSchema,
    syms: Vec<String>,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let r = range.as_deref().unwrap_or("1y").to_lowercase();
    let gql_range = match crate::tools::helpers::parse_range(r.as_str()) {
        finance_query::TimeRange::OneDay => "ONE_DAY",
        finance_query::TimeRange::FiveDays => "FIVE_DAYS",
        finance_query::TimeRange::OneMonth => "ONE_MONTH",
        finance_query::TimeRange::ThreeMonths => "THREE_MONTHS",
        finance_query::TimeRange::SixMonths => "SIX_MONTHS",
        finance_query::TimeRange::OneYear => "ONE_YEAR",
        finance_query::TimeRange::TwoYears => "TWO_YEARS",
        finance_query::TimeRange::FiveYears => "FIVE_YEARS",
        finance_query::TimeRange::TenYears => "TEN_YEARS",
        finance_query::TimeRange::YearToDate => "YEAR_TO_DATE",
        finance_query::TimeRange::Max => "MAX",
    };

    let field_list = parse_fields(fields);
    // "dividends" here is Vec<GqlDividend> (no per-symbol analytics, unlike
    // single-symbol GqlDividends) — needs its own nested sub-selection.
    let want_dividends = field_list
        .as_ref()
        .map(|fs| fs.iter().any(|f| f == "dividends"))
        .unwrap_or(true);
    let mut selection = String::from("{ symbol ");
    if want_dividends {
        selection.push_str("dividends { timestamp amount } ");
    }
    selection.push('}');

    let syms_literal = gql_string_list_literal(&syms);

    let query = format!(
        "query {{ dividendsBatch(symbols: [{}], range: {gql_range}) {{ dividends {} errors {{ symbol message }} }} }}",
        syms_literal, selection
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "dividendsBatch");
    let text = serde_json::to_string(&data).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}
