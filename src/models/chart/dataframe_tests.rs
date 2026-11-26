use super::StockBar;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn stock_bar_columns() {
    let row: StockBar = serde_json::from_value(json!({"timestamp_ms": 1, "open": 1.5, "high": 1.5, "low": 1.5, "close": 1.5, "volume": 1.5})).unwrap();
    let expected = [
        "timestamp_ms:i64",
        "open:f64",
        "high:f64",
        "low:f64",
        "close:f64",
        "volume:f64",
        "transactions:u64",
        "vwap:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <StockBar>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
