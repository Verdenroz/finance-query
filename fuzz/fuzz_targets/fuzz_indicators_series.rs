#![no_main]
use arbitrary::Arbitrary;
use finance_query::indicators::{alma, bollinger_bands, ema, macd, rsi, sma, stochastic_rsi};
use libfuzzer_sys::fuzz_target;

// Exercises single-series price indicators with shared structured input.
// RSI uses Wilder smoothing (division), EMA uses recursive multiplication,
// Bollinger Bands computes std deviation, MACD nests EMAs with three independent
// periods, ALMA applies a Gaussian kernel (sigma=0 hits division), and
// stochastic_rsi chains RSI → stochastic (four independent periods) —
// together covering NaN/inf propagation and period=0/1 edge cases.
#[derive(Debug, Arbitrary)]
struct SeriesInput {
    prices: Vec<f64>,
    /// u8 keeps periods in a realistic range (0–255) without blowing up allocations.
    period: u8,
    fast: u8,
    slow: u8,
    signal: u8,
    std_dev_mult: f64,
    offset: f64,
    sigma: f64,
}

fuzz_target!(|input: SeriesInput| {
    let _ = rsi(&input.prices, input.period as usize);
    let _ = sma(&input.prices, input.period as usize);
    let _ = ema(&input.prices, input.period as usize);
    let _ = bollinger_bands(&input.prices, input.period as usize, input.std_dev_mult);
    let _ = macd(
        &input.prices,
        input.fast as usize,
        input.slow as usize,
        input.signal as usize,
    );
    let _ = alma(&input.prices, input.period as usize, input.offset, input.sigma);
    let _ = stochastic_rsi(
        &input.prices,
        input.period as usize,
        input.fast as usize,
        input.slow as usize,
        input.signal as usize,
    );
});
