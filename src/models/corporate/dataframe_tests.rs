use super::governance::{EmployeeCount, ExecutiveCompensation};
use super::press_release::PressRelease;
use super::transcript::*;
use super::*;
use crate::models::corporate::asset_profile::CompanyOfficer;
use crate::models::corporate::earnings::Earnings;
use crate::models::corporate::earnings::{
    QuarterlyEarnings, QuarterlyFinancials, YearlyFinancials,
};
use crate::models::corporate::earnings_transcript::EarningsTranscript;
use crate::models::corporate::fund_performance::{
    AnnualReturn, AnnualTotalReturns, RiskOverviewStatisticsCat, RiskStatistic,
};
use crate::models::corporate::sec_filings::{SecExhibit, SecFiling};
use crate::models::corporate::top_holdings::{BondRating, EquityHoldings, Holding};
use crate::models::dataframe_test_util::columns;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn parse<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn fv(raw: f64) -> Value {
    json!({"raw": raw, "fmt": raw.to_string()})
}

fn fvi(raw: i64) -> Value {
    json!({"raw": raw, "fmt": raw.to_string()})
}

#[test]
fn earnings_history_wrapper_frame() {
    let history: EarningsHistory = parse(json!({
        "history": [{"period": "-1q", "epsActual": fv(1.5), "quarter": {"raw": 1700000000}}]
    }));
    let df = history.to_dataframe().unwrap();

    assert_eq!(
        columns(&df),
        [
            "max_age:i64",
            "quarter:i64",
            "period:str",
            "currency:str",
            "eps_actual:f64",
            "eps_estimate:f64",
            "eps_difference:f64",
            "surprise_percent:f64",
        ]
    );
    assert_eq!(df.height(), 1);
    assert_eq!(
        parse::<EarningsHistory>(json!({}))
            .to_dataframe()
            .unwrap()
            .height(),
        0
    );
}

#[test]
fn earnings_row_frames() {
    let quarterly: Vec<QuarterlyEarnings> = parse(json!([{"date": "1Q2024", "actual": fv(1.0)}]));
    assert_eq!(
        columns(&QuarterlyEarnings::vec_to_dataframe(&quarterly).unwrap()),
        [
            "date:str",
            "actual:f64",
            "estimate:f64",
            "difference:str",
            "surprise_pct:str",
            "calendar_quarter:str",
            "fiscal_quarter:str",
        ]
    );

    let yearly: Vec<YearlyFinancials> = parse(json!([{"date": 2024, "revenue": fvi(1)}]));
    assert_eq!(
        columns(&YearlyFinancials::vec_to_dataframe(&yearly).unwrap()),
        ["date:i64", "revenue:i64", "earnings:i64"]
    );

    let quarters: Vec<QuarterlyFinancials> = parse(json!([{"date": "1Q2024"}]));
    assert_eq!(
        columns(&QuarterlyFinancials::vec_to_dataframe(&quarters).unwrap()),
        ["date:str", "revenue:i64", "earnings:i64"]
    );
}

#[test]
fn earnings_trend_flattens_estimates() {
    let trend: EarningsTrend = parse(json!({
        "trend": [{"endDate": "2024-12-31", "earningsEstimate": {"avg": fv(2.0)}}]
    }));
    let df = trend.to_dataframe().unwrap();
    let cols = columns(&df);

    assert_eq!(cols[0], "end_date:str");
    assert!(cols.contains(&"earnings_estimate_avg:f64".to_string()));
    assert!(cols.contains(&"revenue_estimate_number_of_analysts:i64".to_string()));
    assert!(cols.contains(&"eps_trend_seven_days_ago:f64".to_string()));
    assert!(cols.contains(&"eps_revisions_up_last7days:i64".to_string()));
    assert_eq!(df.height(), 1);
    assert_eq!(
        parse::<EarningsTrend>(json!({}))
            .to_dataframe()
            .unwrap()
            .width(),
        df.width()
    );
}

