//! Compile and runtime tests for docs/library/providers/worldbank.md
//!
//! Requires the `worldbank` feature flag:
//!   cargo test --test doc_worldbank --features worldbank
//!   cargo test --test doc_worldbank --features worldbank -- --ignored   (network tests)
//!
//! Offline behaviour of the series path (envelope parsing, error messages,
//! period normalisation, chronological ordering) is covered by the mock +
//! unit tests in `src/adapters/worldbank/mod.rs`.

#![cfg(feature = "worldbank")]

use finance_query::{Capability, Provider, Providers};

/// The provider id round-trips through the string form documented in the page.
#[test]
fn worldbank_provider_id_round_trips() {
    assert_eq!(Provider::WorldBank.as_str(), "worldbank");
    assert_eq!(
        Provider::from_id_str("worldbank"),
        Some(Provider::WorldBank)
    );
}

/// Routing `ECONOMIC` to World Bank must not need any key or env var, so
/// `build()` succeeds on a machine with nothing configured.
#[tokio::test]
#[ignore = "requires network access"]
async fn keyless_build_succeeds() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::WorldBank])
        .build()
        .await
        .expect("World Bank needs no API key");
    let _ = providers.economic("USA/NY.GDP.MKTP.CD");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn us_gdp_series_has_chronological_annual_observations() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::WorldBank])
        .build()
        .await
        .unwrap();

    let series = providers
        .economic("USA/NY.GDP.MKTP.CD")
        .series()
        .await
        .unwrap();

    assert_eq!(series.series_id, "USA/NY.GDP.MKTP.CD");
    assert_eq!(series.frequency.as_deref(), Some("Annual"));
    assert!(series.observations.len() > 50);
    assert!(
        series
            .observations
            .windows(2)
            .all(|w| w[0].date <= w[1].date),
        "observations are not in chronological order"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn bare_indicator_resolves_against_the_world_aggregate() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::WorldBank])
        .build()
        .await
        .unwrap();

    let series = providers.economic("SP.POP.TOTL").series().await.unwrap();
    assert!(
        series.title.as_deref().is_some_and(|t| t.contains("World")),
        "expected the world aggregate, got {:?}",
        series.title
    );
}
