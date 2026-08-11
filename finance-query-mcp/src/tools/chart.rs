use finance_query::{Interval, TimeRange};
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
use crate::tools::helpers::{interval_to_gql, range_to_gql};

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
    for f in ["symbol", "interval", "range"] {
        if chosen.contains(&f) {
            sel.push_str(f);
            sel.push(' ');
        }
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

/// Split a comma-separated symbols param into a trimmed, non-empty list.
fn split_symbols(symbols: &str) -> Vec<String> {
    symbols
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Accepts one or more comma-separated symbols: a single symbol returns the
/// flat chart shape (and honors `start`/`end`), multiple symbols return the
/// batch `{charts, errors}` shape (`start`/`end` are ignored for batch — the
/// underlying batch query only supports `interval`/`range`).
#[allow(clippy::too_many_arguments)]
pub async fn get_chart(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    start: Option<i64>,
    end: Option<i64>,
    fields: Option<String>,
    limit: Option<u32>,
    cursor: Option<String>,
) -> Result<CallToolResult, McpError> {
    let syms = split_symbols(&symbols);
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

/// Build the single-symbol `chart(...)` query text: absolute `start`/`end`
/// timestamps take priority over `interval`/`range` (mirrors the original
/// inline branching in `get_one_chart` exactly, including the "now" fallback
/// for a missing `end`).
fn build_chart_query(
    selection: &str,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    start: Option<i64>,
    end: Option<i64>,
) -> String {
    if let Some(start) = start {
        let end = end.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });
        format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(start: {start}, end: {end}) {selection} }} }}"
        )
    } else {
        let gql_interval = interval_to_gql(interval.unwrap_or(Interval::OneDay));
        let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneMonth));
        format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(interval: {gql_interval}, range: {gql_range}) {selection} }} }}"
        )
    }
}

#[allow(clippy::too_many_arguments)]
async fn get_one_chart(
    schema: &FinanceSchema,
    symbol: String,
    interval: Option<Interval>,
    range: Option<TimeRange>,
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
    let query = build_chart_query(&selection, interval, range, start, end);
    let mut variables = async_graphql::Variables::default();
    variables.insert(async_graphql::Name::new("symbol"), symbol.into());

    let json = execute_query(schema, &query, variables).await?;

    let chart = wrap_nested_connection(unwrap_ticker_field(json, "chart"), "candles");

    let text = serde_json::to_string(&chart).map_err(ser_err)?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        text,
    )]))
}

/// Build the outer `{ symbol [chart <chart_sel>] }` selection for a batch
/// charts response, omitting the nested `chart` block entirely when not
/// requested.
fn build_batch_chart_selection(want_chart: bool, chart_sel: &str) -> String {
    let mut selection = String::from("{ symbol ");
    if want_chart {
        selection.push_str("chart ");
        selection.push_str(chart_sel);
        selection.push(' ');
    }
    selection.push('}');
    selection
}

async fn get_many_charts(
    schema: &FinanceSchema,
    syms: Vec<String>,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.unwrap_or(Interval::OneDay));
    let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneMonth));

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

    let selection = build_batch_chart_selection(want_chart, &chart_sel);

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

