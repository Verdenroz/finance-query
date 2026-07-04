use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_INDICATORS_DEFAULT_FIELDS, GQL_INDICATORS_VALID_FIELDS,
    INDICATOR_COMPOSITE_FIELDS, build_connection_selection, build_type_spec_selection,
    escape_gql_string, execute_query, gql_string_list_literal, parse_fields, unwrap_field,
    unwrap_ticker_field, wrap_nested_connection,
};
use crate::tools::helpers::{parse_interval, parse_range};

fn interval_to_gql(s: &str) -> &'static str {
    match parse_interval(s) {
        finance_query::Interval::OneMinute => "ONE_MINUTE",
        finance_query::Interval::FiveMinutes => "FIVE_MINUTES",
        finance_query::Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        finance_query::Interval::ThirtyMinutes => "THIRTY_MINUTES",
        finance_query::Interval::OneHour => "ONE_HOUR",
        finance_query::Interval::OneDay => "ONE_DAY",
        finance_query::Interval::OneWeek => "ONE_WEEK",
        finance_query::Interval::OneMonth => "ONE_MONTH",
        finance_query::Interval::ThreeMonths => "THREE_MONTHS",
    }
}

fn range_to_gql(s: &str) -> &'static str {
    match parse_range(s) {
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
    }
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat indicators shape, multiple symbols return the batch `{indicators, errors}` shape.
pub async fn get_indicators(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<String>,
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
        get_one_indicators(
            schema,
            syms.into_iter().next().unwrap(),
            interval,
            range,
            fields,
        )
        .await
    } else {
        get_many_indicators(schema, syms, interval, range, fields, limit, cursor).await
    }
}

async fn get_one_indicators(
    schema: &FinanceSchema,
    symbol: String,
    interval: Option<String>,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.as_deref().unwrap_or("1d"));
    let gql_range = range_to_gql(range.as_deref().unwrap_or("1y"));
    let field_list = parse_fields(fields);
    let selection = build_type_spec_selection(
        field_list.as_deref(),
        GQL_INDICATORS_VALID_FIELDS,
        GQL_INDICATORS_DEFAULT_FIELDS,
        INDICATOR_COMPOSITE_FIELDS,
    );
    let query = format!(
        "query GetIndicators($symbol: String!) {{ ticker(symbol: $symbol) {{ indicators(interval: {gql_interval}, range: {gql_range}) {selection} }} }}"
    );
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());
    let json = execute_query(schema, &query, variables).await?;
    let data = unwrap_ticker_field(json, "indicators");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

async fn get_many_indicators(
    schema: &FinanceSchema,
    syms: Vec<String>,
    interval: Option<String>,
    range: Option<String>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.as_deref().unwrap_or("1d"));
    let gql_range = range_to_gql(range.as_deref().unwrap_or("1y"));
    let syms_literal = gql_string_list_literal(&syms);
    let field_list = parse_fields(fields);
    // "indicators" (`GqlIndicatorsSummary`) is composite and needs its own
    // nested sub-selection, not a bare field name.
    let want_indicators = field_list
        .as_ref()
        .map(|fs| fs.iter().any(|f| f == "indicators"))
        .unwrap_or(true);
    let mut item_selection = String::from("{ symbol ");
    if want_indicators {
        item_selection.push_str("indicators ");
        item_selection.push_str(&build_type_spec_selection(
            field_list.as_deref(),
            GQL_INDICATORS_VALID_FIELDS,
            GQL_INDICATORS_DEFAULT_FIELDS,
            INDICATOR_COMPOSITE_FIELDS,
        ));
        item_selection.push(' ');
    }
    item_selection.push('}');
    let selection = build_connection_selection(&item_selection);

    let mut conn_args = vec![format!("first: {}", limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE))];
    if let Some(c) = cursor.as_deref() {
        conn_args.push(format!("after: \"{}\"", escape_gql_string(c)));
    }
    let conn_args = conn_args.join(", ");

    let query = format!(
        "query {{ indicatorsBatch(symbols: [{}], interval: {gql_interval}, range: {gql_range}) {{ indicators({conn_args}) {} errors {{ symbol message }} }} }}",
        syms_literal, selection
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = wrap_nested_connection(unwrap_field(json, "indicatorsBatch"), "indicators");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
