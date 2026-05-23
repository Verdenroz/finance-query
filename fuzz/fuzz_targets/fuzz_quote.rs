#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into a Quote never panics.
// Malformed / unexpected JSON must return Err, not abort or unwind.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::Quote>(data);
});
