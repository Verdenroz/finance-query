use super::SymbolSentiment;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn symbol_sentiment_columns() {
    let row: SymbolSentiment = serde_json::from_value(json!({})).unwrap();
    let expected = ["score:f64", "label:str"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SymbolSentiment>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
