#![no_main]
use libfuzzer_sys::fuzz_target;

// Verify that deserializing arbitrary bytes into FinancialStatement never panics.
// FinancialStatement flattens Yahoo Finance's nested timeseries response into a
// columnar map keyed by date — the transformation logic handles missing fields,
// mixed value types, and provider-specific camelCase variants that differ across
// income/balance/cashflow statements.
fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<finance_query::FinancialStatement>(data);
});
