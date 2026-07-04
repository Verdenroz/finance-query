//! GraphQL types for predefined and custom stock/fund screeners.
//!
//! `ScreenerQuote`'s `FormattedValue<T>` fields follow the same convention as
//! `GqlQuote`: the resolver runs `ValueFormat::transform()` on the raw JSON
//! first, so these are exposed as opaque `Json<serde_json::Value>` here
//! (scalar in raw mode, string in fmt mode, full object in both mode).

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
    pub regular_market_price: Json<serde_json::Value>,
    pub regular_market_change: Json<serde_json::Value>,
    pub regular_market_change_percent: Json<serde_json::Value>,
    pub regular_market_open: Option<Json<serde_json::Value>>,
    pub regular_market_day_high: Option<Json<serde_json::Value>>,
    pub regular_market_day_low: Option<Json<serde_json::Value>>,
    pub regular_market_previous_close: Option<Json<serde_json::Value>>,
    pub regular_market_time: Option<Json<serde_json::Value>>,
    pub regular_market_volume: Option<Json<serde_json::Value>>,
    pub average_daily_volume3_month: Option<Json<serde_json::Value>>,
    pub average_daily_volume10_day: Option<Json<serde_json::Value>>,
    pub market_cap: Option<Json<serde_json::Value>>,
    pub shares_outstanding: Option<Json<serde_json::Value>>,
    pub fifty_two_week_high: Option<Json<serde_json::Value>>,
    pub fifty_two_week_low: Option<Json<serde_json::Value>>,
    pub fifty_two_week_change: Option<Json<serde_json::Value>>,
    pub fifty_two_week_change_percent: Option<Json<serde_json::Value>>,
    pub fifty_day_average: Option<Json<serde_json::Value>>,
    pub fifty_day_average_change: Option<Json<serde_json::Value>>,
    pub fifty_day_average_change_percent: Option<Json<serde_json::Value>>,
    pub two_hundred_day_average: Option<Json<serde_json::Value>>,
    pub two_hundred_day_average_change: Option<Json<serde_json::Value>>,
    pub two_hundred_day_average_change_percent: Option<Json<serde_json::Value>>,
    pub average_analyst_rating: Option<String>,
    #[graphql(name = "trailingPE")]
    #[serde(rename = "trailingPE")]
    pub trailing_pe: Option<Json<serde_json::Value>>,
    #[graphql(name = "forwardPE")]
    #[serde(rename = "forwardPE")]
    pub forward_pe: Option<Json<serde_json::Value>>,
    pub price_to_book: Option<Json<serde_json::Value>>,
    pub book_value: Option<Json<serde_json::Value>>,
    pub eps_trailing_twelve_months: Option<Json<serde_json::Value>>,
    pub eps_forward: Option<Json<serde_json::Value>>,
    pub eps_current_year: Option<Json<serde_json::Value>>,
    pub price_eps_current_year: Option<Json<serde_json::Value>>,
    pub dividend_yield: Option<Json<serde_json::Value>>,
    pub dividend_rate: Option<Json<serde_json::Value>>,
    pub dividend_date: Option<Json<serde_json::Value>>,
    pub trailing_annual_dividend_rate: Option<Json<serde_json::Value>>,
    pub trailing_annual_dividend_yield: Option<Json<serde_json::Value>>,
    pub bid: Option<Json<serde_json::Value>>,
    pub bid_size: Option<Json<serde_json::Value>>,
    pub ask: Option<Json<serde_json::Value>>,
    pub ask_size: Option<Json<serde_json::Value>>,
    pub post_market_price: Option<Json<serde_json::Value>>,
    pub post_market_change: Option<Json<serde_json::Value>>,
    pub post_market_change_percent: Option<Json<serde_json::Value>>,
    pub post_market_time: Option<Json<serde_json::Value>>,
    pub pre_market_price: Option<Json<serde_json::Value>>,
    pub pre_market_change: Option<Json<serde_json::Value>>,
    pub pre_market_change_percent: Option<Json<serde_json::Value>>,
    pub pre_market_time: Option<Json<serde_json::Value>>,
    pub earnings_timestamp: Option<Json<serde_json::Value>>,
    pub earnings_timestamp_start: Option<Json<serde_json::Value>>,
    pub earnings_timestamp_end: Option<Json<serde_json::Value>>,
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