#[test]
fn ownership_wrapper_frames() {
    let owners = json!({"ownershipList": [{"organization": "Vanguard", "pctHeld": fv(0.08)}]});
    let expected = [
        "max_age:i64",
        "organization:str",
        "pct_held:f64",
        "position:i64",
        "value:i64",
        "pct_change:f64",
        "report_date:i64",
    ];
    let funds: FundOwnership = parse(owners.clone());
    assert_eq!(columns(&funds.to_dataframe().unwrap()), expected);
    let institutions: InstitutionOwnership = parse(owners);
    assert_eq!(columns(&institutions.to_dataframe().unwrap()), expected);
    assert_eq!(
        parse::<FundOwnership>(json!({}))
            .to_dataframe()
            .unwrap()
            .height(),
        0
    );
}

#[test]
fn insider_wrapper_frames() {
    let holders: InsiderHolders = parse(json!({"holders": [{"name": "Cook"}]}));
    let df = holders.to_dataframe().unwrap();
    assert_eq!(
        columns(&df)[..4],
        ["max_age:i64", "name:str", "relation:str", "url:str"]
    );
    assert_eq!(df.height(), 1);

    let transactions: InsiderTransactions =
        parse(json!({"transactions": [{"filerName": "Cook", "shares": fvi(10)}]}));
    let df = transactions.to_dataframe().unwrap();
    let cols = columns(&df);
    assert!(cols.contains(&"filer_name:str".to_string()));
    assert!(cols.contains(&"shares:i64".to_string()));
    assert_eq!(
        parse::<InsiderTransactions>(json!({}))
            .to_dataframe()
            .unwrap()
            .width(),
        df.width()
    );
}

#[test]
fn recommendation_and_grade_wrapper_frames() {
    let trend: RecommendationTrend =
        parse(json!({"trend": [{"period": "0m", "strongBuy": 5, "hold": 2}]}));
    assert_eq!(
        columns(&trend.to_dataframe().unwrap()),
        [
            "period:str",
            "strong_buy:i32",
            "buy:i32",
            "hold:i32",
            "sell:i32",
            "strong_sell:i32",
        ]
    );

    let grades: UpgradeDowngradeHistory =
        parse(json!({"history": [{"firm": "GS", "currentPriceTarget": 200.0}]}));
    let cols = columns(&grades.to_dataframe().unwrap());
    assert_eq!(cols[..2], ["epoch_grade_date:i64", "firm:str"]);
    assert!(cols.contains(&"current_price_target:f64".to_string()));
}

#[test]
fn sec_filings_wrapper_skips_exhibit_list() {
    let filings: SecFilings = parse(json!({
        "filings": [{"type": "10-K", "epochDate": 1, "exhibits": [{"type": "EX-1", "url": "u"}]}]
    }));
    assert_eq!(
        columns(&filings.to_dataframe().unwrap()),
        [
            "max_age:i64",
            "date:str",
            "epoch_date:i64",
            "filing_type:str",
            "title:str",
            "edgar_url:str",
        ]
    );

    let exhibits: Vec<SecExhibit> = parse(json!([{"type": "EX-1"}]));
    assert_eq!(
        columns(&SecExhibit::vec_to_dataframe(&exhibits).unwrap()),
        ["exhibit_type:str", "url:str"]
    );
}

#[test]
fn single_row_quote_summary_frames() {
    let holders: MajorHoldersBreakdown =
        parse(json!({"insidersPercentHeld": fv(0.02), "institutionsCount": fvi(5)}));
    assert_eq!(
        columns(&holders.to_dataframe().unwrap()),
        [
            "insiders_percent_held:f64",
            "institutions_count:i64",
            "institutions_float_percent_held:f64",
            "institutions_percent_held:f64",
            "max_age:i64",
        ]
    );

    let activity: NetSharePurchaseActivity = parse(json!({"period": "6m", "buyInfoCount": fvi(3)}));
    let df = activity.to_dataframe().unwrap();
    assert_eq!(columns(&df)[..2], ["period:str", "buy_info_count:i64"]);
    assert_eq!(df.height(), 1);

    let profile: SummaryProfile = parse(json!({"city": "Cupertino", "fullTimeEmployees": 5}));
    let cols = columns(&profile.to_dataframe().unwrap());
    assert!(cols.contains(&"city:str".to_string()));
    assert!(cols.contains(&"full_time_employees:i64".to_string()));
    assert!(!cols.iter().any(|c| c.starts_with("company_officers")));
}

