use crate::models::dataframe_test_util::columns;
use crate::models::quote::snapshot::MarketSnapshot;
use crate::models::quote::{Price, QuoteTypeData};
use serde_json::json;

#[test]
fn price_columns() {
    let row: Price = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "max_age:i64",
        "pre_market_change_percent:f64",
        "pre_market_change:f64",
        "pre_market_time:i64",
        "pre_market_price:f64",
        "pre_market_source:str",
        "post_market_change_percent:f64",
        "post_market_change:f64",
        "post_market_time:i64",
        "post_market_price:f64",
        "post_market_source:str",
        "regular_market_change_percent:f64",
        "regular_market_change:f64",
        "regular_market_time:i64",
        "price_hint:i64",
        "regular_market_price:f64",
        "regular_market_day_high:f64",
        "regular_market_day_low:f64",
        "regular_market_volume:i64",
        "average_daily_volume10_day:i64",
        "average_daily_volume3_month:i64",
        "regular_market_previous_close:f64",
        "regular_market_source:str",
        "regular_market_open:f64",
        "exchange:str",
        "exchange_name:str",
        "exchange_data_delayed_by:i32",
        "market_state:str",
        "quote_type:str",
        "symbol:str",
        "underlying_symbol:str",
        "short_name:str",
        "long_name:str",
        "currency:str",
        "quote_source_name:str",
        "currency_symbol:str",
        "from_currency:str",
        "to_currency:str",
        "last_market:str",
        "market_cap:i64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <Price>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn quote_type_data_columns() {
    let row: QuoteTypeData = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "exchange:str",
        "first_trade_date_epoch_utc:i64",
        "gmt_off_set_milliseconds:i64",
        "long_name:str",
        "max_age:i64",
        "message_board_id:str",
        "quote_type:str",
        "short_name:str",
        "symbol:str",
        "time_zone_full_name:str",
        "time_zone_short_name:str",
        "underlying_symbol:str",
        "uuid:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <QuoteTypeData>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn market_snapshot_columns() {
    let row: MarketSnapshot = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "asset_class:str",
        "market_status:str",
        "last_price:f64",
        "bid:f64",
        "ask:f64",
        "open:f64",
        "high:f64",
        "low:f64",
        "close:f64",
        "previous_close:f64",
        "volume:f64",
        "change:f64",
        "change_percent:f64",
        "error:str",
        "message:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <MarketSnapshot>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
