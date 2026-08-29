//! Pins the serialized form of [`Provider`] from outside the crate.
//!
//! `provider_id` is a public field on ten models and reaches REST and GraphQL
//! responses, so its serialized value is a wire contract. Nothing else in the
//! tree asserts it: `openapi.yaml` types the field as a bare string, so
//! `spec gen --check` cannot see the values move.
//!
//! One id per provider, used by `Serialize`, `Deserialize`, `as_str`,
//! `Display`, and `from_id_str` alike. A change to any of them must fail here
//! first and be re-recorded deliberately.

use finance_query::Provider;

fn json(provider: Provider) -> String {
    serde_json::to_string(&provider).expect("Provider serializes")
}

#[test]
fn serializes_as_the_provider_id() {
    assert_eq!(json(Provider::Yahoo), "\"yahoo\"");
    assert_eq!(json(Provider::Edgar), "\"edgar\"");
    assert_eq!(
        json(Provider::LocalMarketCalendar),
        "\"local_market_calendar\""
    );
    assert_eq!(json(Provider::LocalExchange), "\"local_exchange\"");
}

#[cfg(any(feature = "housetrades", feature = "senatetrades"))]
#[test]
fn congress_trades_serializes_as_its_id() {
    assert_eq!(json(Provider::CongressTrades), "\"congresstrades\"");
    assert_eq!(Provider::CongressTrades.as_str(), "congresstrades");
}

#[cfg(feature = "defi")]
#[test]
fn defillama_serializes_as_its_id() {
    assert_eq!(json(Provider::DefiLlama), "\"defillama\"");
    assert_eq!(Provider::DefiLlama.as_str(), "defillama");
}

#[test]
fn display_and_serialize_agree_with_as_str() {
    for provider in [Provider::Yahoo, Provider::Edgar, Provider::LocalExchange] {
        assert_eq!(provider.to_string(), provider.as_str());
        assert_eq!(json(provider), format!("\"{}\"", provider.as_str()));
    }
}

#[test]
fn the_documented_id_deserializes() {
    assert_eq!(Provider::from_id_str("yahoo"), Some(Provider::Yahoo));
    assert_eq!(
        serde_json::from_str::<Provider>("\"yahoo\"").unwrap(),
        Provider::Yahoo
    );
}

/// `serde_json::from_str` on an unescaped literal is the one input shape that
/// hands the deserializer a borrowed `&str`. Deserializing through anything
/// else must work too.
#[test]
fn deserializes_from_inputs_that_cannot_borrow() {
    assert_eq!(
        serde_json::from_reader::<_, Provider>(std::io::Cursor::new(b"\"yahoo\"")).unwrap(),
        Provider::Yahoo
    );
    assert_eq!(
        serde_json::from_str::<Provider>("\"yah\\u006fo\"").unwrap(),
        Provider::Yahoo
    );
    assert_eq!(
        serde_json::from_slice::<Provider>(b"\"yahoo\"").unwrap(),
        Provider::Yahoo
    );
}

#[test]
fn the_old_variant_name_no_longer_deserializes() {
    assert!(serde_json::from_str::<Provider>("\"Yahoo\"").is_err());
    assert!(serde_json::from_str::<Provider>("\"LocalExchange\"").is_err());
}

#[test]
fn serialized_form_round_trips_with_itself() {
    for provider in [
        Provider::Yahoo,
        Provider::Edgar,
        Provider::LocalMarketCalendar,
        Provider::LocalExchange,
    ] {
        let encoded = json(provider);
        let decoded: Provider = serde_json::from_str(&encoded).expect("round trips");
        assert_eq!(decoded, provider);
    }
}

/// Interning is process-wide and append-only, so these ids must not appear in
/// any other test: the "unknown before registration" assertion would then
/// depend on which test ran first.
#[test]
fn a_custom_provider_round_trips_once_registered() {
    assert_eq!(Provider::from_id_str("wire-format-round-trip"), None);
    assert!(serde_json::from_str::<Provider>("\"wire-format-round-trip\"").is_err());

    let custom = Provider::custom("wire-format-round-trip");
    assert_eq!(json(custom), "\"wire-format-round-trip\"");
    assert_eq!(custom.as_str(), "wire-format-round-trip");
    assert_eq!(custom.to_string(), "wire-format-round-trip");
    assert_eq!(
        Provider::from_id_str("wire-format-round-trip"),
        Some(custom)
    );
    assert_eq!(
        serde_json::from_str::<Provider>("\"wire-format-round-trip\"").unwrap(),
        custom
    );
}

#[test]
fn interning_the_same_id_twice_yields_the_same_value() {
    assert_eq!(
        Provider::custom("wire-format-same"),
        Provider::custom("wire-format-same")
    );
    assert_ne!(
        Provider::custom("wire-format-same"),
        Provider::custom("wire-format-other")
    );
}

#[test]
fn every_id_is_accepted_by_from_id_str() {
    for id in ["yahoo", "edgar", "local_market_calendar", "local_exchange"] {
        assert_eq!(
            Provider::from_id_str(id).map(Provider::as_str),
            Some(id),
            "{id} should round-trip through from_id_str"
        );
    }
    assert_eq!(Provider::from_id_str("not-a-provider"), None);
}
