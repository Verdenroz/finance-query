//! Compile and runtime tests for docs/library/providers/fiscaldata.md
//!
//! Requires the `fiscaldata` feature flag:
//!   cargo test --test doc_fiscaldata --features fiscaldata
//!   cargo test --test doc_fiscaldata --features fiscaldata -- --ignored   (network tests)
//!
//! Offline behaviour (series-id resolution, string-encoded numbers, the
//! `"null"` sentinel, pagination, API error mapping) is covered by the mock +
//! unit tests in `src/adapters/fiscaldata/mod.rs`.

#![cfg(feature = "fiscaldata")]

use finance_query::{Capability, Provider, Providers};

#[test]
fn fiscaldata_provider_id_round_trips() {
    assert_eq!(Provider::FiscalData.as_str(), "fiscaldata");
    assert_eq!(
        Provider::from_id_str("fiscaldata"),
        Some(Provider::FiscalData)
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn keyless_build_succeeds() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .expect("FiscalData needs no API key");
    let _ = providers.economic("DEBT_TO_PENNY");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn debt_to_penny_is_a_daily_dollar_series() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    let series = providers.economic("DEBT_TO_PENNY").series().await.unwrap();

    assert_eq!(series.units.as_deref(), Some("US Dollars"));
    assert_eq!(series.frequency.as_deref(), Some("Daily"));
    assert!(series.observations.len() > 1000);
    assert!(
        series
            .observations
            .windows(2)
            .all(|w| w[0].date <= w[1].date),
        "observations are not in chronological order"
    );
    assert!(
        series
            .observations
            .last()
            .and_then(|o| o.value)
            .is_some_and(|v| v > 1e13),
        "expected the public debt to be in the tens of trillions"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn passthrough_form_reaches_an_uncurated_column() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    let series = providers
        .economic("v2/accounting/od/debt_to_penny:debt_held_public_amt")
        .series()
        .await
        .unwrap();

    assert!(!series.observations.is_empty());
    // Passthrough series make no frequency claim.
    assert_eq!(series.frequency, None);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn unknown_series_id_is_rejected_before_any_request() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    assert!(providers.economic("NOT_A_SERIES").series().await.is_err());
}
