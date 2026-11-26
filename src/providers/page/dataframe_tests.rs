use super::ProviderPage;
use crate::models::dataframe_test_util::columns;
use crate::{Dividend, Provider, ToDataFrame};
use serde_json::json;

fn page(timestamps: &[i64]) -> ProviderPage<Dividend> {
    let items = timestamps
        .iter()
        .map(|timestamp| serde_json::from_value(json!({"timestamp": timestamp, "amount": 0.5})))
        .collect::<Result<_, _>>()
        .unwrap();
    ProviderPage::new(items, Provider::Yahoo, None)
}

#[test]
fn provider_page_frame_is_its_items() {
    let expected = ["timestamp:i64", "amount:f64", "provider_id:str"];
    let df = page(&[1, 2]).to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 2);

    let empty = page(&[]).to_dataframe().unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn provider_pages_stack_and_keep_the_empty_schema() {
    let df = ProviderPage::vec_to_dataframe(&[page(&[1]), page(&[]), page(&[2, 3])]).unwrap();
    assert_eq!(df.height(), 3);
    let empty = <ProviderPage<Dividend> as ToDataFrame>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), columns(&df));
}

#[test]
fn provider_page_nested_frame_uses_the_item_type() {
    let nested = page(&[1]).to_nested_dataframe().unwrap();
    assert_eq!(nested.height(), 1);
    let stacked = ProviderPage::vec_to_nested_dataframe(&[page(&[1]), page(&[2])]).unwrap();
    assert_eq!(stacked.height(), 2);
}
