//! Real-time pricing data from Yahoo Finance WebSocket
//!
//! This module contains the protobuf message definition for streaming price data.

use prost::Message;
use serde::{Deserialize, Serialize};

/// Quote type enumeration
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
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

impl std::fmt::Display for QuoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "NONE",
            Self::AltSymbol => "ALT_SYMBOL",
            Self::Heartbeat => "HEARTBEAT",
            Self::Equity => "EQUITY",
            Self::Index => "INDEX",
            Self::MutualFund => "MUTUAL_FUND",
            Self::MoneyMarket => "MONEY_MARKET",
            Self::Option => "OPTION",
            Self::Currency => "CURRENCY",
            Self::Warrant => "WARRANT",
            Self::Bond => "BOND",
            Self::Future => "FUTURE",
            Self::Etf => "ETF",
            Self::Commodity => "COMMODITY",
            Self::EcnQuote => "ECN_QUOTE",
            Self::Cryptocurrency => "CRYPTOCURRENCY",
            Self::Indicator => "INDICATOR",
            Self::Industry => "INDUSTRY",
        })
    }
}

/// Option type enumeration
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
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

impl std::fmt::Display for OptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Call => "CALL",
            Self::Put => "PUT",
        })
    }
}

/// Market hours type enumeration
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
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

impl std::fmt::Display for MarketHoursType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PreMarket => "PRE_MARKET",
            Self::RegularMarket => "REGULAR_MARKET",
            Self::PostMarket => "POST_MARKET",
            Self::ExtendedHoursMarket => "EXTENDED_HOURS_MARKET",
        })
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
    #[prost(sint64, tag = "20")]
    pub options_type: i64,

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "dataframe", derive(crate::ToDataFrame))]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct PriceUpdate {
    pub id: String,
    pub price: f64,
    pub time: i64,
    pub currency: String,
    pub exchange: String,
    pub quote_type: QuoteType,
    pub market_hours: MarketHoursType,
    pub change_percent: f64,
    pub day_volume: i64,
    pub day_high: f64,
    pub day_low: f64,
    pub change: f64,
    pub short_name: String,
    pub expire_date: i64,
    pub open_price: f64,
    pub previous_close: f64,
    pub strike_price: f64,
    pub underlying_symbol: String,
    pub open_interest: i64,
    pub options_type: OptionType,
    pub mini_option: i64,
    pub last_size: i64,
    pub bid: f64,
    pub bid_size: i64,
    pub ask: f64,
    pub ask_size: i64,
    pub price_hint: i64,
    pub vol_24hr: i64,
    pub vol_all_currencies: i64,
    pub from_currency: String,
    pub last_market: String,
    pub circulating_supply: f64,
    pub market_cap: f64,
}

const ROUND: f64 = 6_755_399_441_055_744.0;
const SCALES: [f64; 11] = [1.0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10];

/// Widens by shortest decimal form so `189.98f32` becomes `189.98f64`, not
/// `189.97999572753906`.
fn widen(value: f32) -> f64 {
    widen_scaled(value).unwrap_or_else(|| widen_shortest(value))
}

fn hint_scale(hint: i64) -> Option<f64> {
    SCALES.get(usize::try_from(hint).ok()?).copied()
}

/// `hint_scale` is 10^`price_hint`, Yahoo's quote decimal count. Below 2^20
/// scaled units an f32 rounds to at most one integer there, so that candidate
/// is the shortest decimal whenever it round-trips.
fn widen_price(value: f32, hint_scale: Option<f64>) -> f64 {
    let Some(scale) = hint_scale else {
        return widen(value);
    };
    let wide = f64::from(value);
    let scaled = wide * scale;
    if scaled.is_nan() || scaled.abs() >= 1_048_576.0 {
        return widen(value);
    }
    let candidate = ((scaled + ROUND) - ROUND) / scale;
    if candidate as f32 == value {
        candidate.copysign(wide)
    } else {
        widen(value)
    }
}

/// Below 2^24 the shortest decimal has a fractional digit count, so the first
/// scale whose rounded quotient round-trips is it. Adding and subtracting
/// 1.5 * 2^52 rounds half to even, as the shortest form does on exact ties.
fn widen_scaled(value: f32) -> Option<f64> {
    if value.is_nan() || value.abs() >= 16_777_216.0 {
        return None;
    }
    let wide = f64::from(value);
    SCALES.iter().find_map(|&scale| {
        let candidate = ((wide * scale + ROUND) - ROUND) / scale;
        (candidate as f32 == value).then(|| candidate.copysign(wide))
    })
}

