use finance_query_server::graphql::FinanceSchema;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_CANDLE_DEFAULT_FIELDS, GQL_CANDLE_VALID_FIELDS,
    GQL_CHART_DEFAULT_FIELDS, GQL_CHART_META_DEFAULT_FIELDS, GQL_CHART_META_VALID_FIELDS,
    GQL_CHART_VALID_FIELDS, GQL_SPARK_DEFAULT_FIELDS, GQL_SPARK_VALID_FIELDS,
    build_connection_selection, build_selection_or_default, escape_gql_string, execute_query,
    gql_string_list_literal, parse_fields, unwrap_field, unwrap_ticker_field,
    wrap_nested_connection,
};
use crate::tools::helpers::{parse_interval, parse_range};

/// `limit`/`cursor` are `None` for batch callers (which don't expose
/// pagination params for the nested candles list).
fn build_chart_selection(
    fields: Option<&[String]>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    // Resolve the field list once, then read `meta`/`candles` membership
    // directly off it — cheaper and clearer than rebuilding a selection
    // string just to `.contains()`-scan it.
    let chosen: Vec<&str> = match fields {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_CHART_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_CHART_DEFAULT_FIELDS.to_vec(),
    };

    let want_meta = chosen.contains(&"meta");
    let want_candles = chosen.contains(&"candles");

    if !want_meta && !want_candles {
        let mut sel = String::from("{ ");
        for f in &chosen {
            sel.push_str(f);
            sel.push(' ');
        }
        sel.push('}');
        return sel;
    }

    // Rebuild with nested sub-selections.
    let mut sel = String::from("{ ");
    if chosen.contains(&"symbol") {
        sel.push_str("symbol ");
    }
    if want_meta {
        let meta_sel = build_selection_or_default(
            None, // caller didn't specify sub-fields; use default
            GQL_CHART_META_VALID_FIELDS,
            GQL_CHART_META_DEFAULT_FIELDS,
        );
        sel.push_str("meta ");
        sel.push_str(&meta_sel);
        sel.push(' ');
    }
    if want_candles {
        let candle_sel =
            build_selection_or_default(None, GQL_CANDLE_VALID_FIELDS, GQL_CANDLE_DEFAULT_FIELDS);
        let mut args = Vec::new();
        if let Some(limit) = limit {
            args.push(format!("first: {limit}"));
        }
        if let Some(cursor) = cursor {
            args.push(format!("after: \"{}\"", escape_gql_string(cursor)));
        }
        sel.push_str("candles");
        if !args.is_empty() {
            sel.push('(');
            sel.push_str(&args.join(", "));
            sel.push(')');
        }
        sel.push(' ');
        sel.push_str(&build_connection_selection(&candle_sel));
        sel.push(' ');
    }
    sel.push('}');
    sel
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat chart shape (and honors `start`/`end`), multiple symbols return the
/// batch `{charts, errors}` shape (`start`/`end` are ignored for batch — the
/// underlying batch query only supports `interval`/`range`).
#[allow(clippy::too_many_arguments)]
pub async fn get_chart(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<String>,
    range: Option<String>,
    start: Option<i64>,
    end: Option<i64>,
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
        get_one_chart(
            schema,
            syms.into_iter().next().unwrap(),
            interval,
            range,
            start,
            end,
            fields,
            limit,
            cursor,
        )
        .await
    } else {
        get_many_charts(schema, syms, interval, range, fields).await
    }
}

#[allow(clippy::too_many_arguments)]
async fn get_one_chart(
    schema: &FinanceSchema,
    symbol: String,
    interval: Option<String>,
    range: Option<String>,
    start: Option<i64>,
    end: Option<i64>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    if start.is_none() && end.is_some() {
        return Err(McpError::invalid_params(
            "`end` requires `start` to be set",
            None,
        ));
    }

    let field_list = parse_fields(fields);

    let selection = build_chart_selection(
        field_list.as_deref(),
        Some(limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE)),
        cursor.as_deref(),
    );

    // Build query based on whether start date is set.
    let (query, variables) = if let Some(start) = start {
        let end = end.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });
        let q = format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(start: {start}, end: {end}) {selection} }} }}"
        );
        let mut vars = async_graphql::Variables::default();
        vars.insert(async_graphql::Name::new("symbol"), symbol.into());
        (q, vars)
    } else {
        let interval_str = interval.as_deref().unwrap_or("1d");
        let range_str = range.as_deref().unwrap_or("1mo");
        let gql_interval = match parse_interval(interval_str) {
            finance_query::Interval::OneMinute => "ONE_MINUTE",
            finance_query::Interval::FiveMinutes => "FIVE_MINUTES",
            finance_query::Interval::FifteenMinutes => "FIFTEEN_MINUTES",
            finance_query::Interval::ThirtyMinutes => "THIRTY_MINUTES",
            finance_query::Interval::OneHour => "ONE_HOUR",
            finance_query::Interval::OneDay => "ONE_DAY",
            finance_query::Interval::OneWeek => "ONE_WEEK",
            finance_query::Interval::OneMonth => "ONE_MONTH",
            finance_query::Interval::ThreeMonths => "THREE_MONTHS",
        };
        let gql_range = match parse_range(range_str) {
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
        let q = format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(interval: {gql_interval}, range: {gql_range}) {selection} }} }}"
        );
        let mut vars = async_graphql::Variables::default();
        vars.insert(async_graphql::Name::new("symbol"), symbol.into());
        (q, vars)
    };

    let json = execute_query(schema, &query, variables).await?;

    let chart = wrap_nested_connection(unwrap_ticker_field(json, "chart"), "candles");

    let text = serde_json::to_string(&chart).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

