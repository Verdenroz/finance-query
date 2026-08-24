use finance_query::TimeRange;
use finance_query_server::graphql::FinanceSchema;
use finance_query_server::params::MarketCalendarKindParam;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    CALENDAR_EVENT_UNION_SELECTION, GQL_CALENDAR_VALID_FIELDS, GQL_MARKET_CALENDAR_VALID_FIELDS,
    MARKET_CALENDAR_DETAIL_UNION_SELECTION, build_union_selection, escape_gql_string,
    execute_query, gql_string_list_literal, parse_fields, unwrap_field,
};
use crate::tools::helpers::range_to_gql;

/// Build the `calendar { ... }` selection set, expanding `event` with its
/// full union inline-fragment selection.
fn build_calendar_selection(fields: Option<&[String]>) -> String {
    build_union_selection(
        fields,
        GQL_CALENDAR_VALID_FIELDS,
        "event",
        &["timestamp", "date", "symbol"],
        CALENDAR_EVENT_UNION_SELECTION,
    )
}

/// Aggregate upcoming financial events (earnings, dividends, options
/// expirations, and — when `FRED_API_KEY` is set — economic releases) across
/// the given symbols into a single time-sorted list.
pub async fn get_calendar(
    schema: &FinanceSchema,
    symbols: String,
    range: Option<TimeRange>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let gql_range = range_to_gql(range.unwrap_or(TimeRange::OneMonth));
    let syms: Vec<String> = symbols.split(',').map(|s| s.trim().to_string()).collect();
    let syms_literal = gql_string_list_literal(&syms);
    let field_list = parse_fields(fields);
    let selection = build_calendar_selection(field_list.as_deref());

    let query =
        format!("query {{ calendar(symbols: [{syms_literal}], range: {gql_range}) {selection} }}");
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "calendar");
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&data).map_err(ser_err)?,
    )]))
}

/// Build the `marketCalendar { ... }` selection set, expanding `detail` with
/// its full union inline-fragment selection.
fn build_market_calendar_selection(fields: Option<&[String]>) -> String {
    build_union_selection(
        fields,
        GQL_MARKET_CALENDAR_VALID_FIELDS,
        "detail",
        &["symbol", "date"],
        MARKET_CALENDAR_DETAIL_UNION_SELECTION,
    )
}

/// Market-wide event calendar (earnings/IPO/dividend/split/economic/holiday/
/// status) over `[from, to]` (`YYYY-MM-DD`). `from`/`to` are ignored by
/// `market-holiday`/`market-status`, which return providers' upcoming/live
/// sets regardless.
pub async fn get_market_calendar(
    schema: &FinanceSchema,
    kind: MarketCalendarKindParam,
    from: Option<String>,
    to: Option<String>,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let field_list = parse_fields(fields);
    let selection = build_market_calendar_selection(field_list.as_deref());
    let gql_kind = kind.as_gql_str();
    let from = from.unwrap_or_default();
    let to = to.unwrap_or_default();

    let query = format!(
        "query {{ marketCalendar(kind: {gql_kind}, from: \"{}\", to: \"{}\") {selection} }}",
        escape_gql_string(&from),
        escape_gql_string(&to)
    );
    let json = execute_query(schema, &query, async_graphql::Variables::default()).await?;
    let data = unwrap_field(json, "marketCalendar");
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
    fn build_calendar_selection_defaults_to_all_fields_including_event_union() {
        let selection = build_calendar_selection(None);

        assert!(selection.contains("timestamp"));
        assert!(selection.contains("date"));
        assert!(selection.contains("symbol"));
        assert!(selection.contains("event"));
        assert!(selection.contains("__typename"));
        assert!(selection.contains("GqlEarningsEvent"));
    }

    #[test]
    fn build_calendar_selection_falls_back_to_all_fields_when_list_is_empty() {
        let empty = fields(&[]);
        let selection = build_calendar_selection(Some(&empty));

        assert!(selection.contains("event"));
        assert!(selection.contains("__typename"));
    }

    #[test]
    fn build_calendar_selection_without_event_returns_flat_selection() {
        let requested = fields(&["timestamp", "symbol"]);
        let selection = build_calendar_selection(Some(&requested));

        assert_eq!(selection, "{ timestamp symbol }");
        assert!(!selection.contains("event"));
        assert!(!selection.contains("__typename"));
    }

    #[test]
    fn build_calendar_selection_with_event_expands_union_and_omits_unrequested_scalars() {
        let requested = fields(&["symbol", "event"]);
        let selection = build_calendar_selection(Some(&requested));

        assert!(selection.contains("symbol"));
        assert!(selection.contains("event"));
        assert!(selection.contains("__typename"));
        assert!(!selection.contains("timestamp"));
        assert!(!selection.contains("date "));
    }

    #[test]
    fn build_calendar_selection_drops_unknown_field_names() {
        let requested = fields(&["bogusField"]);
        let selection = build_calendar_selection(Some(&requested));

        assert_eq!(selection, "{ }");
    }

    #[test]
    fn build_calendar_selection_is_injection_safe_against_unmatched_strings() {
        let requested = fields(&["\") { __schema", "event } evil"]);
        let selection = build_calendar_selection(Some(&requested));

        assert_eq!(selection, "{ }");
        assert!(!selection.contains("__schema"));
    }
}
