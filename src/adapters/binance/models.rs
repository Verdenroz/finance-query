//! Binance public market-data wire types.

use serde::Deserialize;
use serde::de::{self, SeqAccess, Visitor};
use std::fmt;

/// `GET /api/v3/ticker/24hr` — rolling 24-hour statistics. Every numeric
/// field is a decimal string.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Ticker24hr {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub high_price: String,
    pub low_price: String,
    /// Volume denominated in the quote asset — the comparable figure to other
    /// providers' `volume_24h`, unlike `volume` (base asset).
    pub quote_volume: String,
}

/// The error body Binance returns for a rejected request. Only the message is
/// kept — the numeric `code` adds nothing the HTTP status and text do not.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BinanceError {
    #[serde(default)]
    pub msg: Option<String>,
}

/// One `GET /api/v3/klines` candle.
///
/// Binance sends these as heterogeneous JSON arrays whose trailing elements
/// have changed type over time (the last field has been both `0` and `"0"`).
/// Only the leading six are read, and the rest are skipped, so a future change
/// to the tail cannot break parsing.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Kline {
    /// Candle open time, milliseconds since epoch.
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    /// Base-asset volume.
    pub volume: f64,
}

impl<'de> Deserialize<'de> for Kline {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KlineVisitor;

        impl<'de> Visitor<'de> for KlineVisitor {
            type Value = Kline;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a Binance kline array")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Kline, A::Error> {
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

                let open_time: i64 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::missing_field("open_time"))?;
                let open = next_num(&mut seq, "open")?;
                let high = next_num(&mut seq, "high")?;
                let low = next_num(&mut seq, "low")?;
                let close = next_num(&mut seq, "close")?;
                let volume = next_num(&mut seq, "volume")?;
                while seq.next_element::<de::IgnoredAny>()?.is_some() {}

                Ok(Kline {
                    open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                })
            }
        }

        deserializer.deserialize_seq(KlineVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kline_parses_and_ignores_the_variable_tail() {
        // Verbatim shape from data-api.binance.vision, whose final element is
        // a string today and was a number historically.
        let raw = r#"[1785628800000,"62823.65000000","63796.33000000","62806.58000000",
            "63570.00000000","8387.01154000",1785715199999,"531238376.42127370",1286064,
            "4566.33112000","289282425.13673770","0"]"#;
        let k: Kline = serde_json::from_str(raw).unwrap();
        assert_eq!(k.open_time, 1785628800000);
        assert_eq!(k.open, 62823.65);
        assert_eq!(k.high, 63796.33);
        assert_eq!(k.low, 62806.58);
        assert_eq!(k.close, 63570.0);
        assert_eq!(k.volume, 8387.01154);
    }

    #[test]
    fn kline_tolerates_a_numeric_tail_element() {
        let raw = r#"[1,"1.0","2.0","0.5","1.5","10.0",2,"3.0",4,"5.0","6.0",0]"#;
        assert!(serde_json::from_str::<Kline>(raw).is_ok());
    }

    #[test]
    fn kline_rejects_a_truncated_array() {
        let raw = r#"[1,"1.0","2.0"]"#;
        assert!(serde_json::from_str::<Kline>(raw).is_err());
    }
}
