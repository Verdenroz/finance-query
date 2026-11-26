use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[cfg(feature = "fmp")]
#[test]
fn analyst_estimate_columns() {
    let row: AnalystEstimate = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "date:str",
        "estimated_revenue_low:f64",
        "estimated_revenue_high:f64",
        "estimated_revenue_avg:f64",
        "estimated_ebitda_low:f64",
        "estimated_ebitda_high:f64",
        "estimated_ebitda_avg:f64",
        "estimated_eps_avg:f64",
        "estimated_eps_high:f64",
        "estimated_eps_low:f64",
        "number_analyst_estimated_revenue:i32",
        "number_analysts_estimated_eps:i32",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <AnalystEstimate>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[cfg(feature = "fmp")]
#[test]
fn analyst_recommendation_columns() {
    let row: AnalystRecommendation = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "date:str",
        "analyst_ratings_buy:i32",
        "analyst_ratings_hold:i32",
        "analyst_ratings_sell:i32",
        "analyst_ratings_strong_buy:i32",
        "analyst_ratings_strong_sell:i32",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <AnalystRecommendation>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