fn widen_shortest(value: f32) -> f64 {
    if !value.is_finite() {
        return f64::from(value);
    }
    ryu::Buffer::new()
        .format_finite(value)
        .parse()
        .unwrap_or_else(|_| f64::from(value))
}

impl From<PricingData> for PriceUpdate {
    fn from(data: PricingData) -> Self {
        let hint = hint_scale(data.price_hint);
        Self {
            id: data.id,
            price: widen_price(data.price, hint),
            time: data.time,
            currency: data.currency,
            exchange: data.exchange,
            quote_type: QuoteType::from(data.quote_type),
            market_hours: MarketHoursType::from(data.market_hours),
            change_percent: widen(data.change_percent),
            day_volume: data.day_volume,
            day_high: widen_price(data.day_high, hint),
            day_low: widen_price(data.day_low, hint),
            change: widen_price(data.change, hint),
            short_name: data.short_name,
            expire_date: data.expire_date,
            open_price: widen_price(data.open_price, hint),
            previous_close: widen_price(data.previous_close, hint),
            strike_price: widen_price(data.strike_price, hint),
            underlying_symbol: data.underlying_symbol,
            open_interest: data.open_interest,
            options_type: OptionType::from(data.options_type as i32),
            mini_option: data.mini_option,
            last_size: data.last_size,
            bid: widen_price(data.bid, hint),
            bid_size: data.bid_size,
            ask: widen_price(data.ask, hint),
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

    #[test]
    fn widening_keeps_the_shortest_decimal_form() {
        let update = PriceUpdate::from(PricingData {
            price: 189.98,
            bid: f32::NAN,
            ask: f32::INFINITY,
            ..Default::default()
        });
        assert_eq!(update.price, 189.98_f64);
        assert!(update.bid.is_nan());
        assert_eq!(update.ask, f64::INFINITY);
    }

    #[test]
    fn scaled_widening_matches_shortest_form() {
        let strided = (0..=u32::MAX).step_by(4099);
        let cents = (1..=1_000_000u32).map(|i| (i as f32 * 1e-2).to_bits());
        let prices = (1..=1_000_000u32).map(|i| (i as f32 * 1e-4).to_bits());
        let edges = [
            0,
            1,
            0x007f_ffff,
            f32::NAN.to_bits(),
            f32::INFINITY.to_bits(),
        ];
        for bits in strided.chain(cents).chain(prices).chain(edges) {
            for bits in [bits, bits | 1 << 31] {
                let value = f32::from_bits(bits);
                let shortest = widen_shortest(value).to_bits();
                assert_eq!(widen(value).to_bits(), shortest, "{bits:#x}");
                for hint in [-1, 0, 2, 4, 10, 11] {
                    assert_eq!(
                        widen_price(value, hint_scale(hint)).to_bits(),
                        shortest,
                        "{bits:#x} hint {hint}"
                    );
                }
            }
        }
    }

    fn sample() -> PricingData {
        PricingData {
            id: "AAPL".into(),
            price: 189.5,
            time: 1_700_000_000_000,
            currency: "USD".into(),
            exchange: "NMS".into(),
            quote_type: QuoteTypeProto::Equity as i32,
            market_hours: MarketHoursTypeProto::PostMarket as i32,
            change_percent: -0.25,
            day_volume: 51_234_567,
            bid: 189.25,
            ask: 189.75,
            options_type: 1,
            market_cap: 2.9e12,
            ..Default::default()
        }
    }

    fn encode_base64(data: &PricingData) -> String {
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            data.encode_to_vec(),
        )
    }

    #[test]
    fn from_base64_round_trips_every_set_field() {
        let data = sample();
        assert_eq!(
            PricingData::from_base64(&encode_base64(&data)).unwrap(),
            data
        );
    }

    #[test]
    fn price_update_maps_enums_and_fields() {
        let update = PriceUpdate::from(sample());
        assert_eq!(update.id, "AAPL");
        assert_eq!(update.price, 189.5);
        assert_eq!(update.quote_type, QuoteType::Equity);
        assert_eq!(update.market_hours, MarketHoursType::PostMarket);
        assert_eq!(update.options_type, OptionType::Put);
        assert_eq!(update.day_volume, 51_234_567);
        assert_eq!(update.market_cap, 2.9e12);
    }

    #[test]
    fn from_base64_rejects_invalid_base64() {
        assert!(matches!(
            PricingData::from_base64("not base64!"),
            Err(PricingDecodeError::Base64(_))
        ));
    }

    #[test]
    fn from_base64_rejects_invalid_protobuf() {
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            [0x0a, 0xff, 0x01],
        );
        assert!(matches!(
            PricingData::from_base64(&encoded),
            Err(PricingDecodeError::Protobuf(_))
        ));
    }
}
