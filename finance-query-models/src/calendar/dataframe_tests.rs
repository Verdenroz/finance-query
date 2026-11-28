use super::market::MarketCalendarEntry;
use super::*;
use crate::dataframe_test_util::columns;
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
        columns(&df),
        [
            "timestamp:datetime[μs, UTC]",
            "date:date",
            "symbol:str",
            "event_type:str"
        ]
    );
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
    assert_eq!((df.height(), df.width()), (0, 4));
}

#[test]
fn market_entry_frame() {
    let entries: Vec<MarketCalendarEntry> = serde_json::from_value(json!([
        {"symbol": "AAPL", "date": "2026-07-30", "detail": {"type": "ipo"}},
        {"symbol": null, "date": null, "detail": {"type": "split"}}
    ]))
    .unwrap();
    let df = MarketCalendarEntry::vec_to_dataframe(&entries).unwrap();

    assert_eq!(columns(&df), ["symbol:str", "date:date", "event_type:str"]);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("date").unwrap().null_count(), 1);

    let empty = MarketCalendarEntry::vec_to_dataframe(&[]).unwrap();
    assert_eq!((empty.height(), empty.width()), (0, 3));
}
