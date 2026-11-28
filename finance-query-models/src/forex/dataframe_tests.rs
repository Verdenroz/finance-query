use super::*;
use crate::dataframe_test_util::columns;

#[test]
fn forex_quote_columns() {
    let quotes: Vec<ForexQuote> = serde_json::from_value(serde_json::json!([
        {"symbol": "EURUSD", "bid": 1.08, "ask": 1.09}
    ]))
    .unwrap();
    let df = ForexQuote::vec_to_dataframe(&quotes).unwrap();

    assert_eq!(
        columns(&df),
        [
            "symbol:str",
            "base_currency:str",
            "quote_currency:str",
            "bid:f64",
            "ask:f64",
            "price:f64",
            "change:f64",
            "change_percent:f64",
            "timestamp:datetime[μs, UTC]",
        ]
    );
}
