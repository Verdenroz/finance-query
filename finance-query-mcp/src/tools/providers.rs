//! `get_forex`/`get_futures`/`get_commodity` tools: thin wrappers over the
//! `Providers`/domain-handle API (`finance_query::{Providers, ForexPair,
//! FuturesContract, Commodity}`) rather than the Yahoo-only `finance::`
//! shortcut the rest of this crate's tools bridge through GraphQL.
//!
//! These domains have no GraphQL query field today — the server crate never
//! enables the `alphavantage`/`fmp`/`polygon` library features (it only ever
//! talks to Yahoo/keyless sources), and wiring a brand-new provider-routed
//! GraphQL surface (services layer + types + REST handler) is a much larger
//! cross-cutting change than "MCP parity" calls for. So these tools call the
//! library directly and do their own small `fields` filter on the resulting
//! JSON object instead of a GraphQL selection set.
//!
//! Each tool is independently feature-gated (`get_futures` needs only
//! `polygon`; `get_commodity` needs `fmp` or `alphavantage`; `get_forex`
//! needs any of the three) so a minimal MCP build without paid-provider
//! features still compiles.

use finance_query::{Capability, Provider, Providers};
use rmcp::{ErrorData as McpError, model::CallToolResult};
use serde_json::{Map, Value};

use crate::error::{lib_err, ser_err};
use crate::tools::gql::parse_fields;

/// Keep exactly the requested top-level keys of a serialized quote object.
/// `fields` names are the model's own (snake_case) Rust field names — there's
/// no GraphQL type here to define camelCase names against. Falls back to the
/// full object when `fields` is `None`/empty/matches nothing (these quote
/// types are already tiny, so "everything" is a perfectly good default).
fn filter_fields(value: Value, fields: Option<&[String]>, valid_fields: &[&str]) -> Value {
    let (Some(obj), Some(fields)) = (value.as_object(), fields) else {
        return value;
    };
    let mut out = Map::new();
    for f in fields {
        if valid_fields.contains(&f.as_str())
            && let Some(v) = obj.get(f.as_str())
        {
            out.insert(f.clone(), v.clone());
        }
    }
    if out.is_empty() {
        value
    } else {
        Value::Object(out)
    }
}

fn respond(value: Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string(&value).map_err(ser_err)?,
    )]))
}

/// `ProviderAdapter::initialize()` reads its provider's API-key env var and
/// fails the *entire* `Providers::build()` call if it's routed but unset —
/// so the candidate list passed to `.route()` must be pre-filtered to only
/// providers that are both compiled in and have a configured key.
#[cfg(feature = "polygon")]
fn polygon_if_configured(out: &mut Vec<Provider>) {
    if std::env::var("POLYGON_API_KEY").is_ok() {
        out.push(Provider::Polygon);
    }
}

#[cfg(feature = "fmp")]
fn fmp_if_configured(out: &mut Vec<Provider>) {
    if std::env::var("FMP_API_KEY").is_ok() {
        out.push(Provider::Fmp);
    }
}

#[cfg(feature = "alphavantage")]
fn alphavantage_if_configured(out: &mut Vec<Provider>) {
    if std::env::var("ALPHAVANTAGE_API_KEY").is_ok() {
        out.push(Provider::AlphaVantage);
    }
}

#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
fn forex_route() -> Vec<Provider> {
    let mut v = Vec::new();
    #[cfg(feature = "polygon")]
    polygon_if_configured(&mut v);
    #[cfg(feature = "fmp")]
    fmp_if_configured(&mut v);
    #[cfg(feature = "alphavantage")]
    alphavantage_if_configured(&mut v);
    v
}

#[cfg(feature = "polygon")]
fn futures_route() -> Vec<Provider> {
    let mut v = Vec::new();
    polygon_if_configured(&mut v);
    v
}

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
fn commodity_route() -> Vec<Provider> {
    let mut v = Vec::new();
    #[cfg(feature = "fmp")]
    fmp_if_configured(&mut v);
    #[cfg(feature = "alphavantage")]
    alphavantage_if_configured(&mut v);
    v
}

const FOREX_VALID_FIELDS: &[&str] = &[
    "symbol",
    "base_currency",
    "quote_currency",
    "bid",
    "ask",
    "price",
    "change",
    "change_percent",
    "timestamp",
];

#[cfg(feature = "polygon")]
const FUTURES_VALID_FIELDS: &[&str] = &[
    "symbol",
    "name",
    "underlying",
    "exchange",
    "expiration_date",
    "price",
    "change",
    "change_percent",
    "open_interest",
    "volume",
    "timestamp",
];

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
const COMMODITY_VALID_FIELDS: &[&str] = &[
    "symbol",
    "name",
    "unit",
    "price",
    "change",
    "change_percent",
    "timestamp",
];

