#![no_main]
use arbitrary::Arbitrary;
use finance_query::{Candle, patterns};
use libfuzzer_sys::fuzz_target;

// Exercises candlestick pattern detection with structured arbitrary input.
// The pattern engine computes body/wick ratios and compares consecutive bars
// (three-bar patterns) — NaN propagation, zero-body candles, and inverted
// high/low values are all realistic edge cases from provider data.
#[derive(Debug, Arbitrary)]
struct ArbitraryCandle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: i64,
}

fuzz_target!(|candles: Vec<ArbitraryCandle>| {
    let data: Vec<Candle> = candles
        .into_iter()
        .map(|c| {
            let mut candle = Candle::default();
            candle.open = c.open;
            candle.high = c.high;
            candle.low = c.low;
            candle.close = c.close;
            candle.volume = c.volume;
            candle
        })
        .collect();
    let _ = patterns(&data);
});
