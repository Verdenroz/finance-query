//! Real-time pricing data from Yahoo Finance WebSocket
//!
//! This module contains the protobuf message definition for streaming price data.

use prost::Message;
use serde::{Deserialize, Serialize};

/// Quote type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
#[derive(Default)]
pub enum QuoteType {
    #[default]
    None,
    AltSymbol,
    Heartbeat,
    Equity,
    Index,
    MutualFund,
    MoneyMarket,
    Option,
    Currency,
    Warrant,
    Bond,
    Future,
    Etf,
    Commodity,
    EcnQuote,
    Cryptocurrency,
    Indicator,
    Industry,
}

impl From<i32> for QuoteType {
    fn from(value: i32) -> Self {
        match value {
            0 => QuoteType::None,
            5 => QuoteType::AltSymbol,
            7 => QuoteType::Heartbeat,
            8 => QuoteType::Equity,
            9 => QuoteType::Index,
            11 => QuoteType::MutualFund,
            12 => QuoteType::MoneyMarket,
            13 => QuoteType::Option,
            14 => QuoteType::Currency,
            15 => QuoteType::Warrant,
            17 => QuoteType::Bond,
            18 => QuoteType::Future,
            20 => QuoteType::Etf,
            23 => QuoteType::Commodity,
            28 => QuoteType::EcnQuote,
            41 => QuoteType::Cryptocurrency,
            42 => QuoteType::Indicator,
            1000 => QuoteType::Industry,
            _ => QuoteType::None,
        }
    }
}

/// Option type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
#[derive(Default)]
pub enum OptionType {
    #[default]
    Call,
    Put,
}

impl From<i32> for OptionType {
    fn from(value: i32) -> Self {
        match value {
            0 => OptionType::Call,
            1 => OptionType::Put,
            _ => OptionType::Call,
        }
    }
}

/// Market hours type enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
#[derive(Default)]
pub enum MarketHoursType {
    #[default]
    PreMarket,
    RegularMarket,
    PostMarket,
    ExtendedHoursMarket,
}

impl From<i32> for MarketHoursType {
    fn from(value: i32) -> Self {
        match value {
            0 => MarketHoursType::PreMarket,
            1 => MarketHoursType::RegularMarket,
            2 => MarketHoursType::PostMarket,
            3 => MarketHoursType::ExtendedHoursMarket,
            _ => MarketHoursType::PreMarket,
        }
    }
}

/// Internal protobuf struct for decoding Yahoo Finance WebSocket messages.
///
/// Not all fields are populated for every message - typically only fields that have
/// changed since the last update are included.
#[derive(Clone, PartialEq, Message)]
pub(crate) struct PricingData {
    /// Ticker symbol (e.g., "AAPL", "NVDA")
    #[prost(string, tag = "1")]
    pub id: String,

    /// Current price
    #[prost(float, tag = "2")]
    pub price: f32,

    /// Unix timestamp in milliseconds
    #[prost(sint64, tag = "3")]
    pub time: i64,

    /// Currency code (e.g., "USD")
    #[prost(string, tag = "4")]
    pub currency: String,

    /// Exchange code (e.g., "NMS", "NYQ")
    #[prost(string, tag = "5")]
    pub exchange: String,

    /// Quote type
    #[prost(enumeration = "QuoteTypeProto", tag = "6")]
    pub quote_type: i32,

    /// Market hours indicator
    #[prost(enumeration = "MarketHoursTypeProto", tag = "7")]
    pub market_hours: i32,

    /// Percent change from previous close
    #[prost(float, tag = "8")]
    pub change_percent: f32,

    /// Day's trading volume
    #[prost(sint64, tag = "9")]
    pub day_volume: i64,

    /// Day's high price
    #[prost(float, tag = "10")]
    pub day_high: f32,

    /// Day's low price
    #[prost(float, tag = "11")]
    pub day_low: f32,

    /// Price change from previous close
    #[prost(float, tag = "12")]
    pub change: f32,

    /// Short name/description
    #[prost(string, tag = "13")]
    pub short_name: String,

    /// Options expiration date (Unix timestamp)
    #[prost(sint64, tag = "14")]
    pub expire_date: i64,

    /// Opening price
    #[prost(float, tag = "15")]
    pub open_price: f32,

    /// Previous close price
    #[prost(float, tag = "16")]
    pub previous_close: f32,

    /// Strike price (for options)
    #[prost(float, tag = "17")]
    pub strike_price: f32,

    /// Underlying symbol (for options/derivatives)
    #[prost(string, tag = "18")]
    pub underlying_symbol: String,

    /// Open interest (for options)
    #[prost(sint64, tag = "19")]
    pub open_interest: i64,

    /// Options type (call/put)
    #[prost(enumeration = "OptionTypeProto", tag = "20")]
    pub options_type: i32,

    /// Mini option indicator
    #[prost(sint64, tag = "21")]
    pub mini_option: i64,

    /// Last trade size
    #[prost(sint64, tag = "22")]
    pub last_size: i64,

    /// Bid price
    #[prost(float, tag = "23")]
    pub bid: f32,

    /// Bid size
    #[prost(sint64, tag = "24")]
    pub bid_size: i64,

