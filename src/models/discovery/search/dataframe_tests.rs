use super::news::SearchNews;
use super::thumbnail::{NewsThumbnail, ThumbnailResolution};
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn thumbnail_resolution_columns() {
    let row: ThumbnailResolution = serde_json::from_value(json!({})).unwrap();
    let expected = ["url:str", "width:i32", "height:i32", "tag:str"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <ThumbnailResolution>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn news_thumbnail_exposes_resolutions() {
    let thumbnail: NewsThumbnail = serde_json::from_value(json!({
        "resolutions": [{"url": "u", "width": 1, "height": 2, "tag": "original"}]
    }))
    .unwrap();
    let df = thumbnail.resolutions_dataframe().unwrap();
    assert_eq!(
        columns(&df),
        ["url:str", "width:i32", "height:i32", "tag:str"]
    );
    assert_eq!(df.height(), 1);

    let none: NewsThumbnail = serde_json::from_value(json!({})).unwrap();
    assert_eq!(
        columns(&none.resolutions_dataframe().unwrap()),
        columns(&df)
    );
}

#[test]
fn search_news_keeps_flat_columns_and_nests_the_thumbnail() {
    let news: SearchNews = serde_json::from_value(json!({
        "title": "t", "thumbnail": {"resolutions": [{"tag": "original"}]}
    }))
    .unwrap();
    let flat = news.to_dataframe().unwrap();
    assert!(!columns(&flat).iter().any(|c| c.starts_with("thumbnail")));
    assert_eq!(
        columns(&news.to_nested_dataframe().unwrap())
            .last()
            .unwrap(),
        "thumbnail:list[struct[1]]"
    );
}
