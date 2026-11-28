use crate::models::chart::ChartMeta;
use crate::models::dataframe_test_util::{columns, micros};
use crate::models::quote::snapshot::MarketSnapshot;
use crate::models::quote::{Price, Quote, QuoteSummaryResponse, QuoteTypeData};
use serde_json::json;

#[test]
fn price_columns() {
    let row: Price = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "max_age:i64",
        "pre_market_change_percent:f64",
        "pre_market_change:f64",
        "pre_market_time:datetime[μs, UTC]",
        "pre_market_price:f64",
        "pre_market_source:str",
        "post_market_change_percent:f64",
        "post_market_change:f64",
        "post_market_time:datetime[μs, UTC]",
        "post_market_price:f64",
        "post_market_source:str",
        "regular_market_change_percent:f64",
        "regular_market_change:f64",
        "regular_market_time:datetime[μs, UTC]",
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
fn price_epoch_seconds_become_datetimes() {
    let row: Price = serde_json::from_value(json!({
        "regularMarketTime": 1794603601,
        "preMarketTime": 1794583200
    }))
    .unwrap();
    let df = row.to_dataframe().unwrap();
    assert_eq!(
        micros(&df, "regular_market_time"),
        Some(1_794_603_601_000_000)
    );
    assert_eq!(micros(&df, "pre_market_time"), Some(1_794_583_200_000_000));
    assert_eq!(micros(&df, "post_market_time"), None);
}

#[test]
fn quote_type_first_trade_keeps_the_open_time() {
    let row: QuoteTypeData =
        serde_json::from_value(json!({ "firstTradeDateEpochUtc": 345479400 })).unwrap();
    let df = row.to_dataframe().unwrap();
    let time = df
        .column("first_trade_date")
        .unwrap()
        .datetime()
        .unwrap();
    assert_eq!(time.phys.get(0), Some(345_479_400_000_000));
}

#[test]
fn quote_type_data_columns() {
    let row: QuoteTypeData = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "exchange:str",
        "first_trade_date:datetime[μs, UTC]",
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

#[test]
fn quote_summary_flat_frame_has_only_the_symbol() {
    let summary = QuoteSummaryResponse {
        symbol: "AAPL".into(),
        ..Default::default()
    };
    let df = summary.to_dataframe().unwrap();
    assert_eq!(columns(&df), ["symbol:str"]);
    assert_eq!(df.height(), 1);

    let empty = <QuoteSummaryResponse>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), ["symbol:str"]);
    assert_eq!(empty.height(), 0);
}

#[test]
fn quote_summary_nested_frame_carries_every_module() {
    let summary = QuoteSummaryResponse {
        price: Some(serde_json::from_value(json!({})).unwrap()),
        ..Default::default()
    };
    let df = summary.to_nested_dataframe().unwrap();
    assert_eq!(df.height(), 1);
    assert_eq!(df.width(), 28);
    assert!(columns(&df)[1..].iter().all(|c| c.contains(":list[struct")));
    assert_eq!(summary.price_dataframe().unwrap().height(), 1);

    let empty = <QuoteSummaryResponse>::vec_to_nested_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), columns(&df));
}

#[test]
fn quote_epoch_fields_are_typed() {
    let quote: Quote = serde_json::from_value(json!({
        "symbol": "AAPL",
        "regularMarketTime": 1794603601,
        "exDividendDate": { "raw": 1794528000, "fmt": "2026-11-09" },
        "firstTradeDateEpochUtc": 345479400
    }))
    .unwrap();
    let df = quote.to_dataframe().unwrap();
    let columns = columns(&df);
    assert!(columns.contains(&"regular_market_time:datetime[μs, UTC]".to_string()));
    assert!(columns.contains(&"ex_dividend_date:date".to_string()));
    assert!(columns.contains(&"first_trade_date:datetime[μs, UTC]".to_string()));
    let time = df
        .column("regular_market_time")
        .unwrap()
        .datetime()
        .unwrap();
    assert_eq!(time.phys.get(0), Some(1_794_603_601_000_000));
    let day = df.column("ex_dividend_date").unwrap().date().unwrap();
    assert_eq!(day.phys.get(0), Some(20_770));
}

#[test]
fn chart_meta_epoch_fields_are_typed() {
    let meta: ChartMeta = serde_json::from_value(json!({
        "symbol": "AAPL",
        "firstTradeDate": 345479400,
        "regularMarketTime": 1794603601
    }))
    .unwrap();
    let df = meta.to_dataframe().unwrap();
    let first = df.column("first_trade_date").unwrap().datetime().unwrap();
    assert_eq!(first.phys.get(0), Some(345_479_400_000_000));
    let time = df
        .column("regular_market_time")
        .unwrap()
        .datetime()
        .unwrap();
    assert_eq!(time.phys.get(0), Some(1_794_603_601_000_000));
}
