#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

// Structured fuzz input: arbitrary OHLCV slices and period.
// Exercises ATR arithmetic with NaN, infinity, negatives, mismatched
// slice lengths, and zero/huge periods — none should panic.
#[derive(Debug, Arbitrary)]
struct AtrInput {
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    /// u8 keeps period in a realistic range (0–255) without blowing up allocations.
    period: u8,
}

fuzz_target!(|input: AtrInput| {
    let _ = finance_query::atr(
        &input.highs,
        &input.lows,
        &input.closes,
        input.period as usize,
    );
});
