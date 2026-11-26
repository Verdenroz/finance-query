use crate::models::dataframe_test_util::columns;
use crate::models::market::index_trend::TrendEstimate;
use crate::models::market::performance::{IndustryPe, MoverQuote, SectorPe, SectorPerformance};
use serde_json::json;

#[test]
fn sector_performance_columns() {
    let row: SectorPerformance = serde_json::from_value(json!({"sector": "x"})).unwrap();
    let expected = ["sector:str", "exchange:str", "change_percent:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SectorPerformance>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn sector_pe_columns() {
    let row: SectorPe = serde_json::from_value(json!({"sector": "x"})).unwrap();
    let expected = ["sector:str", "exchange:str", "pe:f64", "date:str"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SectorPe>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn industry_pe_columns() {
    let row: IndustryPe = serde_json::from_value(json!({"industry": "x"})).unwrap();
    let expected = ["industry:str", "exchange:str", "pe:f64", "date:str"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <IndustryPe>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn mover_quote_columns() {
    let row: MoverQuote = serde_json::from_value(json!({"symbol": "x"})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "price:f64",
        "change:f64",
        "change_percent:f64",
        "exchange:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <MoverQuote>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn trend_estimate_columns() {
    let row: TrendEstimate = serde_json::from_value(json!({})).unwrap();
    let expected = ["period:str", "growth:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <TrendEstimate>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
