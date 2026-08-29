//! Short-sale, grading, compensation and trailing-twelve-month tools.

use finance_query_server::graphql::FinanceSchema;
use finance_query_server::graphql::fields::{
    GQL_EXECUTIVE_COMPENSATION_VALID_FIELDS, GQL_GRADING_ACTION_VALID_FIELDS,
    GQL_KEY_METRICS_TTM_VALID_FIELDS, GQL_PRICE_TARGET_SUMMARY_VALID_FIELDS,
    GQL_RATIOS_TTM_VALID_FIELDS, GQL_SHORT_VOLUME_VALID_FIELDS, escape_gql_string,
    unwrap_ticker_field,
};
use finance_query_server::graphql::pagination::build_connection_selection;
use rmcp::{ErrorData as McpError, model::CallToolResult};

use crate::error::ser_err;
use crate::tools::gql::{
    DEFAULT_MCP_PAGE_SIZE, GQL_EXECUTIVE_COMPENSATION_DEFAULT_FIELDS,
    GQL_GRADING_ACTION_DEFAULT_FIELDS, GQL_KEY_METRICS_TTM_DEFAULT_FIELDS,
    GQL_PRICE_TARGET_SUMMARY_DEFAULT_FIELDS, GQL_RATIOS_TTM_DEFAULT_FIELDS,
    GQL_SHORT_VOLUME_DEFAULT_FIELDS, build_selection_or_default, execute_query, parse_fields,
    wrap_connection,
};

/// One paginated ticker field, always paged because MCP is the transport most
/// at risk of filling a context window.
macro_rules! paginated_tool {
    ($fn_name:ident, $gql_field:literal, $valid:ident, $default:ident) => {
        pub async fn $fn_name(
            schema: &FinanceSchema,
            symbol: String,
            fields: Option<String>,
            limit: Option<u32>,
            cursor: Option<String>,
        ) -> Result<CallToolResult, McpError> {
            let field_list = parse_fields(fields);
            let inner = build_selection_or_default(field_list.as_deref(), $valid, $default);
            let selection = build_connection_selection(&inner);
            let first = limit.unwrap_or(DEFAULT_MCP_PAGE_SIZE);
            let after = cursor
                .as_deref()
                .map(|c| format!(", after: \"{}\"", escape_gql_string(c)))
                .unwrap_or_default();
            let query = format!(
                "query Get($symbol: String!) {{ ticker(symbol: $symbol) {{ {}(first: {first}{after}) {selection} }} }}",
                $gql_field
            );
            let mut variables = async_graphql::Variables::default();
            variables.insert(async_graphql::Name::new("symbol"), symbol.into());
            let json = execute_query(schema, &query, variables).await?;
            let data = wrap_connection(unwrap_ticker_field(json, $gql_field));
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string(&data).map_err(ser_err)?,
            )]))
        }
    };
}

/// One scalar ticker field.
macro_rules! scalar_tool {
    ($fn_name:ident, $gql_field:literal, $valid:ident, $default:ident) => {
        pub async fn $fn_name(
            schema: &FinanceSchema,
            symbol: String,
            fields: Option<String>,
        ) -> Result<CallToolResult, McpError> {
            let field_list = parse_fields(fields);
            let selection = build_selection_or_default(field_list.as_deref(), $valid, $default);
            let query = format!(
                "query Get($symbol: String!) {{ ticker(symbol: $symbol) {{ {} {selection} }} }}",
                $gql_field
            );
            let mut variables = async_graphql::Variables::default();
            variables.insert(async_graphql::Name::new("symbol"), symbol.into());
            let json = execute_query(schema, &query, variables).await?;
            let data = unwrap_ticker_field(json, $gql_field);
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string(&data).map_err(ser_err)?,
            )]))
        }
    };
}

paginated_tool!(
    get_short_volume,
    "shortVolume",
    GQL_SHORT_VOLUME_VALID_FIELDS,
    GQL_SHORT_VOLUME_DEFAULT_FIELDS
);
paginated_tool!(
    get_grading_actions,
    "gradingActions",
    GQL_GRADING_ACTION_VALID_FIELDS,
    GQL_GRADING_ACTION_DEFAULT_FIELDS
);
paginated_tool!(
    get_executive_compensation,
    "executiveCompensation",
    GQL_EXECUTIVE_COMPENSATION_VALID_FIELDS,
    GQL_EXECUTIVE_COMPENSATION_DEFAULT_FIELDS
);
scalar_tool!(
    get_price_target_summary,
    "priceTargetSummary",
    GQL_PRICE_TARGET_SUMMARY_VALID_FIELDS,
    GQL_PRICE_TARGET_SUMMARY_DEFAULT_FIELDS
);
scalar_tool!(
    get_key_metrics_ttm,
    "keyMetricsTtm",
    GQL_KEY_METRICS_TTM_VALID_FIELDS,
    GQL_KEY_METRICS_TTM_DEFAULT_FIELDS
);
scalar_tool!(
    get_ratios_ttm,
    "ratiosTtm",
    GQL_RATIOS_TTM_VALID_FIELDS,
    GQL_RATIOS_TTM_DEFAULT_FIELDS
);