#[test]
fn asset_profile_and_officer_frames() {
    let profile: AssetProfile = parse(json!({"sector": "Tech", "auditRisk": 3}));
    let cols = columns(&profile.to_dataframe().unwrap());
    assert!(cols.contains(&"sector:str".to_string()));
    assert!(cols.contains(&"audit_risk:i32".to_string()));
    assert!(!cols.iter().any(|c| c.starts_with("company_officers")));

    let officers: Vec<CompanyOfficer> =
        parse(json!([{"name": "Cook", "age": 60, "totalPay": fvi(1)}]));
    let cols = columns(&CompanyOfficer::vec_to_dataframe(&officers).unwrap());
    assert!(cols.contains(&"age:i32".to_string()));
    assert!(cols.contains(&"total_pay:i64".to_string()));
}

#[test]
fn equity_performance_flattens_overviews() {
    let perf: EquityPerformance = parse(json!({
        "benchmark": {"symbol": "^GSPC"},
        "performanceOverview": {"oneMonthReturn": fv(0.01)}
    }));
    let df = perf.to_dataframe().unwrap();
    let cols = columns(&df);

    assert_eq!(cols[0], "max_age:i64");
    assert!(cols.contains(&"benchmark_symbol:str".to_string()));
    assert!(cols.contains(&"performance_overview_one_month_return:f64".to_string()));
    assert!(cols.contains(&"performance_overview_benchmark_max_return:f64".to_string()));
    assert_eq!(df.height(), 1);
}

#[test]
fn calendar_events_flattens_earnings() {
    let events: CalendarEvents = parse(json!({
        "exDividendDate": {"raw": 1700000000},
        "earnings": {"earningsAverage": fv(1.5), "revenueLow": fvi(10)}
    }));
    let df = events.to_dataframe().unwrap();
    let cols = columns(&df);

    assert!(cols.contains(&"ex_dividend_date:i64".to_string()));
    assert!(cols.contains(&"earnings_earnings_average:f64".to_string()));
    assert!(cols.contains(&"earnings_revenue_low:i64".to_string()));
    assert_eq!(df.height(), 1);
}

#[test]
fn fund_performance_flattens_return_blocks() {
    let perf: FundPerformance = parse(json!({
        "fundCategoryName": "Large Blend",
        "trailingReturns": {"ytd": fv(0.1)},
        "trailingReturnsCat": {"ytd": fv(0.2)}
    }));
    let df = perf.to_dataframe().unwrap();
    let cols = columns(&df);

    assert!(cols.contains(&"fund_category_name:str".to_string()));
    assert!(cols.contains(&"performance_overview_ytd_return_pct:f64".to_string()));
    assert!(cols.contains(&"trailing_returns_ytd:f64".to_string()));
    assert!(cols.contains(&"trailing_returns_nav_ytd:f64".to_string()));
    assert!(cols.contains(&"trailing_returns_cat_last_bull_mkt:f64".to_string()));
    assert_eq!(df.height(), 1);

    let annual: Vec<AnnualReturn> = parse(json!([{"year": "2023", "annualValue": fv(0.2)}]));
    assert_eq!(
        columns(&AnnualReturn::vec_to_dataframe(&annual).unwrap()),
        ["year:str", "annual_value:f64"]
    );
    let risk: Vec<RiskStatistic> = parse(json!([{"year": "5y", "alpha": fv(1.0)}]));
    let cols = columns(&RiskStatistic::vec_to_dataframe(&risk).unwrap());
    assert_eq!(cols[..2], ["year:str", "alpha:f64"]);
    assert_eq!(cols.len(), 8);
}

