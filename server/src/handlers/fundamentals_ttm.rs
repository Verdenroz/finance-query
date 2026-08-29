//! Short-sale, grading, compensation and trailing-twelve-month endpoints.

use async_graphql::{Name, Variables};
use axum::{
    Extension, Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use finance_query_server::graphql::fields::{
    GQL_EXECUTIVE_COMPENSATION_VALID_FIELDS, GQL_GRADING_ACTION_VALID_FIELDS,
    GQL_KEY_METRICS_TTM_VALID_FIELDS, GQL_PRICE_TARGET_SUMMARY_VALID_FIELDS,
    GQL_RATIOS_TTM_VALID_FIELDS, GQL_SHORT_VOLUME_VALID_FIELDS, unwrap_ticker_field,
};
use finance_query_server::graphql::pagination::build_connection_selection;
use finance_query_server::params::{AnalysisQuery, FilingsQuery};
use tracing::info;

use super::gql_bridge::{
    build_rest_selection, connection_args, execute_gql_rest, unwrap_connection,
};
use crate::graphql;

/// One paginated ticker field bridged to REST.
macro_rules! paginated_ticker_endpoint {
    ($fn_name:ident, $gql_field:literal, $valid:ident, $log:literal) => {
        pub(crate) async fn $fn_name(
            Extension(schema): Extension<graphql::FinanceSchema>,
            Path(symbol): Path<String>,
            Query(params): Query<FilingsQuery>,
        ) -> impl IntoResponse {
            let inner = build_rest_selection(params.fields.as_deref(), $valid);
            let selection = build_connection_selection(&inner);
            let args = connection_args(params.limit, params.cursor.as_deref());
            let args = match args.is_empty() {
                true => String::new(),
                false => format!("({})", args.join(", ")),
            };
            let query = format!(
                "query Get($symbol: String!) {{ ticker(symbol: $symbol) {{ {}{args} {selection} }} }}",
                $gql_field
            );
            info!("Fetching {} for {}", $log, symbol);
            let mut vars = Variables::default();
            vars.insert(Name::new("symbol"), symbol.clone().into());
            let data = match execute_gql_rest(&schema, &query, vars).await {
                Ok(d) => d,
                Err(resp) => return *resp,
            };
            let paginated = params.limit.is_some() || params.cursor.is_some();
            let result = unwrap_connection(unwrap_ticker_field(data, $gql_field), paginated);
            (StatusCode::OK, Json(result)).into_response()
        }
    };
}

/// One scalar ticker field bridged to REST.
macro_rules! scalar_ticker_endpoint {
    ($fn_name:ident, $gql_field:literal, $valid:ident, $log:literal) => {
        pub(crate) async fn $fn_name(
            Extension(schema): Extension<graphql::FinanceSchema>,
            Path(symbol): Path<String>,
            Query(params): Query<AnalysisQuery>,
        ) -> impl IntoResponse {
            let selection = build_rest_selection(params.fields.as_deref(), $valid);
            let query = format!(
                "query Get($symbol: String!) {{ ticker(symbol: $symbol) {{ {} {selection} }} }}",
                $gql_field
            );
            info!("Fetching {} for {}", $log, symbol);
            let mut vars = Variables::default();
            vars.insert(Name::new("symbol"), symbol.clone().into());
            let data = match execute_gql_rest(&schema, &query, vars).await {
                Ok(d) => d,
                Err(resp) => return *resp,
            };
            (StatusCode::OK, Json(unwrap_ticker_field(data, $gql_field))).into_response()
        }
    };
}

// GET /v2/short-volume/{symbol}
paginated_ticker_endpoint!(
    get_short_volume,
    "shortVolume",
    GQL_SHORT_VOLUME_VALID_FIELDS,
    "short-sale volume"
);
// GET /v2/grading-actions/{symbol}
paginated_ticker_endpoint!(
    get_grading_actions,
    "gradingActions",
    GQL_GRADING_ACTION_VALID_FIELDS,
    "grading actions"
);
// GET /v2/executive-compensation/{symbol}
paginated_ticker_endpoint!(
    get_executive_compensation,
    "executiveCompensation",
    GQL_EXECUTIVE_COMPENSATION_VALID_FIELDS,
    "executive compensation"
);
// GET /v2/price-target-summary/{symbol}
scalar_ticker_endpoint!(
    get_price_target_summary,
    "priceTargetSummary",
    GQL_PRICE_TARGET_SUMMARY_VALID_FIELDS,
    "price target summary"
);
// GET /v2/key-metrics-ttm/{symbol}
scalar_ticker_endpoint!(
    get_key_metrics_ttm,
    "keyMetricsTtm",
    GQL_KEY_METRICS_TTM_VALID_FIELDS,
    "TTM key metrics"
);
// GET /v2/ratios-ttm/{symbol}
scalar_ticker_endpoint!(
    get_ratios_ttm,
    "ratiosTtm",
    GQL_RATIOS_TTM_VALID_FIELDS,
    "TTM ratios"
);
