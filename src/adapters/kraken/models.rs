//! Kraken public market-data wire types.
//!
//! Every Kraken response is `{"error": [...], "result": {...}}` with HTTP 200
//! even for a rejected request, so the error array — not the status code — is
//! what decides success.

use serde::Deserialize;
use std::collections::HashMap;

/// The envelope wrapping every `/0/public/*` response.
///
/// The explicit bound overrides serde's derived one: `#[serde(default)]` on
/// `Option<T>` would otherwise demand `T: Default`, which the payload types
/// have no reason to implement.
#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
pub(crate) struct KrakenEnvelope<T> {
    #[serde(default)]
    pub error: Vec<String>,
    /// Absent entirely when the request was rejected.
    #[serde(default)]
    pub result: Option<T>,
}

/// `GET /0/public/Ticker` — one entry per requested pair, keyed by Kraken's
/// own pair name (which may differ from the name requested).
pub(crate) type TickerResult = HashMap<String, KrakenTicker>;

/// Kraken packs each statistic into a short array. Fields documented as
/// `[today, last 24 hours]` are read at index 1 for the rolling figure.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct KrakenTicker {
    /// Last trade closed: `[price, lot volume]`.
    pub c: Vec<String>,
    /// Volume: `[today, last 24 hours]`.
    pub v: Vec<String>,
    /// Low: `[today, last 24 hours]`.
    pub l: Vec<String>,
    /// High: `[today, last 24 hours]`.
    pub h: Vec<String>,
    /// Today's opening price — Kraken publishes no price from 24 hours ago.
    pub o: String,
}

impl KrakenTicker {
    /// Read one of the packed arrays at `index`, parsed as a number.
    pub(crate) fn at(field: &[String], index: usize) -> Option<f64> {
        field.get(index)?.parse().ok()
    }
}

/// One `GET /0/public/OHLC` candle, sent as the heterogeneous array
/// `[time, open, high, low, close, vwap, volume, count]`.
///
/// Parsed field by field so a trailing addition to the array cannot break it,
/// and so the two fields the library has no use for (VWAP, trade count) need
/// not exist as dead struct members.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KrakenCandle {
    /// Candle open time, **seconds** since epoch — unlike Binance, already in
    /// the unit the canonical model uses.
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl<'de> Deserialize<'de> for KrakenCandle {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, SeqAccess, Visitor};

        struct CandleVisitor;

        impl<'de> Visitor<'de> for CandleVisitor {
            type Value = KrakenCandle;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a Kraken OHLC array")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<KrakenCandle, A::Error> {
                fn next_num<'de, A: SeqAccess<'de>>(
                    seq: &mut A,
                    field: &'static str,
                ) -> Result<f64, A::Error> {
                    let raw: String = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::missing_field(field))?;
                    raw.parse().map_err(|_| {
                        de::Error::invalid_value(de::Unexpected::Str(&raw), &"a decimal string")
                    })
                }

                let time: i64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::missing_field("time"))?;
                let open = next_num(&mut seq, "open")?;
                let high = next_num(&mut seq, "high")?;
                let low = next_num(&mut seq, "low")?;
                let close = next_num(&mut seq, "close")?;
                let _vwap = next_num(&mut seq, "vwap")?;
                let volume = next_num(&mut seq, "volume")?;
                while seq.next_element::<de::IgnoredAny>()?.is_some() {}

                Ok(KrakenCandle {
                    time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                })
            }
        }

        deserializer.deserialize_seq(CandleVisitor)
    }
}

/// `GET /0/public/OHLC` results mix the candle array with a `"last"` cursor
/// under the same map, so entries are typed as either.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum OhlcEntry {
    Candles(Vec<KrakenCandle>),
    /// The `"last"` key — a resume cursor, not a market. Its value is
    /// irrelevant; the variant exists only so the surrounding map parses.
    Cursor(serde::de::IgnoredAny),
}

pub(crate) type OhlcResult = HashMap<String, OhlcEntry>;
