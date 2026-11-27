use super::filing_index::EdgarFilingIndexDirectory;
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
        "filing_date:date",
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
        ["title:str", "text:str", "category:str", "filing_date:date"]
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
    assert!(cols.contains(&"transaction_date:date".to_string()));
}

#[test]
fn holding_trade_and_ftd_columns() {
    let holdings: Vec<InstitutionalHolding> =
        serde_json::from_value(json!([{"cusip": "037833100", "value": 1.0}])).unwrap();
    let cols = columns(&InstitutionalHolding::vec_to_dataframe(&holdings).unwrap());
    assert_eq!(cols.len(), 13);
    assert!(cols.contains(&"value:f64".to_string()));
    assert!(cols.contains(&"report_date:date".to_string()));

    let trades: Vec<CongressionalTrade> =
        serde_json::from_value(json!([{"symbol": "AAPL", "office": "House"}])).unwrap();
    let cols = columns(&CongressionalTrade::vec_to_dataframe(&trades).unwrap());
    assert_eq!(cols.len(), 11);
    assert_eq!(cols[0], "symbol:str");
    assert!(cols.contains(&"transaction_date:date".to_string()));
    assert!(cols.contains(&"disclosure_date:date".to_string()));

    let ftd: Vec<FailToDeliver> =
        serde_json::from_value(json!([{"symbol": "AAPL", "quantity": 100.0}])).unwrap();
    assert_eq!(
        columns(&FailToDeliver::vec_to_dataframe(&ftd).unwrap()),
        [
            "symbol:str",
            "date:date",
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
            "filed_date:date",
            "period_ending:date",
            "score:f64",
            "url:str",
            "company_names:list[str]",
            "ciks:list[str]",
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

#[test]
fn edgar_search_source_columns() {
    let row: EdgarSearchSource = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "file_date:date",
        "form:str",
        "adsh:str",
        "period_ending:date",
        "ciks:list[str]",
        "display_names:list[str]",
        "root_forms:list[str]",
        "sics:list[str]",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EdgarSearchSource>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn submissions_nest_the_filing_history() {
    let submissions: EdgarSubmissions = serde_json::from_value(json!({
        "cik": "320193",
        "name": "Apple",
        "tickers": ["AAPL"],
        "filings": {
            "recent": {
                "accessionNumber": ["0001", "0002"],
                "filingDate": ["2024-01-01", "2024-02-01"],
                "reportDate": ["2024-01-01", "2024-02-01"],
                "acceptanceDateTime": ["a", "b"],
                "form": ["10-K", "8-K"],
                "size": [1, 2],
                "isXBRL": [1, 0],
                "isInlineXBRL": [1, 0],
                "primaryDocument": ["a.htm", "b.htm"],
                "primaryDocDescription": ["x", "y"]
            },
            "files": [{"name": "older.json", "filingCount": 3}]
        }
    }))
    .unwrap();
    let flat = columns(&submissions.to_dataframe().unwrap());
    assert!(flat.contains(&"tickers:list[str]".to_string()));
    assert!(!flat.iter().any(|c| c.starts_with("filings")));

    let filings = submissions.filings_dataframe().unwrap();
    assert_eq!(filings.height(), 1);
    let cols = columns(&filings);
    assert!(cols[0].starts_with("recent:list[struct["));
    assert!(cols[1].starts_with("files:list[struct["));

    let filings = submissions.filings.as_ref().unwrap();
    let recent = filings.recent_dataframe().unwrap();
    assert_eq!(recent.height(), 2);
    assert_eq!(
        columns(&recent)[..2],
        ["accession_number:str", "filing_date:date"]
    );
    assert_eq!(filings.files_dataframe().unwrap().height(), 1);
}

#[test]
fn filings_without_recent_keep_a_consistent_nested_dtype() {
    let with: EdgarFilings = serde_json::from_value(json!({
        "recent": {"accessionNumber": ["0001"], "form": ["10-K"]},
        "files": []
    }))
    .unwrap();
    let without: EdgarFilings = serde_json::from_value(json!({})).unwrap();
    let both = EdgarFilings::vec_to_nested_dataframe(&[with.clone(), without]).unwrap();
    let recent = both.column("recent").unwrap().list().unwrap();
    assert_eq!(recent.get_as_series(0).unwrap().len(), 1);
    assert_eq!(recent.get_as_series(1).unwrap().len(), 0);
    assert_eq!(with.recent_dataframe().unwrap().height(), 1);
}

#[test]
fn filing_index_directory_exposes_its_items() {
    let directory: EdgarFilingIndexDirectory = serde_json::from_value(json!({
        "item": [{"name": "a.htm", "type": "text.gif", "size": "10"}]
    }))
    .unwrap();
    let items = directory.item_dataframe().unwrap();
    assert_eq!(columns(&items), ["name:str", "item_type:str", "size:u64"]);
    assert_eq!(items.height(), 1);
    assert_eq!(directory.to_dataframe().unwrap().width(), 0);
    assert_eq!(
        columns(&directory.to_nested_dataframe().unwrap()),
        ["item:list[struct[3]]"]
    );
}

#[test]
fn submissions_with_and_without_recent_filings_stack() {
    let with: EdgarSubmissions = serde_json::from_value(json!({
        "filings": {"recent": {"accessionNumber": ["0001"], "form": ["10-K"]}, "files": []}
    }))
    .unwrap();
    let without: EdgarSubmissions =
        serde_json::from_value(json!({"filings": {"files": []}})).unwrap();
    let df = EdgarSubmissions::vec_to_nested_dataframe(&[with, without]).unwrap();
    assert_eq!(df.height(), 2);
    let empty = EdgarSubmissions::vec_to_nested_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), columns(&df));
}

#[test]
fn missing_recent_filings_keep_the_row_columns() {
    let filings: EdgarFilings = serde_json::from_value(json!({})).unwrap();
    let recent = filings.recent_dataframe().unwrap();
    assert_eq!(recent.height(), 0);
    assert_eq!(
        columns(&recent)[..2],
        ["accession_number:str", "filing_date:date"]
    );
}

fn company_facts() -> CompanyFacts {
    serde_json::from_value(json!({
        "cik": 1,
        "facts": {
            "us-gaap": {
                "Revenue": {"units": {
                    "USD": [{"end": "2024-12-31", "val": 2.0}, {"end": "2023-12-31", "val": 1.0}],
                    "EUR": [{"end": "2024-12-31", "val": 3.0}]
                }},
                "Assets": {"units": {"USD": [{"end": "2024-12-31", "val": 4.0}]}}
            },
            "dei": {"Shares": {"units": {"shares": [{"end": "2024-12-31", "val": 5.0}]}}}
        }
    }))
    .unwrap()
}

#[test]
fn company_facts_long_frame_is_sorted() {
    let df = company_facts().to_dataframe().unwrap();
    assert_eq!(
        columns(&df)[..5],
        [
            "taxonomy:str",
            "concept:str",
            "unit:str",
            "start:date",
            "end:date"
        ]
    );
    assert_eq!(df.column("val").unwrap().dtype().to_string(), "f64");
    assert_eq!(df.column("filed").unwrap().dtype().to_string(), "date");
    let strings = |name: &str| -> Vec<String> {
        df.column(name)
            .unwrap()
            .cast(&polars::prelude::DataType::String)
            .unwrap()
            .str()
            .unwrap()
            .into_iter()
            .map(|value| value.unwrap_or_default().to_string())
            .collect()
    };
    assert_eq!(
        strings("taxonomy"),
        ["dei", "us-gaap", "us-gaap", "us-gaap", "us-gaap"]
    );
    assert_eq!(
        strings("concept"),
        ["Shares", "Assets", "Revenue", "Revenue", "Revenue"]
    );
    assert_eq!(strings("unit"), ["shares", "USD", "EUR", "USD", "USD"]);
    assert_eq!(strings("end")[3..], ["2024-12-31", "2023-12-31"]);
}

#[test]
fn facts_by_taxonomy_drops_the_taxonomy_column() {
    let facts = company_facts();
    let df = facts.us_gaap().unwrap().to_dataframe().unwrap();
    assert_eq!(columns(&df)[..3], ["concept:str", "unit:str", "start:date"]);
    assert_eq!(df.height(), 4);
}

#[test]
fn empty_company_facts_keep_the_schema() {
    let full = company_facts().to_dataframe().unwrap();
    let empty: CompanyFacts = serde_json::from_value(json!({})).unwrap();
    let df = empty.to_dataframe().unwrap();
    assert_eq!(df.height(), 0);
    assert_eq!(columns(&df), columns(&full));
}

#[test]
fn company_facts_frame_matches_stacked_unit_frames() {
    use crate::__dataframe::with_leading_column;

    let facts = company_facts();
    let schema = with_leading_column(FactUnit::vec_to_dataframe(&[]).unwrap(), "unit", "").unwrap();
    let schema = with_leading_column(schema, "concept", "").unwrap();
    let mut stacked = with_leading_column(schema, "taxonomy", "").unwrap();
    for (taxonomy, by_concept) in &facts.facts {
        for (concept, fact) in &by_concept.0 {
            for (unit, points) in &fact.units {
                let frame = FactUnit::vec_to_dataframe(points).unwrap();
                let frame = with_leading_column(frame, "unit", unit).unwrap();
                let frame = with_leading_column(frame, "concept", concept).unwrap();
                let frame = with_leading_column(frame, "taxonomy", taxonomy).unwrap();
                stacked.vstack_mut(&frame).unwrap();
            }
        }
    }

    assert!(facts.to_dataframe().unwrap().equals_missing(&stacked));
}
