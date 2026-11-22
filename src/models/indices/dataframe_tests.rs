use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn index_quote_columns() {
    let quotes: Vec<IndexQuote> = serde_json::from_value(json!([{"symbol": "^GSPC"}])).unwrap();
    assert_eq!(
        columns(&IndexQuote::vec_to_dataframe(&quotes).unwrap()),
        [
            "symbol:str",
            "name:str",
            "price:f64",
            "change:f64",
            "change_percent:f64",
            "timestamp:i64",
        ]
    );
}

#[test]
fn constituent_columns() {
    let rows: Vec<IndexConstituent> = serde_json::from_value(json!([{"symbol": "AAPL"}])).unwrap();
    assert_eq!(
        columns(&IndexConstituent::vec_to_dataframe(&rows).unwrap()),
        [
            "symbol:str",
            "name:str",
            "sector:str",
            "sub_sector:str",
            "headquarters:str",
            "date_first_added:str",
            "cik:str",
            "founded:str",
        ]
    );
}

#[test]
fn constituent_change_columns_and_empty() {
    let rows: Vec<IndexConstituentChange> =
        serde_json::from_value(json!([{"date": "2024-03-18", "symbol": "SMCI"}])).unwrap();
    let expected = [
        "date:str",
        "symbol:str",
        "added_security:str",
        "removed_ticker:str",
        "removed_security:str",
        "reason:str",
    ];
    assert_eq!(
        columns(&IndexConstituentChange::vec_to_dataframe(&rows).unwrap()),
        expected
    );

    let empty = IndexConstituentChange::vec_to_dataframe(&[]).unwrap();
    assert_eq!(
        (empty.height(), columns(&empty)),
        (0, expected.map(String::from).to_vec())
    );
}