async fn get_many_charts(
    schema: &FinanceSchema,
    syms: Vec<String>,
    interval: Option<String>,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval_str = interval.as_deref().unwrap_or("1d");
    let range_str = range.as_deref().unwrap_or("1mo");

    let gql_interval = match parse_interval(interval_str) {
        finance_query::Interval::OneMinute => "ONE_MINUTE",
        finance_query::Interval::FiveMinutes => "FIVE_MINUTES",
        finance_query::Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        finance_query::Interval::ThirtyMinutes => "THIRTY_MINUTES",
        finance_query::Interval::OneHour => "ONE_HOUR",
        finance_query::Interval::OneDay => "ONE_DAY",
        finance_query::Interval::OneWeek => "ONE_WEEK",
        finance_query::Interval::OneMonth => "ONE_MONTH",
        finance_query::Interval::ThreeMonths => "THREE_MONTHS",
    };
    let gql_range = match parse_range(range_str) {
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

    let syms_literal = gql_string_list_literal(&syms);

    let field_list = parse_fields(fields);

    // For batch charts, the top-level fields are "symbol" and "chart" (from GqlSymbolChart).
    // We allow selecting "chart" to get the nested GqlChart.
    let want_chart = field_list
        .as_ref()
        .map(|fs| fs.iter().any(|f| f == "chart"))
        .unwrap_or(true); // default: include chart

    let chart_sel = if want_chart {
        build_chart_selection(field_list.as_deref(), None, None)
    } else {
        String::new()
    };

    let mut selection = String::from("{ symbol ");
    if want_chart {
        selection.push_str("chart ");
        selection.push_str(&chart_sel);
        selection.push(' ');
    }
    selection.push('}');

    let query = format!(
        "query {{ charts(symbols: [{}], interval: {gql_interval}, range: {gql_range}) {{ charts {selection} errors {{ symbol message }} }} }}",
        syms_literal
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;

    let mut charts = unwrap_field(json, "charts");
    if let Some(items) = charts.get_mut("charts").and_then(|v| v.as_array_mut()) {
        for item in items.iter_mut() {
            if let Some(chart) = item.get("chart").cloned() {
                item["chart"] = wrap_nested_connection(chart, "candles");
            }
        }
    }

    let text = serde_json::to_string(&charts).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

pub async fn get_spark(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<String>,
    range: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let interval_str = interval.as_deref().unwrap_or("1d");
    let range_str = range.as_deref().unwrap_or("1mo");
    let gql_interval = match parse_interval(interval_str) {
        finance_query::Interval::OneMinute => "ONE_MINUTE",
        finance_query::Interval::FiveMinutes => "FIVE_MINUTES",
        finance_query::Interval::FifteenMinutes => "FIFTEEN_MINUTES",
        finance_query::Interval::ThirtyMinutes => "THIRTY_MINUTES",
        finance_query::Interval::OneHour => "ONE_HOUR",
        finance_query::Interval::OneDay => "ONE_DAY",
        finance_query::Interval::OneWeek => "ONE_WEEK",
        finance_query::Interval::OneMonth => "ONE_MONTH",
        finance_query::Interval::ThreeMonths => "THREE_MONTHS",
    };
    let gql_range = match parse_range(range_str) {
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

    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();
    let syms_literal = gql_string_list_literal(&syms);

    let field_list = parse_fields(fields);
    let chosen: Vec<&str> = match field_list.as_deref() {
        Some(fs) if !fs.is_empty() => fs
            .iter()
            .map(|f| f.trim())
            .filter(|f| GQL_SPARK_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_SPARK_DEFAULT_FIELDS.to_vec(),
    };
    let mut selection = String::from("{ ");
    for f in &chosen {
        if *f != "meta" {
            selection.push_str(f);
            selection.push(' ');
        }
    }
    if chosen.contains(&"meta") {
        let meta_sel = build_selection_or_default(
            None,
            GQL_CHART_META_VALID_FIELDS,
            GQL_CHART_META_DEFAULT_FIELDS,
        );
        selection.push_str("meta ");
        selection.push_str(&meta_sel);
        selection.push(' ');
    }
    selection.push('}');

    let query = format!(
        "query {{ spark(symbols: [{syms_literal}], interval: {gql_interval}, range: {gql_range}) {{ sparks {selection} errors {{ symbol message }} }} }}"
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "spark");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}
