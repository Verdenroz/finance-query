use crate::dataframe_test_util::columns;
use crate::market::index_trend::TrendEstimate;
use crate::market::industries::IndustryData;
use crate::market::performance::{
    IndustryPe, MoverQuote, SectorPe, SectorPerformance, SectorPerformanceHistory,
};
use crate::market::sectors::SectorData;
use crate::market::{IndexTrend, IndustryTrend, SectorTrend};
use serde_json::json;

#[test]
fn sector_performance_columns() {
    let row: SectorPerformance = serde_json::from_value(json!({"sector": "x"})).unwrap();
    let expected = ["sector:str", "exchange:str", "change_percent:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SectorPerformance>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn sector_pe_columns() {
    let row: SectorPe = serde_json::from_value(json!({"sector": "x"})).unwrap();
    let expected = ["sector:str", "exchange:str", "pe:f64", "date:date"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SectorPe>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn industry_pe_columns() {
    let row: IndustryPe = serde_json::from_value(json!({"industry": "x"})).unwrap();
    let expected = ["industry:str", "exchange:str", "pe:f64", "date:date"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <IndustryPe>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn mover_quote_columns() {
    let row: MoverQuote = serde_json::from_value(json!({"symbol": "x"})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "price:f64",
        "change:f64",
        "change_percent:f64",
        "exchange:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <MoverQuote>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn trend_estimate_columns() {
    let row: TrendEstimate = serde_json::from_value(json!({})).unwrap();
    let expected = ["period:str", "growth:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <TrendEstimate>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn sector_data_nests_its_row_lists() {
    let sector: SectorData = serde_json::from_value(json!({
        "name": "Technology",
        "key": "technology",
        "overview": {"companiesCount": 5},
        "topCompanies": [{"symbol": "AAPL"}, {"symbol": "MSFT"}],
        "topEtfs": [],
        "topMutualFunds": [],
        "industries": [{"name": "Software"}],
        "researchReports": [{"id": "r1", "reportDate": "2026-06-12T22:40:58Z"}]
    }))
    .unwrap();
    let flat = columns(&sector.to_dataframe().unwrap());
    assert_eq!(flat[..3], ["name:str", "symbol:str", "key:str"]);
    assert!(flat.contains(&"overview_companies_count:i64".to_string()));
    assert!(!flat.iter().any(|c| c.contains("top_companies")));

    let companies = sector.top_companies_dataframe().unwrap();
    assert_eq!(columns(&companies)[..2], ["symbol:str", "name:str"]);
    assert_eq!(companies.height(), 2);
    let etfs = sector.top_etfs_dataframe().unwrap();
    assert_eq!(etfs.height(), 0);
    assert_eq!(columns(&etfs)[0], "symbol:str");
    assert_eq!(sector.industries_dataframe().unwrap().height(), 1);
    let reports = columns(&sector.research_reports_dataframe().unwrap());
    assert!(reports.contains(&"report_date:datetime[μs, UTC]".to_string()));
    assert!(
        columns(&sector.to_nested_dataframe().unwrap())
            .last()
            .unwrap()
            .starts_with("research_reports:list[struct[")
    );
}

#[test]
fn industry_data_nests_its_row_lists() {
    let industry: IndustryData = serde_json::from_value(json!({
        "name": "Software",
        "key": "software",
        "topCompanies": [{"symbol": "MSFT"}],
        "topGrowthCompanies": [{"symbol": "NOW"}, {"symbol": "CRM"}],
        "researchReports": [{"reportDate": "2026-06-12T10:33:03Z"}]
    }))
    .unwrap();
    let reports = columns(&industry.research_reports_dataframe().unwrap());
    assert!(reports.contains(&"report_date:datetime[μs, UTC]".to_string()));
    assert_eq!(industry.top_companies_dataframe().unwrap().height(), 1);
    assert_eq!(
        industry.top_growth_companies_dataframe().unwrap().height(),
        2
    );
    let performing = industry.top_performing_companies_dataframe().unwrap();
    assert_eq!(performing.height(), 0);
    assert_eq!(columns(&performing)[0], "symbol:str");
    assert_eq!(
        columns(&industry.to_dataframe().unwrap())[..2],
        ["name:str", "key:str"]
    );
}

#[test]
fn sector_performance_history_nests_the_day() {
    let day: SectorPerformanceHistory = serde_json::from_value(json!({
        "date": "2024-01-02",
        "sectors": [{"sector": "Tech", "change_percent": 1.5}]
    }))
    .unwrap();
    assert_eq!(columns(&day.to_dataframe().unwrap()), ["date:date"]);
    let sectors = day.sectors_dataframe().unwrap();
    assert_eq!(
        columns(&sectors),
        ["sector:str", "exchange:str", "change_percent:f64"]
    );
    assert_eq!(sectors.height(), 1);
}

#[test]
fn trend_wrappers_nest_their_estimates() {
    let body = json!({
        "symbol": "SP5",
        "estimates": [{"period": "0q"}, {"period": "+1q"}]
    });
    let index: IndexTrend = serde_json::from_value(body.clone()).unwrap();
    let industry: IndustryTrend = serde_json::from_value(body.clone()).unwrap();
    let sector: SectorTrend = serde_json::from_value(body).unwrap();
    for estimates in [
        index.estimates_dataframe().unwrap(),
        industry.estimates_dataframe().unwrap(),
        sector.estimates_dataframe().unwrap(),
    ] {
        assert_eq!(columns(&estimates), ["period:str", "growth:f64"]);
        assert_eq!(estimates.height(), 2);
    }
    let none: IndexTrend = serde_json::from_value(json!({})).unwrap();
    assert_eq!(none.estimates_dataframe().unwrap().height(), 0);
    assert_eq!(
        columns(&index.to_dataframe().unwrap()),
        ["max_age:i64", "symbol:str", "pe_ratio:f64", "peg_ratio:f64"]
    );
}

#[test]
fn market_summary_spark_columns() {
    use crate::{MarketSummaryQuote, SparkData};

    let expected = ["symbol:str", "timestamp:datetime[μs, UTC]", "close:f64"];
    let row: SparkData = serde_json::from_value(
        json!({"symbol": "^GSPC", "timestamp": [1, 2], "close": [1.5, null]}),
    )
    .unwrap();
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("close").unwrap().null_count(), 1);
    assert_eq!(df.column("symbol").unwrap().null_count(), 0);

    let empty = SparkData::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);

    let quote: MarketSummaryQuote = serde_json::from_value(json!({
        "symbol": "^GSPC",
        "spark": {"timestamp": [1, 2], "close": [1.5, 2.5]}
    }))
    .unwrap();
    let flat = columns(&quote.to_dataframe().unwrap());
    assert!(!flat.iter().any(|c| c.starts_with("spark")));
    let nested = quote.to_nested_dataframe().unwrap();
    assert_eq!(nested.height(), 1);
    assert!(columns(&nested).iter().any(|c| c.starts_with("spark:list")));
    assert_eq!(quote.spark_dataframe().unwrap().height(), 2);
}

#[test]
fn market_summary_quote_time_is_a_datetime() {
    use crate::MarketSummaryQuote;

    let quote: MarketSummaryQuote = serde_json::from_value(json!({
        "symbol": "^GSPC",
        "regularMarketTime": {"raw": 1794603601, "fmt": "4:00PM EST"}
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
fn market_time_parses_the_rfc3339_time() {
    use crate::market::hours::MarketTime;

    let rows: Vec<MarketTime> = serde_json::from_value(json!([
        {"id": "us", "name": "U.S. markets", "status": "closed", "time": "2026-03-22T22:07:54Z"},
        {"id": "jp", "name": "Japan", "status": "open", "time": "not a time"}
    ]))
    .unwrap();
    let df = MarketTime::vec_to_dataframe(&rows).unwrap();
    let micros: Vec<_> = df
        .column("time")
        .unwrap()
        .datetime()
        .unwrap()
        .phys
        .iter()
        .collect();
    assert_eq!(micros, [Some(1_774_217_274_000_000), None]);
}
