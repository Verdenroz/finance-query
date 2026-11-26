use super::Spark;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

fn spark(timestamps: &[i64], closes: &[f64]) -> Spark {
    serde_json::from_value(json!({
        "symbol": "AAPL",
        "meta": {"symbol": "AAPL"},
        "timestamps": timestamps,
        "closes": closes,
        "interval": "5m",
        "range": "1d",
    }))
    .unwrap()
}

#[test]
fn spark_columns() {
    let expected = [
        "symbol:str",
        "timestamp:i64",
        "close:f64",
        "interval:str",
        "range:str",
    ];
    let df = spark(&[1, 2], &[1.5, 2.5]).to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("interval").unwrap().null_count(), 0);

    let empty = Spark::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn spark_pads_missing_closes_with_nulls() {
    let df = spark(&[1, 2, 3], &[1.5]).to_dataframe().unwrap();
    assert_eq!(df.height(), 3);
    assert_eq!(df.column("close").unwrap().null_count(), 2);
}

#[test]
fn spark_meta_is_one_row() {
    let df = spark(&[1], &[1.5]).meta_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert!(df.width() > 0);
}
