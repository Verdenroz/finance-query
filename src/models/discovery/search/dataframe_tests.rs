use super::SearchResults;
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

#[test]
fn search_results_nest_quotes_news_and_reports() {
    let results: SearchResults = serde_json::from_value(json!({
        "count": 2,
        "quotes": [{"symbol": "AAPL"}],
        "news": [{"title": "t"}, {"title": "u"}],
        "researchReports": []
    }))
    .unwrap();
    assert_eq!(
        columns(&results.to_dataframe().unwrap()),
        ["count:i32", "total_time:i64"]
    );
    let quotes = columns(&results.quotes_dataframe().unwrap());
    assert!(quotes.contains(&"name_change_date:date".to_string()));
    assert_eq!(results.news_dataframe().unwrap().height(), 2);
    let reports = results.research_reports_dataframe().unwrap();
    assert_eq!(reports.height(), 0);
    assert!(reports.width() > 0);

    let cols = columns(&results.to_nested_dataframe().unwrap());
    assert!(cols[2].starts_with("quotes:list[struct["));
    assert!(cols[3].starts_with("news:list[struct["));
    assert!(cols[4].starts_with("research_reports:list[struct["));
}

#[test]
fn search_results_nested_news_carries_thumbnails() {
    let results: SearchResults = serde_json::from_value(json!({
        "news": [{"title": "t", "thumbnail": {"resolutions": [{"tag": "original"}]}}]
    }))
    .unwrap();
    let df = results.to_nested_dataframe().unwrap();
    let news = df
        .column("news")
        .unwrap()
        .list()
        .unwrap()
        .get_as_series(0)
        .unwrap();
    assert_eq!(news.len(), 1);
    assert!(news.struct_().unwrap().field_by_name("thumbnail").is_ok());
}

#[test]
fn search_results_nested_schema_does_not_depend_on_the_data() {
    let results: SearchResults = serde_json::from_value(json!({
        "quotes": [{"symbol": "AAPL"}],
        "news": [{"title": "t", "thumbnail": {"resolutions": [{"tag": "original"}]}}],
        "researchReports": [{"id": "1"}]
    }))
    .unwrap();
    let full = SearchResults::vec_to_nested_dataframe(&[results]).unwrap();
    let empty = SearchResults::vec_to_nested_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), columns(&full));
}
