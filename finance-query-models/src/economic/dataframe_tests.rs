use super::*;
use crate::models::dataframe_test_util::columns;
use serde_json::json;

#[test]
fn series_frame_prefixes_series_id() {
    let series: EconomicSeries = serde_json::from_value(json!({
        "series_id": "FEDFUNDS",
        "title": null,
        "units": null,
        "frequency": null,
        "observations": [
            {"date": "2024-01-01", "value": 5.33},
            {"date": "2024-02-01", "value": null}
        ]
    }))
    .unwrap();
    let df = series.to_dataframe().unwrap();

    assert_eq!(columns(&df), ["series_id:str", "date:date", "value:f64"]);
    assert_eq!(df.height(), 2);
    assert_eq!(df.column("value").unwrap().null_count(), 1);
}

#[test]
fn series_frame_empty_keeps_schema() {
    let series: EconomicSeries = serde_json::from_value(json!({
        "series_id": "FEDFUNDS",
        "title": null,
        "units": null,
        "frequency": null,
        "observations": []
    }))
    .unwrap();
    let df = series.to_dataframe().unwrap();

    assert_eq!(columns(&df), ["series_id:str", "date:date", "value:f64"]);
    assert_eq!(df.height(), 0);
}

#[test]
fn catalog_frames_columns() {
    let matches: Vec<EconomicSeriesMatch> =
        serde_json::from_value(json!([{"id": "GDPC1", "popularity": 90}])).unwrap();
    assert_eq!(
        columns(&EconomicSeriesMatch::vec_to_dataframe(&matches).unwrap()),
        [
            "id:str",
            "title:str",
            "frequency:str",
            "units:str",
            "seasonal_adjustment:str",
            "observation_start:date",
            "observation_end:date",
            "popularity:i64",
            "notes:str",
        ]
    );

    let categories: Vec<EconomicCategory> =
        serde_json::from_value(json!([{"id": 1, "parent_id": 0}])).unwrap();
    assert_eq!(
        columns(&EconomicCategory::vec_to_dataframe(&categories).unwrap()),
        ["id:i64", "name:str", "parent_id:i64"]
    );

    let releases: Vec<EconomicRelease> =
        serde_json::from_value(json!([{"id": 1, "press_release": true}])).unwrap();
    assert_eq!(
        columns(&EconomicRelease::vec_to_dataframe(&releases).unwrap()),
        ["id:i64", "name:str", "press_release:bool", "link:str"]
    );
}

#[cfg(feature = "fred")]
#[test]
fn macro_series_and_yield_frames() {
    let series: MacroSeries = serde_json::from_value(json!({
        "id": "DGS10",
        "observations": [{"date": "2024-01-02", "value": 3.9}]
    }))
    .unwrap();
    assert_eq!(
        columns(&series.to_dataframe().unwrap()),
        ["series_id:str", "date:date", "value:f64"]
    );

    let yields: Vec<TreasuryYield> = serde_json::from_value(json!([{
        "date": "01/02/2024", "y1m": 5.5, "y2m": null, "y3m": null, "y4m": null,
        "y6m": null, "y1": null, "y2": null, "y3": null, "y5": null, "y7": null,
        "y10": 3.9, "y20": null, "y30": null
    }]))
    .unwrap();
    let df = TreasuryYield::vec_to_dataframe(&yields).unwrap();
    assert_eq!(df.width(), 14);
    assert_eq!(columns(&df)[..2], ["date:date", "y1m:f64"]);
}

#[cfg(feature = "fiscaldata")]
#[test]
fn auction_frames_columns() {
    let auctions: Vec<TreasuryAuction> = serde_json::from_value(json!([{
        "record_date": "2024-01-02", "cusip": "912797", "security_type": "Bill",
        "security_term": "13-Week", "auction_date": "2024-01-02",
        "issue_date": "2024-01-04", "maturity_date": "2024-04-04",
        "reopening": false, "offering_amt": 1.0e10
    }]))
    .unwrap();
    let df = TreasuryAuction::vec_to_dataframe(&auctions).unwrap();
    let cols = columns(&df);
    assert_eq!(df.width(), 24);
    assert_eq!(cols[0], "record_date:date");
    for date in ["auction_date", "issue_date", "maturity_date"] {
        assert!(cols.contains(&format!("{date}:date")), "{date}");
    }
    assert!(cols.contains(&"reopening:bool".to_string()));
    assert!(cols.contains(&"offering_amt:f64".to_string()));

    let upcoming: Vec<UpcomingAuction> = serde_json::from_value(json!([{
        "record_date": "2024-01-02", "security_type": "Bill", "security_term": "13-Week",
        "cusip": "912797", "announcement_date": "2024-01-01",
        "auction_date": "2024-01-02", "issue_date": "2024-01-04"
    }]))
    .unwrap();
    assert_eq!(
        columns(&UpcomingAuction::vec_to_dataframe(&upcoming).unwrap()),
        [
            "record_date:date",
            "security_type:str",
            "security_term:str",
            "cusip:str",
            "reopening:bool",
            "offering_amt:f64",
            "announcement_date:date",
            "auction_date:date",
            "issue_date:date",
        ]
    );
}
