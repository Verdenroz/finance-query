use async_graphql::Variables;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_CANDLE_VALID_FIELDS, GQL_CHART_META_VALID_FIELDS, GQL_CHART_VALID_FIELDS,
        GQL_SPARK_VALID_FIELDS, gql_string_list_literal, unwrap_field, unwrap_ticker_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_selection, execute_gql_rest, interval_to_gql, range_to_gql};
use super::support::{default_interval, default_range};

#[derive(Deserialize)]
pub(crate) struct ChartQuery {
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Start date as Unix timestamp (seconds). When provided together with `end`,
    /// overrides `range` and uses absolute date boundaries.
    start: Option<i64>,
    /// End date as Unix timestamp (seconds). Defaults to now when `start` is set.
    end: Option<i64>,
    /// Include events (dividends, splits, capital gains) in response
    #[serde(default)]
    events: bool,
    /// Detect candlestick patterns and include per-candle signals in response.
    /// The `patterns` array aligns 1:1 with the `candles` array; `null` means
    /// no pattern was detected on that bar.
    #[serde(default)]
    patterns: bool,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// Query parameters for /v2/spark
#[derive(Deserialize)]
pub(crate) struct SparkQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BatchChartsQuery {
    /// Comma-separated symbols (required)
    symbols: String,
    #[serde(default = "default_interval")]
    interval: String,
    #[serde(default = "default_range")]
    range: String,
    /// Detect candlestick patterns and include per-candle signals in response.
    #[serde(default)]
    patterns: bool,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/chart/{symbol}
///
/// Query: `interval` (default 1d), `range` (default 1mo), `start` (opt i64),
/// `end` (opt i64), `events` (bool), `patterns` (bool), `fields` (csv)
pub(crate) async fn get_chart(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<ChartQuery>,
) -> impl IntoResponse {
    let selection = build_rest_chart_selection(params.fields.as_deref());

    if params.start.is_none() && params.end.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "`end` requires `start` to be set", "status": 400})),
        )
            .into_response();
    }

    let query = if let Some(start) = params.start {
        let end = params.end.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });
        let events_arg = if params.events { ", events: true" } else { "" };
        let patterns_arg = if params.patterns {
            ", patterns: true"
        } else {
            ""
        };
        format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(start: {start}, end: {end}{events_arg}{patterns_arg}) {selection} }} }}"
        )
    } else {
        let gql_interval = interval_to_gql(&params.interval);
        let gql_range = range_to_gql(&params.range);
        let events_arg = if params.events { ", events: true" } else { "" };
        let patterns_arg = if params.patterns {
            ", patterns: true"
        } else {
            ""
        };
        format!(
            "query GetChart($symbol: String!) {{ ticker(symbol: $symbol) {{ chart(interval: {gql_interval}, range: {gql_range}{events_arg}{patterns_arg}) {selection} }} }}"
        )
    };

    info!(
        "Fetching chart data for {} (events={}, patterns={}, fields={:?})",
        symbol, params.events, params.patterns, params.fields
    );

    let mut vars = Variables::default();
    vars.insert(async_graphql::Name::new("symbol"), symbol.clone().into());

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_ticker_field(data, "chart"))).into_response()
}

/// GET /v2/charts?symbols=<csv>&interval=<str>&range=<str>&patterns=<bool>
pub(crate) async fn get_batch_charts(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<BatchChartsQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let gql_interval = interval_to_gql(&params.interval);
    let gql_range = range_to_gql(&params.range);
    let patterns_arg = if params.patterns {
        ", patterns: true"
    } else {
        ""
    };

    // Top-level batch wrapper fields are "symbol"/"chart" (GqlSymbolChart);
    // "chart" is itself composite and needs its own nested sub-selection.
    let want_chart = params
        .fields
        .as_deref()
        .map(|f| f.split(',').any(|x| x.trim() == "chart"))
        .unwrap_or(true);
    let selection = if want_chart {
        format!(
            "{{ symbol chart {} }}",
            build_rest_chart_selection(params.fields.as_deref())
        )
    } else {
        "{ symbol }".to_string()
    };
    let syms_literal = gql_string_list_literal(&symbols);

    let query = format!(
        "query {{ charts(symbols: [{}], interval: {}, range: {}{}) {{ charts {} errors {{ symbol message }} }} }}",
        syms_literal, gql_interval, gql_range, patterns_arg, selection
    );

    info!(
        "Fetching batch charts for {} symbols (interval={}, range={}, patterns={})",
        symbols.len(),
        params.interval,
        params.range,
        params.patterns
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "charts"))).into_response()
}

/// GET /v2/spark
///
/// Batch fetch sparkline data for multiple symbols in a single request.
/// Optimized for rendering sparkline charts with only close prices.
///
/// Query: `symbols` (comma-separated, required), `interval` (default "1d"), `range` (default "1mo")
pub(crate) async fn get_spark(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<SparkQuery>,
) -> impl IntoResponse {
    let symbols: Vec<&str> = params.symbols.split(',').map(|s| s.trim()).collect();
    let gql_interval = interval_to_gql(&params.interval);
    let gql_range = range_to_gql(&params.range);
    let syms_literal = gql_string_list_literal(&symbols);
    let item_selection = build_rest_spark_selection(params.fields.as_deref());

    let query = format!(
        "query {{ spark(symbols: [{}], interval: {}, range: {}) {{ sparks {} errors {{ symbol message }} }} }}",
        syms_literal, gql_interval, gql_range, item_selection
    );

    info!(
        "Fetching spark data for {} symbols (interval={}, range={})",
        symbols.len(),
        params.interval,
        params.range
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "spark"))).into_response()
}

/// Build the `spark { ... }` per-item selection set, expanding `meta` with its
/// required nested sub-selection — mirrors `build_rest_chart_selection`.
fn build_rest_spark_selection(fields: Option<&str>) -> String {
    let top_selection = build_rest_selection(fields, GQL_SPARK_VALID_FIELDS);
    if !top_selection.contains("meta") {
        return top_selection;
    }
    let mut sel = String::from("{ ");
    for f in ["symbol", "timestamps", "closes", "interval", "range"] {
        if top_selection.contains(f) {
            sel.push_str(f);
            sel.push(' ');
        }
    }
    sel.push_str("meta ");
    sel.push_str(&build_rest_selection(None, GQL_CHART_META_VALID_FIELDS));
    sel.push_str(" }");
    sel
}

/// Build the `chart { ... }` (or nested `charts.chart { ... }`) selection
/// set, expanding `meta`/`candles` with their required nested sub-selection
/// — mirrors `build_chart_selection` in finance-query-mcp/src/tools/chart.rs.
pub(crate) fn build_rest_chart_selection(fields: Option<&str>) -> String {
    let top_selection = build_rest_selection(fields, GQL_CHART_VALID_FIELDS);
    let want_meta = top_selection.contains("meta");
    let want_candles = top_selection.contains("candles");
    if !want_meta && !want_candles {
        return top_selection;
    }
    let mut sel = String::from("{ ");
    if top_selection.contains("symbol") {
        sel.push_str("symbol ");
    }
    if want_meta {
        sel.push_str("meta ");
        sel.push_str(&build_rest_selection(None, GQL_CHART_META_VALID_FIELDS));
        sel.push(' ');
    }
    if want_candles {
        sel.push_str("candles ");
        sel.push_str(&build_rest_selection(None, GQL_CANDLE_VALID_FIELDS));
        sel.push(' ');
    }
    sel.push('}');
    sel
}