    /// Ask price
    #[prost(float, tag = "25")]
    pub ask: f32,

    /// Ask size
    #[prost(sint64, tag = "26")]
    pub ask_size: i64,

    /// Price hint (decimal places)
    #[prost(sint64, tag = "27")]
    pub price_hint: i64,

    /// 24-hour volume (for crypto)
    #[prost(sint64, tag = "28")]
    pub vol_24hr: i64,

    /// Volume across all currencies (for crypto)
    #[prost(sint64, tag = "29")]
    pub vol_all_currencies: i64,

    /// From currency (for forex/crypto)
    #[prost(string, tag = "30")]
    pub from_currency: String,

    /// Last market (for crypto)
    #[prost(string, tag = "31")]
    pub last_market: String,

    /// Circulating supply (for crypto)
    #[prost(double, tag = "32")]
    pub circulating_supply: f64,

    /// Market capitalization (for crypto)
    #[prost(double, tag = "33")]
    pub market_cap: f64,
}

/// Protobuf enum for quote type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum QuoteTypeProto {
    None = 0,
    AltSymbol = 5,
    Heartbeat = 7,
    Equity = 8,
    Index = 9,
    MutualFund = 11,
    MoneyMarket = 12,
    Option = 13,
    Currency = 14,
    Warrant = 15,
    Bond = 17,
    Future = 18,
    Etf = 20,
    Commodity = 23,
    EcnQuote = 28,
    Cryptocurrency = 41,
    Indicator = 42,
    Industry = 1000,
}

/// Protobuf enum for option type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum OptionTypeProto {
    Call = 0,
    Put = 1,
}

/// Protobuf enum for market hours type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
#[allow(clippy::enum_variant_names)]
pub enum MarketHoursTypeProto {
    PreMarket = 0,
    RegularMarket = 1,
    PostMarket = 2,
    ExtendedHoursMarket = 3,
}

impl PricingData {
    /// Decode from base64-encoded protobuf message
    pub(crate) fn from_base64(encoded: &str) -> Result<Self, PricingDecodeError> {
        let bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|e| PricingDecodeError::Base64(e.to_string()))?;

        Self::decode(&bytes[..]).map_err(|e| PricingDecodeError::Protobuf(e.to_string()))
    }
}

/// Real-time price update from Yahoo Finance WebSocket.
///
/// This is the user-facing struct with properly typed enum fields
/// that serialize to readable strings like `"EQUITY"` or `"CRYPTOCURRENCY"`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct PriceUpdate {
    pub id: String,
    pub price: f32,
    pub time: i64,
    pub currency: String,
    pub exchange: String,
    pub quote_type: QuoteType,
    pub market_hours: MarketHoursType,
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
    pub options_type: OptionType,
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

impl From<PricingData> for PriceUpdate {
    fn from(data: PricingData) -> Self {
        Self {
            id: data.id,
            price: data.price,
            time: data.time,
            currency: data.currency,
            exchange: data.exchange,
            quote_type: QuoteType::from(data.quote_type),
            market_hours: MarketHoursType::from(data.market_hours),
            change_percent: data.change_percent,
            day_volume: data.day_volume,
            day_high: data.day_high,
            day_low: data.day_low,
            change: data.change,
            short_name: data.short_name,
            expire_date: data.expire_date,
            open_price: data.open_price,
            previous_close: data.previous_close,
            strike_price: data.strike_price,
            underlying_symbol: data.underlying_symbol,
            open_interest: data.open_interest,
            options_type: OptionType::from(data.options_type),
            mini_option: data.mini_option,
            last_size: data.last_size,
            bid: data.bid,
            bid_size: data.bid_size,
            ask: data.ask,
            ask_size: data.ask_size,
            price_hint: data.price_hint,
            vol_24hr: data.vol_24hr,
            vol_all_currencies: data.vol_all_currencies,
            from_currency: data.from_currency,
            last_market: data.last_market,
            circulating_supply: data.circulating_supply,
            market_cap: data.market_cap,
        }
    }
}

/// Error decoding pricing data
#[derive(Debug, Clone)]
pub(crate) enum PricingDecodeError {
    /// Base64 decoding failed
    Base64(String),
    /// Protobuf decoding failed
    Protobuf(String),
}

impl std::fmt::Display for PricingDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PricingDecodeError::Base64(e) => write!(f, "Base64 decode error: {}", e),
            PricingDecodeError::Protobuf(e) => write!(f, "Protobuf decode error: {}", e),
        }
    }
}

impl std::error::Error for PricingDecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_type_from_i32() {
        assert_eq!(QuoteType::from(8), QuoteType::Equity);
        assert_eq!(QuoteType::from(41), QuoteType::Cryptocurrency);
        assert_eq!(QuoteType::from(20), QuoteType::Etf);
        assert_eq!(QuoteType::from(999), QuoteType::None);
    }

    #[test]
    fn test_market_hours_from_i32() {
        assert_eq!(MarketHoursType::from(0), MarketHoursType::PreMarket);
        assert_eq!(MarketHoursType::from(1), MarketHoursType::RegularMarket);
        assert_eq!(MarketHoursType::from(2), MarketHoursType::PostMarket);
    }
}