#[test]
fn fund_profile_flattens_fee_blocks() {
    let profile: FundProfile = parse(json!({
        "family": "Vanguard",
        "managementInfo": {"managerName": "Jane"},
        "feesExpensesInvestment": {"annualReportExpenseRatio": fv(0.0003)}
    }));
    let df = profile.to_dataframe().unwrap();
    let cols = columns(&df);

    assert!(cols.contains(&"family:str".to_string()));
    assert!(cols.contains(&"management_info_manager_name:str".to_string()));
    assert!(cols.contains(&"fees_expenses_investment_annual_report_expense_ratio:f64".to_string()));
    assert!(cols.contains(&"fees_expenses_investment_cat_total_net_assets:f64".to_string()));
    assert!(!cols.iter().any(|c| c.contains("projection_values")));
    assert_eq!(cols.last().unwrap(), "brokerages:list[str]");
    assert_eq!(df.height(), 1);
}

#[test]
fn top_holdings_flattens_blocks_and_rows_have_frames() {
    let top: TopHoldings = parse(json!({
        "cashPosition": fv(0.01),
        "holdings": [{"symbol": "AAPL"}],
        "equityHoldings": {"priceToEarnings": fv(20.0)},
        "sectorWeightings": [{"technology": fv(0.3)}]
    }));
    let df = top.to_dataframe().unwrap();
    let cols = columns(&df);

    assert!(cols.contains(&"cash_position:f64".to_string()));
    assert!(cols.contains(&"equity_holdings_price_to_earnings:f64".to_string()));
    assert!(cols.contains(&"sector_weightings_technology:f64".to_string()));
    assert!(!cols.iter().any(|c| c.starts_with("holdings")));
    assert_eq!(df.height(), 1);

    let holdings: Vec<Holding> = parse(json!([{"symbol": "AAPL", "holdingPercent": fv(0.1)}]));
    assert_eq!(
        columns(&Holding::vec_to_dataframe(&holdings).unwrap()),
        ["symbol:str", "holding_name:str", "holding_percent:f64"]
    );
    let equity: Vec<EquityHoldings> = parse(json!([{}]));
    assert!(EquityHoldings::vec_to_dataframe(&equity).unwrap().width() > 0);
    let ratings: Vec<BondRating> = parse(json!([{"aaa": fv(0.5)}]));
    let cols = columns(&BondRating::vec_to_dataframe(&ratings).unwrap());
    assert_eq!(cols[..2], ["us_government:f64", "aaa:f64"]);
}

#[test]
fn record_frames() {
    let releases: Vec<PressRelease> = parse(json!([{"symbol": "AAPL", "title": "t"}]));
    assert_eq!(
        columns(&PressRelease::vec_to_dataframe(&releases).unwrap()),
        ["symbol:str", "date:str", "title:str", "text:str"]
    );

    let transcripts: Vec<EarningsTranscript> =
        parse(json!([{"symbol": "AAPL", "year": 2024, "text": "hello"}]));
    assert_eq!(
        columns(&EarningsTranscript::vec_to_dataframe(&transcripts).unwrap()),
        [
            "symbol:str",
            "quarter:str",
            "year:i32",
            "date:str",
            "text:str",
        ]
    );

    let pay: Vec<ExecutiveCompensation> =
        parse(json!([{"name_and_position": "Cook", "year": 2024, "total": 1.0}]));
    let cols = columns(&ExecutiveCompensation::vec_to_dataframe(&pay).unwrap());
    assert_eq!(cols.len(), 14);
    assert!(cols.contains(&"year:i32".to_string()));
    assert!(cols.contains(&"total:f64".to_string()));

    let staff: Vec<EmployeeCount> = parse(json!([{"employee_count": 100}]));
    let cols = columns(&EmployeeCount::vec_to_dataframe(&staff).unwrap());
    assert_eq!(cols.len(), 8);
    assert!(cols.contains(&"employee_count:i64".to_string()));
}

