//! GraphQL types for predefined and custom stock/fund screeners.
//!
//! `ScreenerQuote`'s `FormattedValue<T>` fields follow the same convention as
//! `GqlQuote`: the resolver runs `ValueFormat::transform()` on the raw JSON
//! first, so these are exposed as opaque `Json<GqlFormattedValue>` here
//! (scalar in raw mode, string in fmt mode, full object in both mode).

use super::formatted::GqlFormattedValue;
use crate::graphql::pagination::GqlPageInfo;
use async_graphql::{InputObject, Json, SimpleObject};
use serde::Deserialize;

/// A single quote result from a predefined or custom screener.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlScreenerQuote {
    pub symbol: String,
    pub short_name: String,
    pub long_name: Option<String>,
    pub display_name: Option<String>,
    pub quote_type: String,
    pub exchange: String,
    pub regular_market_price: Json<GqlFormattedValue>,
    pub regular_market_change: Json<GqlFormattedValue>,
    pub regular_market_change_percent: Json<GqlFormattedValue>,
    pub regular_market_open: Option<Json<GqlFormattedValue>>,
    pub regular_market_day_high: Option<Json<GqlFormattedValue>>,
    pub regular_market_day_low: Option<Json<GqlFormattedValue>>,
    pub regular_market_previous_close: Option<Json<GqlFormattedValue>>,
    pub regular_market_time: Option<Json<GqlFormattedValue>>,
    pub regular_market_volume: Option<Json<GqlFormattedValue>>,
    pub average_daily_volume3_month: Option<Json<GqlFormattedValue>>,
    pub average_daily_volume10_day: Option<Json<GqlFormattedValue>>,
    pub market_cap: Option<Json<GqlFormattedValue>>,
    pub shares_outstanding: Option<Json<GqlFormattedValue>>,
    pub fifty_two_week_high: Option<Json<GqlFormattedValue>>,
    pub fifty_two_week_low: Option<Json<GqlFormattedValue>>,
    pub fifty_two_week_change: Option<Json<GqlFormattedValue>>,
    pub fifty_two_week_change_percent: Option<Json<GqlFormattedValue>>,
    pub fifty_day_average: Option<Json<GqlFormattedValue>>,
    pub fifty_day_average_change: Option<Json<GqlFormattedValue>>,
    pub fifty_day_average_change_percent: Option<Json<GqlFormattedValue>>,
    pub two_hundred_day_average: Option<Json<GqlFormattedValue>>,
    pub two_hundred_day_average_change: Option<Json<GqlFormattedValue>>,
    pub two_hundred_day_average_change_percent: Option<Json<GqlFormattedValue>>,
    pub average_analyst_rating: Option<String>,
    #[graphql(name = "trailingPE")]
    #[serde(rename = "trailingPE")]
    pub trailing_pe: Option<Json<GqlFormattedValue>>,
    #[graphql(name = "forwardPE")]
    #[serde(rename = "forwardPE")]
    pub forward_pe: Option<Json<GqlFormattedValue>>,
    pub price_to_book: Option<Json<GqlFormattedValue>>,
    pub book_value: Option<Json<GqlFormattedValue>>,
    pub eps_trailing_twelve_months: Option<Json<GqlFormattedValue>>,
    pub eps_forward: Option<Json<GqlFormattedValue>>,
    pub eps_current_year: Option<Json<GqlFormattedValue>>,
    pub price_eps_current_year: Option<Json<GqlFormattedValue>>,
    pub dividend_yield: Option<Json<GqlFormattedValue>>,
    pub dividend_rate: Option<Json<GqlFormattedValue>>,
    pub dividend_date: Option<Json<GqlFormattedValue>>,
    pub trailing_annual_dividend_rate: Option<Json<GqlFormattedValue>>,
    pub trailing_annual_dividend_yield: Option<Json<GqlFormattedValue>>,
    pub bid: Option<Json<GqlFormattedValue>>,
    pub bid_size: Option<Json<GqlFormattedValue>>,
    pub ask: Option<Json<GqlFormattedValue>>,
    pub ask_size: Option<Json<GqlFormattedValue>>,
    pub post_market_price: Option<Json<GqlFormattedValue>>,
    pub post_market_change: Option<Json<GqlFormattedValue>>,
    pub post_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub post_market_time: Option<Json<GqlFormattedValue>>,
    pub pre_market_price: Option<Json<GqlFormattedValue>>,
    pub pre_market_change: Option<Json<GqlFormattedValue>>,
    pub pre_market_change_percent: Option<Json<GqlFormattedValue>>,
    pub pre_market_time: Option<Json<GqlFormattedValue>>,
    pub earnings_timestamp: Option<Json<GqlFormattedValue>>,
    pub earnings_timestamp_start: Option<Json<GqlFormattedValue>>,
    pub earnings_timestamp_end: Option<Json<GqlFormattedValue>>,
    pub currency: Option<String>,
}

/// Results from a predefined or custom screener.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlScreenerResults {
    pub quotes: Vec<GqlScreenerQuote>,
    #[graphql(name = "type")]
    #[serde(rename = "type")]
    pub screener_type: String,
    pub description: Option<String>,
    pub last_updated: Option<i64>,
    pub total: Option<i64>,
    /// Present only for `customScreener` (predefined `screener` has no
    /// underlying offset support upstream, so no pagination metadata to report).
    #[serde(skip)]
    pub page_info: Option<GqlPageInfo>,
}

/// A single filter condition for `customScreener`. `field`/`operator` are
/// validated against `EquityField`/`FundField` server-side (safe to splice
/// into the upstream Yahoo query only after that match succeeds) — same
/// validation as the REST `/v2/screeners/custom` endpoint.
#[derive(InputObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFilterCondition {
    /// Field name (e.g. "region", "avgdailyvol3m", "intradaymarketcap").
    pub field: String,
    /// Operator: eq, gt, gte, lt, lte, btwn.
    pub operator: String,
    /// Value(s) for the condition — a number, string, or `[min, max]` array for `btwn`.
    pub value: Json<serde_json::Value>,
}

/// Input for the `customScreener` root field.
#[derive(InputObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlCustomScreenerInput {
    /// Number of results (default: 25, max: 250).
    #[graphql(default = 25)]
    pub size: u32,
    /// Pagination offset (default: 0).
    #[graphql(default)]
    pub offset: u32,
    /// Sort direction ascending (default: false = descending).
    #[graphql(default)]
    pub sort_ascending: bool,
    /// Field to sort by (default: intradaymarketcap).
    pub sort_field: Option<String>,
    /// Quote type: EQUITY or MUTUALFUND (default: EQUITY).
    pub quote_type: Option<String>,
    /// Filter conditions.
    #[graphql(default)]
    pub filters: Vec<GqlFilterCondition>,
}