#[cfg(any(feature = "polygon", feature = "fmp", feature = "alphavantage"))]
pub async fn get_forex(
    from: String,
    to: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let route = forex_route();
    if route.is_empty() {
        return Err(crate::error::invalid_params(
            "No forex-capable provider configured. Set one of FMP_API_KEY, POLYGON_API_KEY, or ALPHAVANTAGE_API_KEY.",
        ));
    }
    let providers = Providers::builder()
        .route(Capability::FOREX, route)
        .build()
        .await
        .map_err(lib_err)?;
    let quote = providers.forex(from, to).quote().await.map_err(lib_err)?;
    let value = serde_json::to_value(quote).map_err(ser_err)?;
    let field_list = parse_fields(fields);
    respond(filter_fields(
        value,
        field_list.as_deref(),
        FOREX_VALID_FIELDS,
    ))
}

#[cfg(feature = "polygon")]
pub async fn get_futures(
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let route = futures_route();
    if route.is_empty() {
        return Err(crate::error::invalid_params(
            "No futures-capable provider configured. Set POLYGON_API_KEY.",
        ));
    }
    let providers = Providers::builder()
        .route(Capability::FUTURES, route)
        .build()
        .await
        .map_err(lib_err)?;
    let quote = providers.futures(symbol).quote().await.map_err(lib_err)?;
    let value = serde_json::to_value(quote).map_err(ser_err)?;
    let field_list = parse_fields(fields);
    respond(filter_fields(
        value,
        field_list.as_deref(),
        FUTURES_VALID_FIELDS,
    ))
}

#[cfg(any(feature = "fmp", feature = "alphavantage"))]
pub async fn get_commodity(
    symbol: String,
    fields: Option<String>,
) -> Result<CallToolResult, McpError> {
    let route = commodity_route();
    if route.is_empty() {
        return Err(crate::error::invalid_params(
            "No commodity-capable provider configured. Set FMP_API_KEY or ALPHAVANTAGE_API_KEY.",
        ));
    }
    let providers = Providers::builder()
        .route(Capability::COMMODITIES, route)
        .build()
        .await
        .map_err(lib_err)?;
    let quote = providers.commodity(symbol).quote().await.map_err(lib_err)?;
    let value = serde_json::to_value(quote).map_err(ser_err)?;
    let field_list = parse_fields(fields);
    respond(filter_fields(
        value,
        field_list.as_deref(),
        COMMODITY_VALID_FIELDS,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn filter_fields_keeps_only_requested_valid_keys() {
        let value = json!({"symbol": "EURUSD", "bid": 1.1, "ask": 1.2, "price": 1.15});
        let fields = vec!["symbol".to_string(), "price".to_string()];
        let filtered = filter_fields(value, Some(&fields), FOREX_VALID_FIELDS);
        assert_eq!(filtered, json!({"symbol": "EURUSD", "price": 1.15}));
    }

    #[test]
    fn filter_fields_ignores_unknown_field_names() {
        let value = json!({"symbol": "EURUSD", "bid": 1.1});
        let fields = vec!["symbol".to_string(), "not_a_real_field".to_string()];
        let filtered = filter_fields(value, Some(&fields), FOREX_VALID_FIELDS);
        assert_eq!(filtered, json!({"symbol": "EURUSD"}));
    }

    #[test]
    fn filter_fields_returns_full_object_when_fields_is_none() {
        let value = json!({"symbol": "EURUSD", "bid": 1.1});
        let filtered = filter_fields(value.clone(), None, FOREX_VALID_FIELDS);
        assert_eq!(filtered, value);
    }

    #[test]
    fn filter_fields_falls_back_to_full_object_when_selection_is_empty() {
        let value = json!({"symbol": "EURUSD", "bid": 1.1});
        let fields = vec!["not_a_real_field".to_string()];
        let filtered = filter_fields(value.clone(), Some(&fields), FOREX_VALID_FIELDS);
        assert_eq!(filtered, value);
    }

    // No cfg needed here: this whole file is only compiled when at least one
    // of polygon/fmp/alphavantage is enabled (see the `pub mod providers;`
    // gate in `tools/mod.rs`), which is exactly `forex_route`'s own condition.
    #[test]
    fn forex_route_is_empty_without_any_configured_key() {
        // SAFETY: test-only env mutation; no other test in this process reads
        // these particular vars concurrently in a way that would race.
        unsafe {
            std::env::remove_var("FMP_API_KEY");
            std::env::remove_var("POLYGON_API_KEY");
            std::env::remove_var("ALPHAVANTAGE_API_KEY");
        }
        assert!(forex_route().is_empty());
    }
}
