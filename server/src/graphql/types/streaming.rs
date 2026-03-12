//! GraphQL type for real-time price streaming updates.

use async_graphql::SimpleObject;
use finance_query::streaming::PriceUpdate;
use serde::Deserialize;

/// A real-time price update from the streaming WebSocket, mirroring `PriceUpdate`.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct GqlPriceUpdate {
    pub id: String,
    pub price: f32,
    pub time: i64,
    pub currency: String,
    pub exchange: String,
    pub quote_type: String,
    pub market_hours: String,
    pub change_percent: f32,
    pub day_volume: i64,
    pub day_high: f32,
    pub day_low: f32,
    pub change: f32,
    pub short_name: String,
    pub expire_date: i64,
    pub open_price: f32,
    pub previous_close: f32,
    pub strike_price: f32,
    pub underlying_symbol: String,
    pub open_interest: i64,
    pub options_type: String,
    pub mini_option: i64,
    pub last_size: i64,
    pub bid: f32,
    pub bid_size: i64,
    pub ask: f32,
    pub ask_size: i64,
    pub price_hint: i64,
    pub vol_24hr: i64,
    pub vol_all_currencies: i64,
    pub from_currency: String,
    pub last_market: String,
    pub circulating_supply: f64,
    pub market_cap: f64,
}

impl From<PriceUpdate> for GqlPriceUpdate {
    fn from(u: PriceUpdate) -> Self {
        Self {
            id: u.id,
            price: u.price,
            time: u.time,
            currency: u.currency,
            exchange: u.exchange,
            quote_type: format!("{:?}", u.quote_type),
            market_hours: format!("{:?}", u.market_hours),
            change_percent: u.change_percent,
            day_volume: u.day_volume,
            day_high: u.day_high,
            day_low: u.day_low,
            change: u.change,
            short_name: u.short_name,
            expire_date: u.expire_date,
            open_price: u.open_price,
            previous_close: u.previous_close,
            strike_price: u.strike_price,
            underlying_symbol: u.underlying_symbol,
            open_interest: u.open_interest,
            options_type: format!("{:?}", u.options_type),
            mini_option: u.mini_option,
            last_size: u.last_size,
            bid: u.bid,
            bid_size: u.bid_size,
            ask: u.ask,
            ask_size: u.ask_size,
            price_hint: u.price_hint,
            vol_24hr: u.vol_24hr,
            vol_all_currencies: u.vol_all_currencies,
            from_currency: u.from_currency,
            last_market: u.last_market,
            circulating_supply: u.circulating_supply,
            market_cap: u.market_cap,
        }
    }
}
