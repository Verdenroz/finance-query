//! GraphQL types for market reference/metadata data: hours, quote type,
//! currencies, exchanges.

use async_graphql::SimpleObject;
use serde::Deserialize;

/// Market open/close status for a single market.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMarketTime {
    pub id: String,
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub open: Option<String>,
    pub close: Option<String>,
    pub time: Option<String>,
    pub timezone: Option<String>,
    pub timezone_short: Option<String>,
    pub gmt_offset: Option<i32>,
    pub dst: Option<bool>,
}

/// Market hours for one or more markets.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlMarketHours {
    pub markets: Vec<GqlMarketTime>,
}

/// Quote type metadata for a symbol (exchange, timezone, identifiers).
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlQuoteTypeData {
    pub exchange: Option<String>,
    pub first_trade_date_epoch_utc: Option<i64>,
    pub gmt_off_set_milliseconds: Option<i64>,
    pub long_name: Option<String>,
    pub max_age: Option<i64>,
    pub message_board_id: Option<String>,
    pub quote_type: Option<String>,
    pub short_name: Option<String>,
    pub symbol: Option<String>,
    pub time_zone_full_name: Option<String>,
    pub time_zone_short_name: Option<String>,
    pub underlying_symbol: Option<String>,
    pub uuid: Option<String>,
}

/// A currency pair supported by Yahoo Finance.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlCurrency {
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub symbol: Option<String>,
    pub local_long_name: Option<String>,
}

/// A supported stock exchange with its symbol suffix and data provider.
#[derive(SimpleObject, Deserialize, Debug, Clone, Default)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase", default)]
pub struct GqlExchange {
    pub country: String,
    pub market: String,
    pub suffix: String,
    pub delay: String,
    pub data_provider: String,
}
