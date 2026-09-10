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

use finance_query::{Capability, Provider, Providers, TreasuryAuctionQuery};

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

#[tokio::test]
#[ignore = "requires network access"]
async fn recent_bill_auctions_carry_their_results() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    let query = TreasuryAuctionQuery::new().security_type("Bill").limit(20);
    let auctions = providers
        .economic_catalog()
        .treasury_auctions(&query)
        .await
        .unwrap();

    assert_eq!(auctions.len(), 20);
    assert!(auctions.iter().all(|a| a.security_type == "Bill"));
    assert!(
        auctions
            .windows(2)
            .all(|w| w[0].auction_date >= w[1].auction_date),
        "auctions are not newest first"
    );
    // Bills price off the discount rate; the yield column stays empty for them.
    let settled = auctions
        .iter()
        .find(|a| a.bid_to_cover_ratio.is_some())
        .expect("at least one of the last 20 bill auctions has settled");
    assert!(settled.high_discnt_rate.is_some());
    assert!(settled.total_accepted.is_some_and(|v| v > 0.0));
    assert!(settled.cusip.len() == 9);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn auction_query_window_bounds_the_results() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    let query = TreasuryAuctionQuery::new().dates(Some("2025-01-01"), Some("2025-03-31"));
    let auctions = providers
        .economic_catalog()
        .treasury_auctions(&query)
        .await
        .unwrap();

    assert!(!auctions.is_empty());
    assert!(
        auctions
            .iter()
            .all(|a| a.auction_date.as_str() >= "2025-01-01"
                && a.auction_date.as_str() <= "2025-03-31"),
        "an auction fell outside the requested window"
    );
}

#[tokio::test]
#[ignore = "requires network access"]
async fn upcoming_auctions_are_the_newest_schedule() {
    let providers = Providers::builder()
        .route(Capability::ECONOMIC, [Provider::FiscalData])
        .build()
        .await
        .unwrap();

    let upcoming = providers
        .economic_catalog()
        .upcoming_auctions()
        .await
        .unwrap();

    assert!(!upcoming.is_empty());
    let published = &upcoming[0].record_date;
    assert!(
        upcoming.iter().all(|a| a.record_date == *published),
        "stale schedule rows leaked in alongside the newest one"
    );
    assert!(
        upcoming
            .windows(2)
            .all(|w| w[0].auction_date <= w[1].auction_date),
        "upcoming auctions are not soonest first"
    );
    assert!(
        upcoming
            .iter()
            .all(|a| a.auction_date >= a.announcement_date)
    );
}
