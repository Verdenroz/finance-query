//! Compile and runtime tests for docs/library/providers/bls.md
//!
//! Requires the `bls` feature flag:
//!   cargo test --test doc_bls --features bls
//!   cargo test --test doc_bls --features bls -- --ignored   (network tests)
//!
//! Offline behaviour (period-code dating, the `"-"` sentinel, annual-aggregate
//! rows, tier selection, error mapping) is covered by the mock + unit tests in
//! `src/adapters/bls/mod.rs`.

#![cfg(feature = "bls")]

use finance_query::{Capability, Provider, Providers};

#[test]
fn bls_provider_id_round_trips() {
    assert_eq!(Provider::Bls.as_str(), "bls");
    assert_eq!(Provider::from_id_str("bls"), Some(Provider::Bls));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn keyless_build_succeeds() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::Bls])
        .build()
        .await
        .expect("BLS works without a key on the v1 route");
    let _ = providers.economic("CUUR0000SA0");
}

/// Burns one of the 25 keyless queries/day, so it stays behind `--ignored`.
#[tokio::test]
#[ignore = "requires network access"]
async fn cpi_series_is_monthly_and_chronological() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::Bls])
        .build()
        .await
        .unwrap();

    let series = providers.economic("CUUR0000SA0").series().await.unwrap();

    assert_eq!(series.series_id, "CUUR0000SA0");
    assert_eq!(series.frequency.as_deref(), Some("Monthly"));
    assert!(!series.observations.is_empty());
    assert!(
        series
            .observations
            .windows(2)
            .all(|w| w[0].date < w[1].date),
        "observations are not strictly chronological — an annual-aggregate \
         row may have leaked through"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn unknown_series_reports_not_found() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::Bls])
        .build()
        .await
        .unwrap();

    assert!(providers.economic("NOTASERIES").series().await.is_err());
}
