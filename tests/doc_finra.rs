//! Compile and runtime tests for docs/library/providers/finra.md
//!
//! Requires the `finra` feature flag:
//!   cargo test --test doc_finra --features finra
//!   cargo test --test doc_finra --features finra -- --ignored
//!
//! Offline behaviour (per-facility consolidation, the 204 empty-series case,
//! error mapping) is covered by the mock + unit tests in
//! `src/adapters/finra/mod.rs`.

#![cfg(feature = "finra")]

use finance_query::{Capability, Provider, Providers};

#[test]
fn finra_provider_id_round_trips() {
    assert_eq!(Provider::Finra.as_str(), "finra");
    assert_eq!(Provider::from_id_str("finra"), Some(Provider::Finra));
}

#[tokio::test]
#[ignore = "requires network access"]
async fn short_volume_series_is_daily_and_chronological() {
    let providers = Providers::builder()
        .route(Capability::FUNDAMENTALS, [Provider::Finra, Provider::Yahoo])
        .build()
        .await
        .expect("FINRA needs no API key");

    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let series = ticker.short_volume().await.unwrap();

    assert!(!series.is_empty());
    let dates: Vec<_> = series.iter().filter_map(|d| d.date.clone()).collect();
    assert_eq!(dates.len(), series.len(), "every row must carry a date");
    assert!(
        dates.windows(2).all(|w| w[0] < w[1]),
        "dates are not strictly ascending — per-facility rows may not have \
         been consolidated"
    );
    assert!(
        series
            .iter()
            .all(|d| match (d.short_volume, d.total_volume) {
                (Some(short), Some(total)) => short <= total,
                _ => true,
            }),
        "short volume exceeded total volume on some day"
    );
}

/// FINRA serves no financial statements, so a chain must fall through to a
/// provider that does.
#[tokio::test]
#[ignore = "requires network access"]
async fn financials_fall_through_past_finra() {
    let providers = Providers::builder()
        .route(Capability::FUNDAMENTALS, [Provider::Finra, Provider::Yahoo])
        .build()
        .await
        .unwrap();

    let ticker = providers.ticker("AAPL").build().await.unwrap();
    let statement = ticker
        .financials(
            finance_query::StatementType::Income,
            finance_query::Frequency::Annual,
        )
        .await
        .unwrap();
    assert!(!statement.statement.is_empty());
}
