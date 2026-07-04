//! GraphQL `QueryRoot` — top-level query fields.
//!
//! Split into several `MergedObject` pieces (and `GqlTicker` likewise into
//! several ticker-scoped pieces) instead of one giant `#[Object]`/
//! `#[ComplexObject]` impl. async-graphql's derive macros compile every field
//! on a type into a single dispatch fn, so its stack frame is sized for the
//! union of all fields — keeping each piece to a handful of fields keeps that
//! frame small regardless of how large the schema grows overall.

mod root_batch;
mod root_discovery;
mod root_market;
mod root_metadata;
mod ticker_analysis;
mod ticker_core;
mod ticker_events;
mod ticker_holders;

use async_graphql::{ErrorExtensions, MergedObject};

use root_batch::RootBatchQuery;
use root_discovery::RootDiscoveryQuery;
use root_market::RootMarketQuery;
use root_metadata::RootMetadataQuery;
use ticker_analysis::TickerAnalysisQuery;
use ticker_core::TickerCoreQuery;
use ticker_events::TickerEventsQuery;
use ticker_holders::TickerHoldersQuery;

use super::types::{batch::GqlBatchError, options::GqlOptions};

/// Normalize a GraphQL `lang` argument to a canonical translation target.
fn resolve_gql_lang(lang: Option<&str>) -> Option<String> {
    crate::lang::resolve_lang(lang, &axum::http::HeaderMap::new())
}

/// Pull the `errors: {symbol: message}` map every batch service response
/// carries (see `define_batch_response!` in the library) into the shared
/// `GqlBatchError` list every batch root field returns alongside its results.
fn extract_batch_errors(json: &serde_json::Value) -> Vec<GqlBatchError> {
    json.get("errors")
        .and_then(|v| v.as_object())
        .map(|errors| {
            errors
                .iter()
                .map(|(symbol, message)| GqlBatchError {
                    symbol: symbol.clone(),
                    message: message.as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Convert a library `Options` chain into its GraphQL representation —
/// shared by the single-symbol `options` and batch `optionsBatch` resolvers.
fn build_gql_options(opts: finance_query::Options) -> GqlOptions {
    use super::types::options::GqlOptionContract;

    let to_gql_contract = |c: &finance_query::OptionContract| GqlOptionContract {
        contract_symbol: c.contract_symbol.clone(),
        strike: c.strike,
        currency: c.currency.clone(),
        last_price: c.last_price,
        change: c.change,
        percent_change: c.percent_change,
        volume: c.volume,
        open_interest: c.open_interest,
        bid: c.bid,
        ask: c.ask,
        contract_size: c.contract_size.clone(),
        expiration: c.expiration,
        last_trade_date: c.last_trade_date,
        implied_volatility: c.implied_volatility,
        in_the_money: c.in_the_money,
    };
    let calls = opts.calls().iter().map(to_gql_contract).collect();
    let puts = opts.puts().iter().map(to_gql_contract).collect();
    GqlOptions {
        expiration_dates: opts.expiration_dates(),
        strikes: opts.strikes(),
        calls,
        puts,
    }
}

/// Map a custom-screener `ScreenerError` to a GraphQL error, matching the
/// REST `/v2/screeners/custom` status-code taxonomy (400 for invalid
/// field/operator, delegating to `to_gql_error` for upstream failures).
fn screener_error_to_gql(err: crate::services::screener::ScreenerError) -> async_graphql::Error {
    use crate::services::screener::ScreenerError;
    match err {
        ScreenerError::InvalidField(msg) | ScreenerError::InvalidOperator(msg) => {
            async_graphql::Error::new(msg).extend_with(|_, e| {
                e.set("code", "BAD_REQUEST");
                e.set("status", 400);
            })
        }
        ScreenerError::Finance(fe) => super::error::to_gql_error(Box::new(fe)),
    }
}

#[derive(MergedObject, Default)]
pub struct QueryRoot(
    RootBatchQuery,
    RootMarketQuery,
    RootDiscoveryQuery,
    RootMetadataQuery,
);

/// Entry point for per-symbol queries. Actual data fetching happens in each
/// merged piece's resolvers so only requested fields trigger network calls.
#[derive(MergedObject)]
pub struct GqlTicker(
    TickerCoreQuery,
    TickerEventsQuery,
    TickerHoldersQuery,
    TickerAnalysisQuery,
);

impl GqlTicker {
    pub(crate) fn new(symbol: String) -> Self {
        GqlTicker(
            TickerCoreQuery {
                symbol: symbol.clone(),
            },
            TickerEventsQuery {
                symbol: symbol.clone(),
            },
            TickerHoldersQuery {
                symbol: symbol.clone(),
            },
            TickerAnalysisQuery { symbol },
        )
    }
}