/// Build the `spark(...)` selection: `meta` (when requested) is expanded
/// with its own nested sub-selection, every other chosen field is emitted
/// bare.
fn build_spark_selection(fields: Option<&[String]>) -> String {
    let chosen: Vec<&str> = match fields {
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
    selection
}

pub async fn get_spark(
    schema: &FinanceSchema,
    symbols: String,
    interval: Option<Interval>,
    range: Option<TimeRange>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_interval = interval_to_gql(interval.unwrap_or(Interval::OneDay));
    let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneMonth));

    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();
    let syms_literal = gql_string_list_literal(&syms);

    let field_list = parse_fields(fields);
    let selection = build_spark_selection(field_list.as_deref());

    let query = format!(
        "query {{ spark(symbols: [{syms_literal}], interval: {gql_interval}, range: {gql_range}) {{ sparks {selection} errors {{ symbol message }} }} }}"
    );

    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "spark");
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
    fn build_chart_selection_defaults_include_meta_and_candles() {
        let selection = build_chart_selection(None, Some(10), None);

        assert!(selection.contains("symbol"));
        assert!(selection.contains("meta"));
        assert!(selection.contains("candles"));
        assert!(selection.contains("first: 10"));
    }

    #[test]
    fn build_chart_selection_flat_when_neither_meta_nor_candles_requested() {
        let requested = fields(&["symbol", "interval"]);
        let selection = build_chart_selection(Some(&requested), None, None);

        assert_eq!(selection, "{ symbol interval }");
        assert!(!selection.contains("candles"));
    }

    #[test]
    fn build_chart_selection_omits_candles_args_when_limit_and_cursor_absent() {
        let requested = fields(&["candles"]);
        let selection = build_chart_selection(Some(&requested), None, None);

        assert!(selection.contains("candles"));
        assert!(!selection.contains("first:"));
        assert!(!selection.contains("after:"));
    }

    #[test]
    fn build_chart_selection_includes_cursor_arg_when_present() {
        let requested = fields(&["candles"]);
        let selection = build_chart_selection(Some(&requested), Some(5), Some("cur123"));

        assert!(selection.contains("first: 5"));
        assert!(selection.contains("after: \"cur123\""));
    }

    #[test]
    fn build_chart_selection_drops_unknown_fields() {
        let requested = fields(&["bogusField"]);
        let selection = build_chart_selection(Some(&requested), None, None);

        assert_eq!(selection, "{ }");
    }

    #[test]
    fn split_symbols_trims_and_filters_empty_entries() {
        assert_eq!(split_symbols("AAPL"), vec!["AAPL".to_string()]);
        assert_eq!(
            split_symbols("AAPL, MSFT ,GOOG"),
            vec!["AAPL".to_string(), "MSFT".to_string(), "GOOG".to_string()]
        );
        assert_eq!(
            split_symbols("AAPL,,MSFT,"),
            vec!["AAPL".to_string(), "MSFT".to_string()]
        );
    }

    #[test]
    fn split_symbols_returns_empty_vec_for_blank_input() {
        assert_eq!(split_symbols(""), Vec::<String>::new());
        assert_eq!(split_symbols("   "), Vec::<String>::new());
        assert_eq!(split_symbols(","), Vec::<String>::new());
    }

    #[test]
    fn build_chart_query_with_start_only_defaults_end_to_now() {
        let query = build_chart_query("{ symbol }", None, None, Some(100), None);

        assert!(query.contains("start: 100"));
        assert!(query.contains("end:"));
        assert!(!query.contains("interval:"));
    }

    #[test]
    fn build_chart_query_with_start_and_end_uses_both_explicitly() {
        let query = build_chart_query("{ symbol }", None, None, Some(100), Some(200));

        assert!(query.contains("start: 100"));
        assert!(query.contains("end: 200"));
    }

    #[test]
    fn build_chart_query_without_start_uses_interval_and_range() {
        let query = build_chart_query(
            "{ symbol }",
            Some(Interval::OneHour),
            Some(TimeRange::FiveDays),
            None,
            None,
        );

        assert!(query.contains("interval: ONE_HOUR"));
        assert!(query.contains("range: FIVE_DAYS"));
        assert!(!query.contains("start:"));
    }

    #[test]
    fn build_chart_query_without_start_defaults_interval_and_range() {
        let query = build_chart_query("{ symbol }", None, None, None, None);

        assert!(query.contains("interval: ONE_DAY"));
        assert!(query.contains("range: ONE_MONTH"));
    }

    #[test]
    fn build_batch_chart_selection_includes_chart_block_when_requested() {
        let selection = build_batch_chart_selection(true, "{ candles { edges { } } }");

        assert!(selection.contains("symbol"));
        assert!(selection.contains("chart"));
        assert!(selection.contains("candles"));
    }

    #[test]
    fn build_batch_chart_selection_omits_chart_block_when_not_requested() {
        let selection = build_batch_chart_selection(false, "{ candles { } }");

        assert_eq!(selection, "{ symbol }");
        assert!(!selection.contains("chart"));
    }

    #[test]
    fn build_spark_selection_defaults_omit_meta() {
        let selection = build_spark_selection(None);

        assert!(selection.contains("symbol"));
        assert!(selection.contains("closes"));
        assert!(!selection.contains("meta"));
    }

    #[test]
    fn build_spark_selection_expands_meta_when_requested() {
        let requested = fields(&["symbol", "meta"]);
        let selection = build_spark_selection(Some(&requested));

        assert!(selection.contains("symbol"));
        assert!(selection.contains("meta"));
        assert!(selection.contains("currency"));
    }

    #[test]
    fn build_spark_selection_drops_unknown_fields() {
        let requested = fields(&["bogusField"]);
        let selection = build_spark_selection(Some(&requested));

        assert_eq!(selection, "{ }");
    }
}
