//! Compile and runtime tests for docs/library/providers/openfigi.md
//!
//! Requires the `openfigi` feature flag:
//!   cargo test --test doc_openfigi --features openfigi
//!   cargo test --test doc_openfigi --features openfigi -- --ignored
//!
//! Offline behaviour (positional pairing, the capitalised `compositeFIGI`
//! keys, batch limits, error mapping) is covered by the mock + unit tests in
//! `src/adapters/openfigi/mod.rs`.

#![cfg(feature = "openfigi")]

use finance_query::openfigi::{self, SecurityIdKind, SecurityMapping};

/// Verifies the fields documented on the page exist with the stated types.
#[allow(dead_code)]
fn _verify_security_mapping_fields(m: SecurityMapping) {
    let _: String = m.figi;
    let _: Option<String> = m.ticker;
    let _: Option<String> = m.name;
    let _: Option<String> = m.exchange_code;
    let _: Option<String> = m.composite_figi;
    let _: Option<String> = m.share_class_figi;
    let _: Option<String> = m.security_type;
    let _: Option<String> = m.market_sector;
}

#[test]
fn id_kinds_expose_openfigis_type_names() {
    assert_eq!(SecurityIdKind::Cusip.to_string(), "ID_CUSIP");
    assert_eq!(SecurityIdKind::Isin.to_string(), "ID_ISIN");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn apples_cusip_resolves_to_a_us_listing() {
    let listings = openfigi::resolve_cusip("037833100").await.unwrap();

    assert!(!listings.is_empty());
    let composite = listings
        .iter()
        .find(|l| l.exchange_code.as_deref() == Some("US"))
        .expect("expected a US composite listing");
    assert_eq!(composite.ticker.as_deref(), Some("AAPL"));
    // The capitalised `compositeFIGI` key must actually be read.
    assert!(composite.composite_figi.is_some());
    assert!(composite.share_class_figi.is_some());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn isin_and_cusip_agree_on_the_same_security() {
    let by_cusip = openfigi::resolve_cusip("037833100").await.unwrap();
    let by_isin = openfigi::resolve_isin("US0378331005").await.unwrap();

    let cusip_composite = by_cusip.iter().find_map(|l| l.composite_figi.clone());
    let isin_composite = by_isin.iter().find_map(|l| l.composite_figi.clone());
    assert_eq!(cusip_composite, isin_composite);
}

/// The result must be positional, including for identifiers that match
/// nothing.
#[tokio::test]
#[ignore = "requires network access"]
async fn batch_results_pair_positionally_with_their_inputs() {
    let ids = ["037833100", "000000000", "594918104"];
    let results = openfigi::resolve_many(SecurityIdKind::Cusip, &ids)
        .await
        .unwrap();

    assert_eq!(results.len(), ids.len());
    assert!(!results[0].is_empty(), "Apple should resolve");
    // A well-formed CUSIP matching nothing is an empty list, not an error.
    assert!(results[1].is_empty(), "000000000 should match nothing");
    assert!(!results[2].is_empty(), "Microsoft should resolve");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn resolving_an_empty_batch_is_a_no_op() {
    assert!(
        openfigi::resolve_many(SecurityIdKind::Cusip, &[])
            .await
            .unwrap()
            .is_empty()
    );
}
