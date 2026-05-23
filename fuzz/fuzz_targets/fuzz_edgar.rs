#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into EDGAR types never panics.
// EdgarSubmissions uses custom deserializers that filter empty strings and
// strip empty vec elements — edge cases from the live SEC EDGAR API.
// CompanyFacts wraps FactsByTaxonomy (a HashMap of FactConcept) with nested
// FactUnit slices — deeply nested maps are a common source of unexpected shapes.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::EdgarSubmissions>(data);
    let _ = serde_json::from_slice::<finance_query::CompanyFacts>(data);
});
