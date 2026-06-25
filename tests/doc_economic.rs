//! Compile and runtime tests for docs/library/economic.md
//!
//! Requires the `fred` feature flag:
//!   cargo test --test doc_economic --features fred
//!   cargo test --test doc_economic --features fred -- --ignored   (network tests)

#![cfg(feature = "fred")]

use finance_query::EconomicSeries;
use finance_query::fred::MacroObservation;

// ---------------------------------------------------------------------------
// Model fields — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies EconomicSeries fields documented in economic.md exist with correct types.
/// Never called; exists only for the compiler to type-check.
#[allow(dead_code)]
fn _verify_economic_series_fields(s: EconomicSeries) {
    let _: String = s.series_id;
    let _: Option<String> = s.title;
    let _: Option<String> = s.units;
    let _: Option<String> = s.frequency;
    let _: Vec<MacroObservation> = s.observations;
}

/// Verifies MacroObservation fields documented in economic.md.
#[allow(dead_code)]
fn _verify_macro_observation_fields(o: MacroObservation) {
    let _: String = o.date;
    let _: Option<f64> = o.value;
}

// ---------------------------------------------------------------------------
// Capability and Provider constants — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies Capability::ECONOMIC and Provider::Fred exist as documented.
#[allow(dead_code)]
fn _verify_routing_constants() {
    use finance_query::{Capability, Provider};
    let _: Capability = Capability::ECONOMIC;
    let _: Provider = Provider::Fred;
}

// ---------------------------------------------------------------------------
// FRED initialization — compile-time verification
// ---------------------------------------------------------------------------

/// Verifies fred::init compiles with the documented pattern from economic.md.
/// Never called; exists only to ensure the compile-time pattern is valid.
#[allow(dead_code)]
fn _verify_fred_init_pattern() {
    use finance_query::fred;

    // This block mirrors the pattern shown in economic.md lines 35-39
    // within the "FRED API Key" note section.
    // In practice, this would use: fred::init(std::env::var("FRED_API_KEY").unwrap())?;
    // For compile-only verification, we use a test key.
    let _ = fred::init("test-key");
}

// ---------------------------------------------------------------------------
// Network tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access (FRED_API_KEY)"]
async fn test_economic_series() {
    use finance_query::{Capability, Provider, Providers};
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, &[Provider::Fred])
        .build()
        .await
        .unwrap();
    let gdp = providers.economic("GDP");
    let series = gdp.series().await.unwrap();
    let _ = series;
}
