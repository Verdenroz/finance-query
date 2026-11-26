use super::{FearAndGreed, FearGreedLabel, SymbolSentiment};
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

#[test]
fn fear_and_greed_columns() {
    let row: FearAndGreed = serde_json::from_value(
        json!({"value": 20, "classification": "Extreme Fear", "timestamp": 1}),
    )
    .unwrap();
    let expected = ["value:u32", "classification:str", "timestamp:i64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);
    assert_eq!(
        df.column("classification").unwrap().str().unwrap().get(0),
        Some("Extreme Fear")
    );

    let empty = <FearAndGreed>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn fear_greed_label_displays_its_serde_string() {
    for label in [
        FearGreedLabel::ExtremeFear,
        FearGreedLabel::Fear,
        FearGreedLabel::Neutral,
        FearGreedLabel::Greed,
        FearGreedLabel::ExtremeGreed,
    ] {
        assert_eq!(
            serde_json::to_value(label).unwrap(),
            json!(label.to_string())
        );
    }
}
