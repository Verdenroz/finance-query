use super::*;
use crate::models::dataframe_test_util::columns;

#[test]
fn futures_quote_columns() {
    let quotes: Vec<FuturesQuote> = serde_json::from_value(serde_json::json!([
        {"symbol": "ESM26", "open_interest": 10, "volume": 20}
    ]))
    .unwrap();

    assert_eq!(
        columns(&FuturesQuote::vec_to_dataframe(&quotes).unwrap()),
        [
            "symbol:str",
            "name:str",
            "underlying:str",
            "exchange:str",
            "expiration_date:date",
            "price:f64",
            "change:f64",
            "change_percent:f64",
            "open_interest:u64",
            "volume:u64",
            "timestamp:datetime[μs, UTC]",
        ]
    );
}

#[cfg(feature = "cftc")]
#[test]
fn commitments_of_traders_frame() {
    use super::cot::CommitmentsOfTraders;

    let mut cot: CommitmentsOfTraders = serde_json::from_value(serde_json::json!({
        "symbol": "GC=F",
        "market_and_exchange_name": "GOLD",
        "cftc_contract_market_code": "088691",
        "observations": [
            {"report_date": "2024-01-02", "open_interest": 100}
        ]
    }))
    .unwrap();
    let df = cot.to_dataframe().unwrap();
    let cols = columns(&df);
    assert_eq!(
        cols[..3],
        ["symbol:str", "report_date:date", "open_interest:i64"]
    );
    assert_eq!(df.width(), 18);
    assert_eq!(df.height(), 1);

    cot.observations.clear();
    let empty = cot.to_dataframe().unwrap();
    assert_eq!((empty.height(), empty.width()), (0, 18));
}
