#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into Options never panics.
// Options carries nested OptionChain → Contracts → OptionContract with
// Greeks (delta, gamma, theta, vega), bid/ask spreads, expiration dates,
// and deeply optional fields — a wide surface for shape mismatches from
// real provider API responses.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::Options>(data);
});
