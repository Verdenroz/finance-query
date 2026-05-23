#![no_main]
use arbitrary::Arbitrary;
use finance_query::indicators::{
    adx, balance_of_power, keltner_channels, mfi, parabolic_sar, stochastic, supertrend, vwap,
};
use libfuzzer_sys::fuzz_target;

// Exercises multi-series OHLCV indicators with shared structured input.
// ADX computes smoothed directional movement (division by TR), stochastic
// normalises %K/%D, supertrend builds ATR-based dynamic bands, vwap divides
// cumulative money-flow by cumulative volume, mfi uses typical-price ratios,
// parabolic_sar uses f64 acceleration/maximum params (negative/NaN/zero are
// interesting), keltner_channels layers EMA+ATR with a f64 multiplier, and
// balance_of_power divides by (high-low) — covering division-by-zero and
// mismatched-slice panics.
#[derive(Debug, Arbitrary)]
struct OhlcvInput {
    opens: Vec<f64>,
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    volumes: Vec<f64>,
    period: u8,
    k_slow: u8,
    d_period: u8,
    atr_period: u8,
    multiplier: f64,
    acceleration: f64,
    maximum: f64,
}

fuzz_target!(|input: OhlcvInput| {
    let _ = adx(&input.highs, &input.lows, &input.closes, input.period as usize);
    let _ = stochastic(
        &input.highs,
        &input.lows,
        &input.closes,
        input.period as usize,
        input.k_slow as usize,
        input.d_period as usize,
    );
    let _ = supertrend(
        &input.highs,
        &input.lows,
        &input.closes,
        input.period as usize,
        input.multiplier,
    );
    let _ = vwap(&input.highs, &input.lows, &input.closes, &input.volumes);
    let _ = mfi(
        &input.highs,
        &input.lows,
        &input.closes,
        &input.volumes,
        input.period as usize,
    );
    let _ = parabolic_sar(
        &input.highs,
        &input.lows,
        &input.closes,
        input.acceleration,
        input.maximum,
    );
    let _ = keltner_channels(
        &input.highs,
        &input.lows,
        &input.closes,
        input.period as usize,
        input.atr_period as usize,
        input.multiplier,
    );
    let smooth = if input.period > 0 {
        Some(input.period as usize)
    } else {
        None
    };
    let _ = balance_of_power(
        &input.opens,
        &input.highs,
        &input.lows,
        &input.closes,
        smooth,
    );
});
