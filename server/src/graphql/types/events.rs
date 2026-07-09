//! GraphQL types for corporate events: dividends, splits, capital gains.

use super::batch::GqlBatchError;
use crate::graphql::pagination::{self, Page};
use async_graphql::{ComplexObject, Result, SimpleObject};
use serde::Deserialize;

/// A single dividend payment.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlDividend {
    pub timestamp: i64,
    pub amount: f64,
}

/// A single stock split.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlSplit {
    pub timestamp: i64,
    pub numerator: f64,
    pub denominator: f64,
    pub ratio: String,
}

/// A single capital gain distribution.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
pub struct GqlCapitalGain {
    pub timestamp: i64,
    pub amount: f64,
}

/// Computed dividend analytics.
///
/// Mirrors `finance_query::models::chart::DividendAnalytics`, which has no
/// serde rename of its own (plain snake_case JSON keys) — must not rename
/// for deserialization either, even though GraphQL field names are camelCase.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlDividendAnalytics {
    pub total_paid: f64,
    pub payment_count: i64,
    pub average_payment: f64,
    pub cagr: Option<f64>,
    pub last_payment: Option<GqlDividend>,
    pub first_payment: Option<GqlDividend>,
}

/// Dividends response: payment history + computed analytics.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
#[serde(rename_all = "camelCase")]
pub struct GqlDividends {
    #[graphql(skip)]
    pub dividends: Vec<GqlDividend>,
    pub analytics: GqlDividendAnalytics,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlDividends {
    /// Dividend payment history.
    async fn dividends(
        &self,
        #[graphql(desc = "Max entries to return; omitted = every matching entry in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlDividend>> {
        pagination::paginate(&self.dividends, first, after).await
    }
}

/// Wrapper for batch dividends: `{symbol, dividends}`. `dividends` is a plain
/// payment list here (mirrors `BatchDividendsResponse.dividends:
/// HashMap<String, Vec<Dividend>>`) — batch dividends has no per-symbol
/// analytics, unlike the single-symbol `GqlDividends`.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolDividends {
    pub symbol: String,
    pub dividends: Vec<GqlDividend>,
}

/// Result of the batch `dividendsBatch` root field: successfully fetched
/// dividend histories plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase", complex)]
pub struct GqlDividendsBatch {
    #[graphql(skip)]
    pub dividends: Vec<GqlSymbolDividends>,
    pub errors: Vec<GqlBatchError>,
}

#[ComplexObject(rename_fields = "camelCase")]
impl GqlDividendsBatch {
    /// Successfully fetched per-symbol dividend histories.
    async fn dividends(
        &self,
        #[graphql(desc = "Max symbols to return; omitted = every matching symbol in one page")]
        first: Option<i32>,
        #[graphql(desc = "Opaque continuation cursor from a previous page's endCursor")]
        after: Option<String>,
    ) -> Result<Page<GqlSymbolDividends>> {
        pagination::paginate(&self.dividends, first, after).await
    }
}

/// Wrapper for batch splits: `{symbol, splits}`.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolSplits {
    pub symbol: String,
    pub splits: Vec<GqlSplit>,
}

/// Result of the batch `splitsBatch` root field: successfully fetched split
/// histories plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSplitsBatch {
    pub splits: Vec<GqlSymbolSplits>,
    pub errors: Vec<GqlBatchError>,
}

/// Wrapper for batch capital gains: `{symbol, capitalGains}`.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlSymbolCapitalGains {
    pub symbol: String,
    pub capital_gains: Vec<GqlCapitalGain>,
}

/// Result of the batch `capitalGainsBatch` root field: successfully fetched
/// capital gains histories plus any per-symbol fetch errors.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlCapitalGainsBatch {
    pub capital_gains: Vec<GqlSymbolCapitalGains>,
    pub errors: Vec<GqlBatchError>,
}
