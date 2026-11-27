use finance_query::{Quote, format::Raw};

const SPY: &str = include_str!("../../benches/fixtures/quote_etf.json");

#[test]
fn quote_both_round_trips() {
    let quote: Quote = serde_json::from_str(SPY).unwrap();
    let json = serde_json::to_value(&quote).unwrap();
    let back: Quote = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(&back).unwrap(), json);
}

#[test]
fn quote_raw_round_trips() {
    let quote: Quote = serde_json::from_str(SPY).unwrap();
    let raw = quote.into_raw();
    let json = serde_json::to_value(&raw).unwrap();
    let back: Quote<Raw> = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(&back).unwrap(), json);
}

#[test]
fn sector_weightings_survive_round_trip() {
    let quote: Quote = serde_json::from_str(SPY).unwrap();
    let json = serde_json::to_string(&quote).unwrap();
    let back: Quote = serde_json::from_str(&json).unwrap();
    let sectors = back.top_holdings.unwrap().sector_weightings.unwrap();
    assert!(sectors.technology.is_some());
}