#[test]
fn transcript_segment_frames() {
    let words: Vec<Word> = parse(json!([{
        "word": "a", "punctuated_word": "A", "start": 0.0, "end": 1.0, "confidence": 0.9
    }]));
    assert_eq!(
        columns(&Word::vec_to_dataframe(&words).unwrap()),
        [
            "word:str",
            "punctuated_word:str",
            "start:f64",
            "end:f64",
            "confidence:f64",
        ]
    );

    let sentences: Vec<Sentence> =
        parse(json!([{"start": 0.0, "end": 1.0, "text": "A.", "words": []}]));
    assert_eq!(
        columns(&Sentence::vec_to_dataframe(&sentences).unwrap()),
        ["start:f64", "end:f64", "text:str"]
    );

    let paragraphs: Vec<Paragraph> = parse(json!([{
        "speaker": 1, "start": 0.0, "end": 1.0, "text": "A.", "sentences": []
    }]));
    let cols = columns(&Paragraph::vec_to_dataframe(&paragraphs).unwrap());
    assert_eq!(
        cols[..4],
        ["speaker:i32", "start:f64", "end:f64", "text:str"]
    );
}

#[test]
fn transcript_speaker_and_metadata_frames() {
    let speakers: Vec<SpeakerMapping> = parse(json!([{
        "speaker": 1, "speaker_data": {"name": "Cook", "role": "CEO"}
    }]));
    assert_eq!(
        columns(&SpeakerMapping::vec_to_dataframe(&speakers).unwrap()),
        [
            "speaker:i32",
            "speaker_data_company:str",
            "speaker_data_name:str",
            "speaker_data_role:str",
        ]
    );

    let metadata: Vec<TranscriptMetadata> = parse(json!([{
        "date": 1, "eventId": 2, "eventType": "call", "fiscalPeriod": "Q1", "fiscalYear": 2024,
        "isLatest": true, "s3Url": "u", "title": "t", "transcriptId": 3, "type": "x", "updated": 4
    }]));
    assert_eq!(
        columns(&TranscriptMetadata::vec_to_dataframe(&metadata).unwrap()),
        [
            "date:i64",
            "event_id:i64",
            "event_type:str",
            "fiscal_period:str",
            "fiscal_year:i32",
            "is_latest:bool",
            "s3_url:str",
            "title:str",
            "transcript_id:i64",
            "transcript_type:str",
            "updated:i64",
        ]
    );

    let empty = TranscriptWithMeta::vec_to_dataframe(&[]).unwrap();
    assert_eq!(
        columns(&empty),
        [
            "event_id:str",
            "quarter:str",
            "year:i32",
            "title:str",
            "url:str"
        ]
    );
}

#[test]
fn flat_frames_of_nested_types_keep_their_exact_columns() {
    let profile: AssetProfile = parse(json!({"companyOfficers": [{"name": "Cook"}]}));
    assert_eq!(
        columns(&profile.to_dataframe().unwrap()),
        [
            "address1:str",
            "address2:str",
            "address3:str",
            "city:str",
            "state:str",
            "zip:str",
            "country:str",
            "phone:str",
            "fax:str",
            "website:str",
            "industry:str",
            "industry_key:str",
            "industry_disp:str",
            "sector:str",
            "sector_key:str",
            "sector_disp:str",
            "long_business_summary:str",
            "full_time_employees:i64",
            "audit_risk:i32",
            "board_risk:i32",
            "compensation_risk:i32",
            "shareholder_rights_risk:i32",
            "overall_risk:i32",
            "governance_epoch_date:i64",
            "compensation_as_of_epoch_date:i64",
            "max_age:i64",
        ]
    );

    let top: TopHoldings = parse(json!({"holdings": [{"symbol": "AAPL"}]}));
    assert_eq!(
        columns(&top.to_dataframe().unwrap()),
        [
            "max_age:i64",
            "cash_position:f64",
            "stock_position:f64",
            "bond_position:f64",
            "other_position:f64",
            "preferred_position:f64",
            "convertible_position:f64",
            "equity_holdings_price_to_earnings:f64",
            "equity_holdings_price_to_book:f64",
            "equity_holdings_price_to_sales:f64",
            "equity_holdings_price_to_cashflow:f64",
            "sector_weightings_realestate:f64",
            "sector_weightings_consumer_cyclical:f64",
            "sector_weightings_basic_materials:f64",
            "sector_weightings_consumer_defensive:f64",
            "sector_weightings_technology:f64",
            "sector_weightings_communication_services:f64",
            "sector_weightings_financial_services:f64",
            "sector_weightings_utilities:f64",
            "sector_weightings_industrials:f64",
            "sector_weightings_energy:f64",
            "sector_weightings_healthcare:f64",
        ]
    );

    let filing: SecFiling = parse(json!({"exhibits": [{"type": "EX-1"}]}));
    assert_eq!(
        columns(&filing.to_dataframe().unwrap()),
        [
            "max_age:i64",
            "date:str",
            "epoch_date:i64",
            "filing_type:str",
            "title:str",
            "edgar_url:str",
        ]
    );
}

