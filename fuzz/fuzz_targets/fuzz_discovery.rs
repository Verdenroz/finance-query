#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into discovery response types never panics.
// SearchResults carries mixed quotes + news from multiple provider APIs, and
// ScreenerResults wraps ScreenerQuote slices with pagination metadata — both
// are shaped by external providers and regularly change structure across endpoints.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::SearchResults>(data);
    let _ = serde_json::from_slice::<finance_query::ScreenerResults>(data);
});
