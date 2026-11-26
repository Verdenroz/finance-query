use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn filing_index_frame() {
    let index: EdgarFilingIndex = serde_json::from_value(json!({
        "directory": {"item": [
            {"name": "a.htm", "type": "text.gif", "size": "10"},
            {"name": "b.htm", "type": "text.gif", "size": ""}
        ]}
    }))
    .unwrap();
    let df = index.to_dataframe().unwrap();

    assert_eq!(columns(&df), ["name:str", "item_type:str", "size:u64"]);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("size").unwrap().null_count(), 1);

    let empty: EdgarFilingIndex = serde_json::from_value(json!({})).unwrap();
    assert_eq!(empty.to_dataframe().unwrap().height(), 0);
}

#[test]
fn cik_entry_columns() {
    let rows: Vec<CikEntry> =
        serde_json::from_value(json!([{"cik": 320193, "ticker": "AAPL", "title": "Apple"}]))
            .unwrap();
    assert_eq!(
        columns(&CikEntry::vec_to_dataframe(&rows).unwrap()),
        ["cik:u64", "ticker:str", "title:str"]
    );
}

#[test]
fn provider_filings_frame() {
    let filings: ProviderFilings = serde_json::from_value(json!({
        "symbol": "AAPL",
        "filings": [{"accession_number": "0001", "filing_type": "10-K"}]
    }))
    .unwrap();
    let df = filings.to_dataframe().unwrap();
    let expected = [
        "symbol:str",
        "accession_number:str",
        "filing_date:str",
        "filing_type:str",
        "filing_url:str",
        "company_name:str",
        "cik:str",
    ];
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty: ProviderFilings =
        serde_json::from_value(json!({"symbol": "AAPL", "filings": []})).unwrap();
    let empty = empty.to_dataframe().unwrap();
    assert_eq!(
        (empty.height(), columns(&empty)),
        (0, expected.map(String::from).to_vec())
    );
}

#[test]
fn section_and_risk_factor_columns() {
    let sections: Vec<FilingSection> = serde_json::from_value(json!([{"section": "1A"}])).unwrap();
    assert_eq!(
        columns(&FilingSection::vec_to_dataframe(&sections).unwrap()),
        ["section:str", "content:str"]
    );

    let risks: Vec<RiskFactor> = serde_json::from_value(json!([{"title": "t"}])).unwrap();
    assert_eq!(
        columns(&RiskFactor::vec_to_dataframe(&risks).unwrap()),
        ["title:str", "text:str", "category:str", "filing_date:str"]
    );
}

#[test]
fn insider_trade_columns() {
    let rows: Vec<InsiderTrade> = serde_json::from_value(json!([{
        "is_director": true, "is_officer": false, "is_ten_percent_owner": false,
        "is_derivative": false, "shares": 5.0
    }]))
    .unwrap();
    let cols = columns(&InsiderTrade::vec_to_dataframe(&rows).unwrap());

    assert_eq!(cols.len(), 19);
    assert!(cols.contains(&"is_director:bool".to_string()));
    assert!(cols.contains(&"shares:f64".to_string()));
    assert!(cols.contains(&"transaction_date:str".to_string()));
}

#[test]
fn holding_trade_and_ftd_columns() {
    let holdings: Vec<InstitutionalHolding> =
        serde_json::from_value(json!([{"cusip": "037833100", "value": 1.0}])).unwrap();
    let cols = columns(&InstitutionalHolding::vec_to_dataframe(&holdings).unwrap());
    assert_eq!(cols.len(), 13);
    assert!(cols.contains(&"value:f64".to_string()));
    assert!(cols.contains(&"report_date:str".to_string()));

    let trades: Vec<CongressionalTrade> =
        serde_json::from_value(json!([{"symbol": "AAPL", "office": "House"}])).unwrap();
    let cols = columns(&CongressionalTrade::vec_to_dataframe(&trades).unwrap());
    assert_eq!(cols.len(), 11);
    assert_eq!(cols[0], "symbol:str");

    let ftd: Vec<FailToDeliver> =
        serde_json::from_value(json!([{"symbol": "AAPL", "quantity": 100.0}])).unwrap();
    assert_eq!(
        columns(&FailToDeliver::vec_to_dataframe(&ftd).unwrap()),
        [
            "symbol:str",
            "date:str",
            "quantity:f64",
            "price:f64",
            "name:str",
            "description:str",
        ]
    );
}

#[test]
fn search_hit_and_file_columns() {
    let hits: Vec<FilingSearchHit> =
        serde_json::from_value(json!([{"company_names": ["Apple"], "ciks": [], "score": 1.0}]))
            .unwrap();
    assert_eq!(
        columns(&FilingSearchHit::vec_to_dataframe(&hits).unwrap()),
        [
            "accession_number:str",
            "form:str",
            "filed_date:str",
            "period_ending:str",
            "score:f64",
            "url:str",
        ]
    );

    let files: Vec<EdgarFilingFile> =
        serde_json::from_value(json!([{"name": "CIK-submissions-001.json", "filingCount": 3}]))
            .unwrap();
    assert_eq!(
        columns(&EdgarFilingFile::vec_to_dataframe(&files).unwrap()),
        [
            "name:str",
            "filing_count:u32",
            "filing_from:str",
            "filing_to:str",
        ]
    );
}

#[test]
fn edgar_search_total_columns() {
    let row: EdgarSearchTotal = serde_json::from_value(json!({})).unwrap();
    let expected = ["value:u64", "relation:str"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EdgarSearchTotal>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}