#[test]
fn nested_row_fields_have_child_frames() {
    let profile: AssetProfile = parse(json!({"companyOfficers": [{"name": "Cook", "age": 60}]}));
    let officers = profile.company_officers_dataframe().unwrap();
    assert_eq!(officers.height(), 1);
    assert!(columns(&officers).contains(&"age:i32".to_string()));

    let top: TopHoldings = parse(json!({
        "holdings": [{"symbol": "AAPL"}, {"symbol": "MSFT"}],
        "bondRatings": [{"aaa": fv(0.5)}]
    }));
    assert_eq!(top.holdings_dataframe().unwrap().height(), 2);
    assert_eq!(top.bond_ratings_dataframe().unwrap().height(), 1);
    let none: TopHoldings = parse(json!({}));
    let empty = none.holdings_dataframe().unwrap();
    assert_eq!(empty.height(), 0);
    assert_eq!(columns(&empty), columns(&top.holdings_dataframe().unwrap()));

    let filing: SecFiling = parse(json!({"exhibits": [{"type": "EX-1", "url": "u"}]}));
    assert_eq!(
        columns(&filing.exhibits_dataframe().unwrap()),
        ["exhibit_type:str", "url:str"]
    );
}

#[test]
fn nested_frame_holds_rows_as_list_of_struct() {
    let filing: SecFiling = parse(json!({"exhibits": [{"type": "EX-1", "url": "u"}]}));
    let df = filing.to_nested_dataframe().unwrap();
    assert_eq!(columns(&df).last().unwrap(), "exhibits:list[struct[2]]");
    assert_eq!(df.height(), 1);
}

#[test]
fn transcript_rows_nest_sentences_and_words() {
    let paragraph: Paragraph = parse(json!({
        "speaker": 1, "text": "A b.",
        "sentences": [{"text": "A b.", "words": [{"word": "A"}, {"word": "b"}]}]
    }));
    let sentences = paragraph.sentences_dataframe().unwrap();
    assert_eq!(columns(&sentences), ["start:f64", "end:f64", "text:str"]);
    assert_eq!(sentences.height(), 1);

    let sentence = &paragraph.sentences[0];
    let words = sentence.words_dataframe().unwrap();
    assert_eq!(words.height(), 2);
    assert_eq!(columns(&words)[0], "word:str");
}

#[test]
fn fund_performance_nests_its_row_wrappers() {
    let perf: FundPerformance = parse(json!({
        "annualTotalReturns": {"returns": [{"year": "2023", "annualValue": fv(0.2)}]},
        "riskOverviewStatistics": {"riskStatistics": [{"year": "5y"}]}
    }));
    let annual = perf.annual_total_returns_dataframe().unwrap();
    assert_eq!(
        columns(&annual),
        ["returns:list[struct[2]]", "returns_cat:list[struct[2]]"]
    );
    assert_eq!(annual.height(), 1);
    let returns = perf
        .annual_total_returns
        .as_ref()
        .unwrap()
        .returns_dataframe()
        .unwrap();
    assert_eq!(columns(&returns), ["year:str", "annual_value:f64"]);

    let risk = perf.risk_overview_statistics_dataframe().unwrap();
    assert_eq!(columns(&risk), ["risk_statistics:list[struct[8]]"]);
    let cols = columns(&perf.to_nested_dataframe().unwrap());
    assert!(
        cols.iter()
            .any(|c| c.starts_with("annual_total_returns:list[struct["))
    );
    assert!(
        !columns(&perf.to_dataframe().unwrap())
            .iter()
            .any(|c| c.starts_with("annual_total_returns"))
    );
}

