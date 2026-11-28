use super::market::MarketCalendarEntry;
use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn calendar_event_frame() {
    let events: Vec<CalendarEvent> = serde_json::from_value(json!([
        {"timestamp": 1700000000, "date": "2023-11-14", "symbol": "AAPL",
         "event": {"type": "ex_dividend", "amount": 0.24}},
        {"timestamp": 1700086400, "date": "2023-11-15", "symbol": null,
         "event": {"type": "options_expiration", "contract_count": null}}
    ]))
    .unwrap();
    let df = CalendarEvent::vec_to_dataframe(&events).unwrap();

    assert_eq!(
        columns(&df)[..11],
        [
            "timestamp:datetime[μs, UTC]",
            "date:date",
            "symbol:str",
            "event_type:str",
            "eps_estimate_low:f64",
            "eps_estimate_avg:f64",
            "eps_estimate_high:f64",
            "revenue_estimate_avg:i64",
            "is_estimate:bool",
            "amount:f64",
            "contract_count:u64",
        ]
    );
    assert_eq!(
        df.column("amount").unwrap().f64().unwrap().get(0),
        Some(0.24)
    );
    assert_eq!(df.column("amount").unwrap().null_count(), 1);
    let types: Vec<_> = df
        .column("event_type")
        .unwrap()
        .str()
        .unwrap()
        .iter()
        .map(Option::unwrap)
        .collect();
    assert_eq!(types, ["ex_dividend", "options_expiration"]);
    assert_eq!(df.column("symbol").unwrap().null_count(), 1);
}

#[test]
fn calendar_event_empty_keeps_schema() {
    let df = CalendarEvent::vec_to_dataframe(&[]).unwrap();
    assert_eq!(
        (df.height(), df.width()),
        (0, 11 + 2 * usize::from(cfg!(feature = "fred")))
    );
}

#[test]
fn market_entry_frame() {
    let entries: Vec<MarketCalendarEntry> = serde_json::from_value(json!([
        {"symbol": "AAPL", "date": "2026-07-30", "detail": {"type": "ipo"}},
        {"symbol": null, "date": null, "detail": {"type": "split"}}
    ]))
    .unwrap();
    let df = MarketCalendarEntry::vec_to_dataframe(&entries).unwrap();

    assert_eq!(
        columns(&df)[..4],
        ["symbol:str", "date:date", "event_type:str", "eps:f64"]
    );
    assert_eq!(df.width(), 38);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("date").unwrap().null_count(), 1);

    let empty = MarketCalendarEntry::vec_to_dataframe(&[]).unwrap();
    assert_eq!((empty.height(), empty.width()), (0, 38));
}

#[test]
fn market_entry_frame_expands_the_payload() {
    let entries: Vec<MarketCalendarEntry> = serde_json::from_value(json!([
        {"symbol": "AAPL", "date": "2026-07-30", "detail": {
            "type": "earnings", "eps": 1.5, "time": "amc", "fiscal_date_ending": "2026-06-30",
            "company": "Apple", "estimate_count": 30}},
        {"symbol": "XYZ", "date": "2026-08-01", "detail": {
            "type": "ipo", "company": "Xyz", "shares": 10.0}},
        {"symbol": null, "date": "2026-08-02", "detail": {"type": "split", "numerator": 2.0}}
    ]))
    .unwrap();
    let df = MarketCalendarEntry::vec_to_dataframe(&entries).unwrap();
    let cols = columns(&df);

    for expected in [
        "fiscal_date_ending:date",
        "time_of_day:str",
        "estimate_count:u32",
        "shares:f64",
        "numerator:f64",
        "record_date:date",
    ] {
        assert!(cols.contains(&expected.to_string()), "{expected}");
    }
    let company = df.column("company").unwrap().str().unwrap();
    assert_eq!(
        company.iter().collect::<Vec<_>>(),
        [Some("Apple"), Some("Xyz"), None]
    );
    assert_eq!(df.column("time_of_day").unwrap().null_count(), 2);
}
