use super::{Dividend, DividendAnalytics, StockBar};
use crate::models::dataframe_test_util::{columns, micros};
use serde_json::json;

#[test]
fn stock_bar_columns() {
    let row: StockBar = serde_json::from_value(json!({"timestamp_ms": 1, "open": 1.5, "high": 1.5, "low": 1.5, "close": 1.5, "volume": 1.5})).unwrap();
    let expected = [
        "timestamp:datetime[μs, UTC]",
        "open:f64",
        "high:f64",
        "low:f64",
        "close:f64",
        "volume:f64",
        "transactions:u64",
        "vwap:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <StockBar>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn dividend_analytics_flattens_both_payments() {
    let dividend = |timestamp| -> Dividend {
        serde_json::from_value(json!({"timestamp": timestamp, "amount": 0.5})).unwrap()
    };
    let analytics = DividendAnalytics::from_dividends(&[dividend(86_400), dividend(172_800)]);
    let expected = [
        "total_paid:f64",
        "payment_count:u64",
        "average_payment:f64",
        "cagr:f64",
        "last_payment_timestamp:datetime[μs, UTC]",
        "last_payment_amount:f64",
        "last_payment_provider_id:str",
        "first_payment_timestamp:datetime[μs, UTC]",
        "first_payment_amount:f64",
        "first_payment_provider_id:str",
    ];
    let df = analytics.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);
    assert_eq!(
        micros(&df, "last_payment_timestamp"),
        Some(2 * 86_400_000_000)
    );
    assert_eq!(micros(&df, "first_payment_timestamp"), Some(86_400_000_000));

    let none = DividendAnalytics::from_dividends(&[])
        .to_dataframe()
        .unwrap();
    assert_eq!(columns(&none), expected);
    assert_eq!(none.column("last_payment_amount").unwrap().null_count(), 1);

    let empty = <DividendAnalytics>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
