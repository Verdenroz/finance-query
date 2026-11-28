use super::alerts::{AlertCondition, AlertEvent};
#[cfg(feature = "polygon")]
use super::book::{BookLevel, OrderBookUpdate};
#[cfg(feature = "fred")]
use super::economic::SeriesUpdate;
#[cfg(feature = "polygon")]
use super::options::{Greeks, OptionContractUpdate};
use super::pricing::{MarketHoursType, OptionType, PriceUpdate, QuoteType};
#[cfg(feature = "polygon")]
use super::trades::TradeTick;
use crate::models::dataframe_test_util::{columns, columns_exempt, micros};
use polars::prelude::DataType;
use serde_json::json;

#[cfg(any(feature = "polygon", feature = "fred"))]
fn assert_schema<T: crate::ToDataFrame>(row: &T, expected: &[&str]) {
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = T::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn display_matches_serde_names() {
    for value in [QuoteType::AltSymbol, QuoteType::EcnQuote, QuoteType::Etf] {
        assert_eq!(
            json!(value.to_string()),
            serde_json::to_value(value).unwrap()
        );
    }
    for value in [OptionType::Call, OptionType::Put] {
        assert_eq!(
            json!(value.to_string()),
            serde_json::to_value(value).unwrap()
        );
    }
    for value in [
        MarketHoursType::PreMarket,
        MarketHoursType::ExtendedHoursMarket,
    ] {
        assert_eq!(
            json!(value.to_string()),
            serde_json::to_value(value).unwrap()
        );
    }
}

#[test]
fn price_update_columns() {
    let update = PriceUpdate {
        quote_type: QuoteType::Equity,
        market_hours: MarketHoursType::RegularMarket,
        ..Default::default()
    };
    let df = update.to_dataframe().unwrap();
    assert_eq!(df.width(), 33);
    assert_eq!(
        columns_exempt(&df, &["expire_date"])[..8],
        [
            "id:str",
            "price:f64",
            "time:datetime[μs, UTC]",
            "currency:str",
            "exchange:str",
            "quote_type:str",
            "market_hours:str",
            "change_percent:f64",
        ]
    );
    assert_eq!(
        df.column("quote_type").unwrap().str().unwrap().get(0),
        Some("EQUITY")
    );
    assert_eq!(
        df.column("options_type").unwrap().dtype().to_string(),
        "str"
    );
    assert_eq!(PriceUpdate::vec_to_dataframe(&[]).unwrap().width(), 33);
}

#[test]
fn price_update_time_is_epoch_millis_as_microseconds() {
    let update = PriceUpdate {
        time: 1_700_000_000_000,
        ..Default::default()
    };
    let df = update.to_dataframe().unwrap();
    assert_eq!(micros(&df, "time"), Some(1_700_000_000_000_000));
    assert_eq!(df.column("expire_date").unwrap().dtype(), &DataType::Int64);
}

#[cfg(feature = "polygon")]
#[test]
fn book_level_and_greeks_columns() {
    assert_schema(&BookLevel::default(), &["price:f64", "size:f64"]);
    assert_schema(
        &Greeks::default(),
        &["delta:f64", "gamma:f64", "theta:f64", "vega:f64"],
    );
}

#[cfg(feature = "polygon")]
#[test]
fn trade_tick_columns_leave_out_conditions() {
    assert_schema(
        &TradeTick::default(),
        &[
            "symbol:str",
            "price:f64",
            "size:f64",
            "exchange:i32",
            "trade_id:str",
            "time:datetime[μs, UTC]",
        ],
    );
}

#[cfg(feature = "polygon")]
#[test]
fn order_book_nests_both_sides() {
    let book = OrderBookUpdate {
        symbol: "BTC-USD".into(),
        bids: vec![BookLevel::default(), BookLevel::default()],
        asks: vec![BookLevel::default()],
        ..Default::default()
    };
    assert_schema(
        &book,
        &["symbol:str", "exchange:i32", "time:datetime[μs, UTC]"],
    );
    assert_eq!(book.bids_dataframe().unwrap().height(), 2);
    assert_eq!(book.asks_dataframe().unwrap().height(), 1);

    let nested = book.to_nested_dataframe().unwrap();
    assert_eq!(nested.height(), 1);
    assert_eq!(
        columns(&nested)[3..],
        [
            "bids:list[struct[2]]".to_string(),
            "asks:list[struct[2]]".to_string()
        ]
    );
}

#[cfg(feature = "polygon")]
#[test]
fn option_contract_flattens_greeks() {
    let update = OptionContractUpdate {
        option_type: Some(OptionType::Put),
        greeks: Some(Greeks::default()),
        ..Default::default()
    };
    let df = update.to_dataframe().unwrap();
    let names = columns(&df);
    assert!(names.contains(&"option_type:str".to_string()));
    assert_eq!(
        names[names.len() - 5..names.len() - 1],
        [
            "greeks_delta:f64",
            "greeks_gamma:f64",
            "greeks_theta:f64",
            "greeks_vega:f64",
        ]
    );
    assert_eq!(names.last().unwrap(), "time:datetime[μs, UTC]");
    let expiration = OptionContractUpdate {
        expiration: Some(1_737_072_000),
        ..Default::default()
    };
    let days = expiration
        .to_dataframe()
        .unwrap()
        .column("expiration")
        .unwrap()
        .cast(&DataType::Int32)
        .unwrap();
    assert_eq!(days.i32().unwrap().get(0), Some(20_105));
    assert_eq!(
        df.column("option_type").unwrap().str().unwrap().get(0),
        Some("PUT")
    );
    assert_eq!(
        columns(&OptionContractUpdate::vec_to_dataframe(&[]).unwrap()),
        names
    );
}

#[cfg(feature = "fred")]
#[test]
fn series_update_columns() {
    assert_schema(
        &SeriesUpdate {
            series_id: "FEDFUNDS".into(),
            date: "2024-01-01".into(),
            value: None,
            previous_value: None,
            revision: false,
            observed_at: 0,
        },
        &[
            "series_id:str",
            "date:date",
            "value:f64",
            "previous_value:f64",
            "revision:bool",
            "observed_at:datetime[μs, UTC]",
        ],
    );
}

#[test]
fn alert_event_flattens_the_tick_and_appends_the_condition_kind() {
    let event = AlertEvent {
        condition: AlertCondition::PriceAbove(1.0),
        previous_price: None,
        update: PriceUpdate::default(),
    };
    let df = event.to_dataframe().unwrap();
    assert_eq!(
        columns_exempt(&df, &["update_expire_date"])[..2],
        ["previous_price:f64", "update_id:str"]
    );
    assert_eq!(df.width(), 1 + 33 + 1);
    assert_eq!(
        columns_exempt(&df, &["update_expire_date"]).last().unwrap(),
        "condition_kind:str"
    );
    assert_eq!(
        df.column("condition_kind").unwrap().str().unwrap().get(0),
        Some("priceAbove")
    );

    let empty = AlertEvent::vec_to_dataframe(&[]).unwrap();
    assert_eq!(
        columns_exempt(&empty, &["update_expire_date"]),
        columns_exempt(&df, &["update_expire_date"])
    );
    assert_eq!(empty.height(), 0);

    let two = AlertEvent::vec_to_dataframe(&[event.clone(), event]).unwrap();
    assert_eq!(two.height(), 2);
}

#[test]
fn zero_stream_times_are_null() {
    #[cfg(feature = "polygon")]
    {
        let trade = TradeTick::default().to_dataframe().unwrap();
        assert_eq!(trade.column("time").unwrap().null_count(), 1);
    }
    let price = PriceUpdate::default().to_dataframe().unwrap();
    assert_eq!(price.column("time").unwrap().null_count(), 1);
}
