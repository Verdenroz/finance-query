use async_graphql::{Name, Variables};
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use finance_query_server::graphql::{
    self,
    fields::{
        GQL_CURRENCY_VALID_FIELDS, GQL_EXCHANGE_VALID_FIELDS, GQL_MARKET_HOURS_VALID_FIELDS,
        GQL_QUOTE_TYPE_VALID_FIELDS, MARKET_HOURS_COMPOSITE_FIELDS, escape_gql_string,
        unwrap_field,
    },
};
use serde::Deserialize;
use tracing::info;

use super::gql_bridge::{build_rest_composite_selection, build_rest_selection, execute_gql_rest};

/// Query parameters for /v2/hours
#[derive(Deserialize)]
pub(crate) struct HoursQuery {
    /// Region code (e.g., "US", "JP", "GB"). Defaults to US if not specified.
    region: Option<String>,
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/hours
///
/// Query: `region` (string, optional - e.g., "US", "JP", "GB")
pub(crate) async fn get_hours(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<HoursQuery>,
) -> impl IntoResponse {
    let selection = build_rest_composite_selection(
        params.fields.as_deref(),
        GQL_MARKET_HOURS_VALID_FIELDS,
        MARKET_HOURS_COMPOSITE_FIELDS,
    );
    let region_arg = params
        .region
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| format!("(region: \"{}\")", escape_gql_string(r)));
    let query = format!(
        "query {{ marketHours{} {} }}",
        region_arg.unwrap_or_default(),
        selection
    );

    info!(
        "Fetching market hours for region: {}",
        params.region.as_deref().unwrap_or("US")
    );

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "marketHours"))).into_response()
}

/// Query parameters for /v2/quote-type/{symbol}
#[derive(Deserialize)]
pub(crate) struct QuoteTypeQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/quote-type/{symbol}
pub(crate) async fn get_quote_type(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Path(symbol): Path<String>,
    Query(params): Query<QuoteTypeQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_QUOTE_TYPE_VALID_FIELDS);
    let query = format!(
        "query GetQuoteType($symbol: String!) {{ quoteType(symbol: $symbol) {selection} }}"
    );
    let mut vars = Variables::default();
    vars.insert(Name::new("symbol"), symbol.clone().into());

    info!("Fetching quote type for {}", symbol);

    let data = match execute_gql_rest(&schema, &query, vars).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "quoteType"))).into_response()
}

/// Query parameters for /v2/currencies
#[derive(Deserialize)]
pub(crate) struct CurrenciesQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/currencies
///
/// Returns available currencies from Yahoo Finance.
pub(crate) async fn get_currencies(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<CurrenciesQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_CURRENCY_VALID_FIELDS);
    let query = format!("query {{ currencies {selection} }}");

    info!("Fetching currencies");

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "currencies"))).into_response()
}

/// Query parameters for /v2/exchanges
#[derive(Deserialize)]
pub(crate) struct ExchangesQuery {
    /// Comma-separated list of fields to include in response
    fields: Option<String>,
}

/// GET /v2/exchanges
///
/// Returns list of supported exchanges with their suffixes and data providers.
pub(crate) async fn get_exchanges(
    Extension(schema): Extension<graphql::FinanceSchema>,
    Query(params): Query<ExchangesQuery>,
) -> impl IntoResponse {
    let selection = build_rest_selection(params.fields.as_deref(), GQL_EXCHANGE_VALID_FIELDS);
    let query = format!("query {{ exchanges {selection} }}");

    info!("Fetching exchanges");

    let data = match execute_gql_rest(&schema, &query, Variables::default()).await {
        Ok(d) => d,
        Err(resp) => return resp,
    };
    (StatusCode::OK, Json(unwrap_field(data, "exchanges"))).into_response()
}
