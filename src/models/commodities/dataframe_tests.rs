use super::*;
use crate::models::dataframe_test_util::columns;

#[test]
fn commodity_quote_columns() {
    let quotes: Vec<CommodityQuote> = serde_json::from_value(serde_json::json!([
        {"symbol": "GCUSD", "price": 2000.5, "timestamp": 1700000000}
    ]))
    .unwrap();
    let df = CommodityQuote::vec_to_dataframe(&quotes).unwrap();

    assert_eq!(
        columns(&df),
        [
            "symbol:str",
            "name:str",
            "unit:str",
            "price:f64",
            "change:f64",
            "change_percent:f64",
            "timestamp:i64",
        ]
    );
    assert_eq!(df.height(), 1);
}

#[test]
fn commodity_quote_empty_keeps_schema() {
    let df = CommodityQuote::vec_to_dataframe(&[]).unwrap();
    assert_eq!(df.height(), 0);
    assert_eq!(df.width(), 7);
}
