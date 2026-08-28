//! Pins the serialized form of [`Provider`] from outside the crate.
//!
//! `provider_id` is a public field on ten models and reaches REST and GraphQL
//! responses, so its serialized value is a wire contract. Nothing else in the
//! tree asserts it: `openapi.yaml` types the field as a bare string, so
//! `spec gen --check` cannot see the values move.
//!
//! These tests record what ships today, including the disagreement between
//! `Serialize` and `as_str`. A change to either must fail here first and be
//! re-recorded deliberately.

use finance_query::Provider;

fn json(provider: Provider) -> String {
    serde_json::to_string(&provider).expect("Provider serializes")
}

#[test]
fn serializes_as_the_variant_name() {
    assert_eq!(json(Provider::Yahoo), "\"Yahoo\"");
    assert_eq!(json(Provider::Edgar), "\"Edgar\"");
    assert_eq!(json(Provider::LocalExchange), "\"LocalExchange\"");
}

#[cfg(any(feature = "housetrades", feature = "senatetrades"))]
#[test]
fn congress_trades_serializes_unlike_its_id() {
    assert_eq!(json(Provider::CongressTrades), "\"CongressTrades\"");
    assert_eq!(Provider::CongressTrades.as_str(), "congresstrades");
}

#[cfg(feature = "defi")]
#[test]
fn defillama_serializes_unlike_its_id() {
    assert_eq!(json(Provider::DefiLlama), "\"DefiLlama\"");
    assert_eq!(Provider::DefiLlama.as_str(), "defillama");
}

#[test]
fn local_market_calendar_serializes_unlike_its_id() {
    assert_eq!(
        json(Provider::LocalMarketCalendar),
        "\"LocalMarketCalendar\""
    );
    assert_eq!(
        Provider::LocalMarketCalendar.as_str(),
        "local_market_calendar"
    );
}

#[test]
fn display_agrees_with_as_str_not_with_serialize() {
    assert_eq!(Provider::Yahoo.to_string(), Provider::Yahoo.as_str());
    assert_ne!(format!("\"{}\"", Provider::Yahoo), json(Provider::Yahoo));
}

/// The documented id is what `from_id_str` accepts, and it is the one form
/// `Deserialize` rejects.
#[test]
fn the_documented_id_does_not_deserialize() {
    assert_eq!(Provider::from_id_str("yahoo"), Some(Provider::Yahoo));
    assert!(serde_json::from_str::<Provider>("\"yahoo\"").is_err());
    assert!(serde_json::from_str::<Provider>("\"Yahoo\"").is_ok());
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
