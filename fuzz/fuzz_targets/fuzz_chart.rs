#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into a Chart never panics.
// Chart carries nested OHLCV candles, metadata, dividends, and splits —
// a wide surface for unexpected field shapes from provider APIs.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::Chart>(data);
});
