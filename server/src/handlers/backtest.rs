//! Strategy backtesting over HTTP.

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::info;

use crate::graphql;
use crate::graphql::fields::{BACKTEST_COMPOSITE_FIELDS, GQL_BACKTEST_VALID_FIELDS};
use crate::graphql::pagination::unwrap_nested_connection;
use crate::handlers::gql_bridge::{execute_gql_rest, interval_to_gql, range_to_gql};
use finance_query_server::params::BacktestRequest;

/// Map a REST strategy id onto its GraphQL enum variant.
fn strategy_to_gql(strategy: &str) -> Option<&'static str> {
    match strategy {
        "sma_crossover" => Some("SMA_CROSSOVER"),
        "rsi_reversal" => Some("RSI_REVERSAL"),
        "macd_signal" => Some("MACD_SIGNAL"),
        "bollinger_mean_reversion" => Some("BOLLINGER_MEAN_REVERSION"),
        "supertrend_follow" => Some("SUPERTREND_FOLLOW"),
        "donchian_breakout" => Some("DONCHIAN_BREAKOUT"),
        _ => None,
    }
}

fn connection_args(limit: Option<i32>, cursor: Option<&str>) -> String {
    let mut args = Vec::new();
    if let Some(n) = limit {
        args.push(format!("first: {n}"));
    }
    if let Some(c) = cursor {
        args.push(format!(
            "after: {}",
            crate::graphql::fields::escape_gql_string(c)
        ));
    }
    match args.is_empty() {
        true => String::new(),
        false => format!("({})", args.join(", ")),
    }
}

/// Build the selection, attaching each connection's own pagination arguments.
/// `build_rest_composite_selection` handles a single connection; this field has
/// two that paginate independently.
fn build_selection(body: &BacktestRequest) -> String {
    let mut requested: Vec<&str> = match body.fields.as_deref() {
        Some(raw) if !raw.trim().is_empty() => raw
            .split(',')
            .map(str::trim)
            .filter(|f| GQL_BACKTEST_VALID_FIELDS.contains(f))
            .collect(),
        _ => GQL_BACKTEST_VALID_FIELDS.to_vec(),
    };
    // An all-unknown `fields` would otherwise emit an empty selection, which
    // is not valid GraphQL.
    if requested.is_empty() {
        requested = GQL_BACKTEST_VALID_FIELDS.to_vec();
    }
    let composite = |name: &str| {
        BACKTEST_COMPOSITE_FIELDS
            .iter()
            .find(|(field, _)| *field == name)
            .map(|(_, selection)| *selection)
    };
    let mut parts = Vec::new();
    for field in requested {
        let args = match field {
            "equityCurve" => connection_args(body.equity_limit, body.equity_cursor.as_deref()),
            "trades" => connection_args(body.trades_limit, body.trades_cursor.as_deref()),
            _ => String::new(),
        };
        match composite(field) {
            Some(selection) => parts.push(format!("{field}{args} {selection}")),
            None => parts.push(field.to_string()),
        }
    }
    format!("{{ {} }}", parts.join(" "))
}

/// POST /v2/backtest/{symbol}
pub(crate) async fn run_backtest(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Json(body): Json<BacktestRequest>,
) -> Response {
    let Some(gql_strategy) = strategy_to_gql(&body.strategy) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Unknown strategy '{}'", body.strategy),
                "valid": ["sma_crossover", "rsi_reversal", "macd_signal",
                          "bollinger_mean_reversion", "supertrend_follow", "donchian_breakout"],
            })),
        )
            .into_response();
    };

    let selection = build_selection(&body);
    let gql_interval = interval_to_gql(body.interval);
    let gql_range = range_to_gql(body.range);
    let vars = async_graphql::Variables::from_json(serde_json::json!({
        "params": body.params,
    }));
    let query = format!(
        "query Backtest($params: GqlBacktestParams!) {{ backtest(symbol: {}, strategy: {gql_strategy}, interval: {gql_interval}, range: {gql_range}, params: $params) {selection} }}",
        crate::graphql::fields::escape_gql_string(&symbol)
    );

    info!("Backtesting {} with {}", symbol, body.strategy);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return *resp,
    };
    let equity_paginated = body.equity_limit.is_some() || body.equity_cursor.is_some();
    let trades_paginated = body.trades_limit.is_some() || body.trades_cursor.is_some();
    let result = crate::graphql::fields::unwrap_field(data, "backtest");
    let result = unwrap_nested_connection(result, "equityCurve", equity_paginated);
    let result = unwrap_nested_connection(result, "trades", trades_paginated);
    (StatusCode::OK, Json(result)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_strategy_id_maps_to_a_gql_variant() {
        for id in [
            "sma_crossover",
            "rsi_reversal",
            "macd_signal",
            "bollinger_mean_reversion",
            "supertrend_follow",
            "donchian_breakout",
        ] {
            assert!(strategy_to_gql(id).is_some(), "{id} has no variant");
        }
        assert_eq!(strategy_to_gql("not_a_strategy"), None);
    }

    #[test]
    fn a_minimal_body_defaults_interval_and_range() {
        let body: BacktestRequest =
            serde_json::from_str(r#"{"strategy":"sma_crossover"}"#).expect("parses");
        assert_eq!(body.interval, finance_query::Interval::OneDay);
        assert_eq!(body.range, finance_query::TimeRange::OneYear);
        assert_eq!(body.params.fast_period, None);
    }

    #[test]
    fn camel_case_knobs_reach_the_shared_input_type() {
        let body: BacktestRequest = serde_json::from_str(
            r#"{"strategy":"rsi_reversal","params":{"fastPeriod":5,"allowShort":true}}"#,
        )
        .expect("parses");
        assert_eq!(body.params.fast_period, Some(5));
        assert_eq!(body.params.allow_short, Some(true));
    }

    #[test]
    fn connection_args_are_omitted_when_unpaginated() {
        assert_eq!(connection_args(None, None), "");
        assert_eq!(connection_args(Some(5), None), "(first: 5)");
        assert!(connection_args(Some(5), Some("abc")).starts_with("(first: 5, after: "));
    }
}
