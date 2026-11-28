use super::Spark;
use crate::dataframe_test_util::columns;
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
        "timestamp:datetime[μs, UTC]",
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
fn spark_rows_pair_each_timestamp_with_its_close() {
    let result = serde_json::from_value(json!({
        "symbol": "AAPL",
        "response": [{
            "meta": {"symbol": "AAPL"},
            "timestamp": [1, 2, 3],
            "indicators": {"quote": [{"close": [1.5, null, 3.5]}]}
        }]
    }))
    .unwrap();
    let spark = Spark::from_response(&result, None, None).unwrap();
    let df = spark.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    let timestamps = &df.column("timestamp").unwrap().datetime().unwrap().phys;
    let closes = df.column("close").unwrap().f64().unwrap();
    assert_eq!(
        timestamps.iter().collect::<Vec<_>>(),
        [Some(1_000_000), Some(3_000_000)]
    );
    assert_eq!(closes.iter().collect::<Vec<_>>(), [Some(1.5), Some(3.5)]);
}

#[test]
fn spark_without_timestamps_keeps_its_closes() {
    let result = serde_json::from_value(json!({
        "symbol": "AAPL",
        "response": [{
            "meta": {"symbol": "AAPL"},
            "indicators": {"quote": [{"close": [1.0, null, 2.0]}]}
        }]
    }))
    .unwrap();
    let spark = Spark::from_response(&result, None, None).unwrap();
    assert_eq!(spark.closes, [1.0, 2.0]);
    assert_eq!(spark.percent_change(), Some(100.0));
    let df = spark.to_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("timestamp").unwrap().null_count(), 2);
}

#[test]
fn spark_pads_the_shorter_series() {
    let df = spark(&[1, 2, 3], &[1.5, 2.5]).to_dataframe().unwrap();
    assert_eq!(df.height(), 3);
    assert_eq!(df.column("close").unwrap().null_count(), 1);
    assert_eq!(df.column("symbol").unwrap().null_count(), 0);
}

#[test]
fn spark_meta_is_one_row() {
    let df = spark(&[1], &[1.5]).meta_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert!(df.width() > 0);
}
