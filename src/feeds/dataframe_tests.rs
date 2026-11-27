use super::FeedEntry;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn feed_entry_columns() {
    let row: FeedEntry =
        serde_json::from_value(json!({"title": "x", "url": "x", "source": "x"})).unwrap();
    let expected = [
        "title:str",
        "url:str",
        "published:datetime[μs, UTC]",
        "summary:str",
        "source:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <FeedEntry>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
