//! GraphQL type for options chain data.

use super::batch::GqlBatchError;
use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Result, SimpleObject};
use serde::Deserialize;

/// An option contract (call or put).
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlOptionContract {
    pub contract_symbol: String,
    pub strike: f64,
    pub currency: Option<String>,
    pub last_price: Option<f64>,
    pub change: Option<f64>,
    pub percent_change: Option<f64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub contract_size: Option<String>,
    pub expiration: Option<i64>,
    pub last_trade_date: Option<i64>,
    pub implied_volatility: Option<f64>,
    pub in_the_money: Option<bool>,
}

/// Options chain data for a symbol.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlOptions {
    /// Available expiration dates (Unix timestamps).
    pub expiration_dates: Vec<i64>,
    /// Available strike prices.
    pub strikes: Vec<f64>,
    /// All call contracts across expirations.
    #[graphql(skip)]
    pub calls: Vec<GqlOptionContract>,
    /// All put contracts across expirations.
    #[graphql(skip)]
    pub puts: Vec<GqlOptionContract>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlOptions {
    /// All call contracts across expirations.
    async fn calls(
        &self,
        #[graphql(desc = "Max contracts to return; omitted = every matching contract in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlOptionContract>> {
        pagination::paginate(&self.calls, first, after).await
    }

    /// All put contracts across expirations.
    async fn puts(
        &self,
        #[graphql(desc = "Max contracts to return; omitted = every matching contract in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlOptionContract>> {
        pagination::paginate(&self.puts, first, after).await
    }
}

/// Wraps a symbol name with its options chain, used by the batch
/// `optionsBatch` root field.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolOptions {
    pub symbol: String,
    pub options: GqlOptions,
}

/// Result of the batch `optionsBatch` root field: successfully fetched
/// options chains plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlOptionsBatch {
    #[graphql(skip)]
    pub options: Vec<GqlSymbolOptions>,
    pub errors: Vec<GqlBatchError>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlOptionsBatch {
    /// Successfully fetched per-symbol options chains.
    async fn options(
        &self,
        #[graphql(desc = "Max symbols to return; omitted = every matching symbol in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlSymbolOptions>> {
        pagination::paginate(&self.options, first, after).await
    }
}
