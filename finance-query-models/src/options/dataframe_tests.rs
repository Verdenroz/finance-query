use super::{OptionChain, OptionsQuote};
use crate::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn option_chain_nests_calls_and_puts() {
    let chain: OptionChain = serde_json::from_value(json!({
        "expirationDate": 1794528000,
        "hasMiniOptions": false,
        "calls": [
            {"contractSymbol": "C1", "strike": 100.0},
            {"contractSymbol": "C2", "strike": 105.0}
        ],
        "puts": []
    }))
    .unwrap();
    assert_eq!(
        columns(&chain.to_dataframe().unwrap()),
        ["expiration_date:date", "has_mini_options:bool"]
    );

    let calls = chain.calls_dataframe().unwrap();
    assert_eq!(columns(&calls)[..2], ["contract_symbol:str", "strike:f64"]);
    assert_eq!(calls.height(), 2);
    let puts = chain.puts_dataframe().unwrap();
    assert_eq!(puts.height(), 0);
    assert_eq!(columns(&puts), columns(&calls));

    let nested = chain.to_nested_dataframe().unwrap();
    let cols = columns(&nested);
    assert!(cols[2].starts_with("calls:list[struct["));
    assert!(cols[3].starts_with("puts:list[struct["));
    assert_eq!(nested.column("puts").unwrap().null_count(), 0);
}

#[test]
fn option_epochs_are_typed() {
    let chain: OptionChain = serde_json::from_value(json!({
        "expirationDate": 1794528000,
        "hasMiniOptions": false,
        "calls": [{
            "contractSymbol": "C1", "strike": 100.0, "expiration": 1794528000,
            "lastTradeDate": 1794603601
        }],
        "puts": []
    }))
    .unwrap();
    let day = chain.to_dataframe().unwrap();
    let days = day.column("expiration_date").unwrap().date().unwrap();
    assert_eq!(days.phys.get(0), Some(20_770));

    let calls = chain.calls_dataframe().unwrap();
    let cols = columns(&calls);
    assert!(cols.contains(&"expiration:date".to_string()));
    assert!(cols.contains(&"last_trade_date:datetime[μs, UTC]".to_string()));
    let traded = calls.column("last_trade_date").unwrap().datetime().unwrap();
    assert_eq!(traded.phys.get(0), Some(1_794_603_601_000_000));

    let quote: OptionsQuote = serde_json::from_value(json!({
        "symbol": "AAPL", "regularMarketTime": 1794603601
    }))
    .unwrap();
    let df = quote.to_dataframe().unwrap();
    let time = df
        .column("regular_market_time")
        .unwrap()
        .datetime()
        .unwrap();
    assert_eq!(time.phys.get(0), Some(1_794_603_601_000_000));
}

#[test]
fn unknown_expirations_are_null() {
    let chain: OptionChain = serde_json::from_value(json!({
        "expirationDate": 0,
        "hasMiniOptions": false,
        "calls": [{ "contractSymbol": "C1", "strike": 100.0, "expiration": 0 }],
        "puts": []
    }))
    .unwrap();
    let frame = chain.to_dataframe().unwrap();
    assert_eq!(frame.column("expiration_date").unwrap().null_count(), 1);
    let calls = chain.calls_dataframe().unwrap();
    assert_eq!(calls.column("expiration").unwrap().null_count(), 1);
}
