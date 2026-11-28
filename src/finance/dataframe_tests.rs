use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[cfg(feature = "fmp")]
#[test]
fn analyst_estimate_columns() {
    let row: AnalystEstimate = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "date:date",
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
        "date:date",
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

#[cfg(feature = "alphavantage")]
#[test]
fn earnings_calendar_entry_columns() {
    let row: EarningsCalendarEntry = serde_json::from_value(json!({"symbol": "x"})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "report_date:date",
        "fiscal_date_ending:date",
        "estimate:f64",
        "currency:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EarningsCalendarEntry>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[cfg(feature = "alphavantage")]
#[test]
fn ipo_calendar_entry_columns() {
    let row: IpoCalendarEntry = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "ipo_date:date",
        "price_range:str",
        "exchange:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <IpoCalendarEntry>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[cfg(feature = "fmp")]
#[test]
fn insider_transaction_columns() {
    let row: InsiderTransaction = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "filing_date:date",
        "transaction_date:date",
        "reporting_name:str",
        "transaction_type:str",
        "securities_transacted:f64",
        "price:f64",
        "securities_owned:f64",
        "type_of_owner:str",
        "link:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <InsiderTransaction>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
