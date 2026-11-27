use super::listings::{StockListing, StockType, TickerChange};
use super::reference::{ExchangeInfo, ScreenerMatch, SymbolDetails, SymbolMatch};
use super::screeners::ScreenerQuote;
use crate::models::dataframe_test_util::columns;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn parse<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

#[cfg(feature = "openfigi")]
#[test]
fn security_mapping_columns() {
    use super::figi::SecurityMapping;

    let rows: Vec<SecurityMapping> = parse(json!([{"figi": "BBG000B9XRY4", "ticker": "AAPL"}]));
    assert_eq!(
        columns(&SecurityMapping::vec_to_dataframe(&rows).unwrap()),
        [
            "figi:str",
            "ticker:str",
            "name:str",
            "exchange_code:str",
            "composite_figi:str",
            "share_class_figi:str",
            "security_type:str",
            "market_sector:str",
        ]
    );
}

#[test]
fn listing_frames() {
    let listings: Vec<StockListing> = parse(json!([{"symbol": "AAPL", "active": true}]));
    assert_eq!(
        columns(&StockListing::vec_to_dataframe(&listings).unwrap()),
        [
            "symbol:str",
            "name:str",
            "exchange:str",
            "stock_type:str",
            "locale:str",
            "active:bool",
            "cik:str",
            "composite_figi:str",
            "share_class_figi:str",
            "list_date:str",
            "delisted_utc:str",
        ]
    );

    let changes: Vec<TickerChange> = parse(json!([{"date": "2022-06-09", "ticker": "META"}]));
    assert_eq!(
        columns(&TickerChange::vec_to_dataframe(&changes).unwrap()),
        ["date:str", "ticker:str"]
    );

    let types: Vec<StockType> = parse(json!([{"code": "CS"}]));
    assert_eq!(
        columns(&StockType::vec_to_dataframe(&types).unwrap()),
        [
            "code:str",
            "description:str",
            "asset_class:str",
            "locale:str",
        ]
    );
    assert_eq!(StockType::vec_to_dataframe(&[]).unwrap().height(), 0);
}

#[test]
fn symbol_match_and_details_columns() {
    let matches: Vec<SymbolMatch> = parse(json!([{"symbol": "BTC", "market_cap_rank": 1}]));
    assert_eq!(
        columns(&SymbolMatch::vec_to_dataframe(&matches).unwrap()),
        [
            "symbol:str",
            "id:str",
            "name:str",
            "exchange:str",
            "asset_type:str",
            "currency:str",
            "active:bool",
            "market_cap_rank:u32",
            "thumbnail:str",
            "image:str",
            "ipo_date:str",
            "delisted_date:str",
        ]
    );

    let details: SymbolDetails = parse(json!({"symbol": "AAPL", "employees": 100}));
    let cols = columns(&details.to_dataframe().unwrap());
    assert_eq!(cols[0], "symbol:str");
    assert!(cols.contains(&"employees:u64".to_string()));
    assert!(cols.contains(&"market_cap:f64".to_string()));
    assert!(cols.contains(&"active:bool".to_string()));
}

#[test]
fn exchange_and_screener_match_columns() {
    let exchanges: Vec<ExchangeInfo> = parse(json!([{"id": 1, "mic": "XNAS"}]));
    assert_eq!(
        columns(&ExchangeInfo::vec_to_dataframe(&exchanges).unwrap()),
        [
            "id:i64",
            "name:str",
            "mic:str",
            "operating_mic:str",
            "asset_class:str",
            "locale:str",
            "exchange_type:str",
            "url:str",
        ]
    );

    let matches: Vec<ScreenerMatch> = parse(json!([{"symbol": "AAPL", "is_etf": false}]));
    let cols = columns(&ScreenerMatch::vec_to_dataframe(&matches).unwrap());
    assert_eq!(cols.len(), 13);
    assert!(cols.contains(&"beta:f64".to_string()));
    assert!(cols.contains(&"is_etf:bool".to_string()));
}

#[test]
fn screener_quote_epochs_are_typed() {
    let quote: ScreenerQuote = parse(json!({
        "symbol": "AAPL",
        "quoteType": "EQUITY",
        "exchange": "NMS",
        "regularMarketPrice": {"raw": 1.0},
        "regularMarketChange": {"raw": 0.1},
        "regularMarketChangePercent": {"raw": 0.1},
        "regularMarketTime": {"raw": 1794603601},
        "dividendDate": {"raw": 1795132800},
        "earningsTimestamp": {"raw": 1785441600}
    }));
    let df = quote.to_dataframe().unwrap();
    let micros = |name| df.column(name).unwrap().datetime().unwrap().phys.get(0);
    assert_eq!(micros("regular_market_time"), Some(1_794_603_601_000_000));
    assert_eq!(micros("earnings_timestamp"), Some(1_785_441_600_000_000));
    assert_eq!(micros("pre_market_time"), None);
    let day = df.column("dividend_date").unwrap().date().unwrap();
    assert_eq!(day.phys.get(0), Some(20_777));
}