#[test]
fn earnings_wrappers_expose_their_rows() {
    let earnings: Earnings = parse(json!({
        "defaultMethodology": "gaap",
        "earningsChart": {"quarterly": [{"date": "1Q2024", "actual": fv(1.0)}]},
        "financialsChart": {
            "yearly": [{"date": 2023, "revenue": fvi(5)}],
            "quarterly": [{"date": "1Q2024"}, {"date": "2Q2024"}]
        }
    }));
    assert_eq!(
        columns(&earnings.to_dataframe().unwrap()),
        ["default_methodology:str"]
    );

    let chart = earnings.earnings_chart_dataframe().unwrap();
    assert_eq!(chart.height(), 1);
    assert_eq!(columns(&chart).last().unwrap(), "quarterly:list[struct[7]]");
    assert!(
        !columns(&chart)
            .iter()
            .any(|c| c.starts_with("earnings_date"))
    );
    let quarterly = earnings
        .earnings_chart
        .as_ref()
        .unwrap()
        .quarterly_dataframe()
        .unwrap();
    assert_eq!(columns(&quarterly)[..2], ["date:str", "actual:f64"]);

    let financials = earnings.financials_chart_dataframe().unwrap();
    assert_eq!(
        columns(&financials),
        ["yearly:list[struct[3]]", "quarterly:list[struct[3]]"]
    );
    let chart = earnings.financials_chart.as_ref().unwrap();
    assert_eq!(chart.yearly_dataframe().unwrap().height(), 1);
    assert_eq!(chart.quarterly_dataframe().unwrap().height(), 2);

    let none: Earnings = parse(json!({}));
    assert_eq!(none.earnings_chart_dataframe().unwrap().height(), 0);
}

#[test]
fn risk_overview_category_wrapper_exposes_its_rows() {
    let risk: RiskOverviewStatisticsCat =
        parse(json!({"riskStatisticsCat": [{"year": "3y"}, {"year": "5y"}]}));
    let df = risk.risk_statistics_cat_dataframe().unwrap();
    assert_eq!(df.height(), 2);
    assert_eq!(columns(&df)[..2], ["year:str", "alpha:f64"]);
}

#[test]
fn transcript_wrappers_nest_speakers_and_paragraphs() {
    let transcript: Transcript = parse(json!({
        "transcriptContent": {
            "company_id": 1,
            "event_id": 2,
            "speaker_mapping": [{"speaker": 1, "speaker_data": {"name": "Cook"}}],
            "transcript": {"paragraphs": [{"speaker": 1, "text": "Hi."}, {"speaker": 2, "text": "Bye."}]}
        },
        "transcriptMetadata": {
            "date": 1, "eventId": 2, "eventType": "call", "fiscalPeriod": "Q1", "fiscalYear": 2024,
            "isLatest": true, "s3Url": "u", "title": "t", "transcriptId": 3, "type": "x", "updated": 4
        }
    }));
    let flat = columns(&transcript.to_dataframe().unwrap());
    assert_eq!(flat[0], "transcript_metadata_date:i64");
    assert_eq!(flat.len(), 11);

    let content = transcript.transcript_content_dataframe().unwrap();
    assert_eq!(
        columns(&content),
        [
            "company_id:i64",
            "event_id:i64",
            "version:str",
            "speaker_mapping:list[struct[4]]",
            "transcript:list[struct[3]]",
        ]
    );
    let content = &transcript.transcript_content;
    let speakers = content.speaker_mapping_dataframe().unwrap();
    assert_eq!(columns(&speakers)[0], "speaker:i32");

    let data = content.transcript_dataframe().unwrap();
    let cols = columns(&data);
    assert_eq!(cols[..2], ["number_of_speakers:i32", "text:str"]);
    assert!(cols[2].starts_with("paragraphs:list[struct["));
    let paragraphs = content
        .transcript
        .as_ref()
        .unwrap()
        .paragraphs_dataframe()
        .unwrap();
    assert_eq!(paragraphs.height(), 2);
}
