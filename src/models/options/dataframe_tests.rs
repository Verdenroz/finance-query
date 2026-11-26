use super::OptionChain;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn option_chain_nests_calls_and_puts() {
    let chain: OptionChain = serde_json::from_value(json!({
        "expirationDate": 1,
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
        ["expiration_date:i64", "has_mini_options:bool"]
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
