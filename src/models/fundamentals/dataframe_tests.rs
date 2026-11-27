use crate::models::dataframe_test_util::{columns, days};
use crate::models::fundamentals::{
    CompanyProfile, DefaultKeyStatistics, EtfCountryWeighting, EtfHolding, EtfProfile,
    EtfSectorWeighting, FinancialData, GradingAction, PriceTargetConsensus, PriceTargetSummary,
    RatingConsensus, SummaryDetail,
};
use serde_json::json;

#[test]
fn company_profile_columns() {
    let row: CompanyProfile = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "name:str",
        "description:str",
        "asset_type:str",
        "exchange:str",
        "currency:str",
        "country:str",
        "sector:str",
        "industry:str",
        "market_capitalization:f64",
        "isin:str",
        "cusip:str",
        "active:bool",
        "is_etf:bool",
        "is_adr:bool",
        "is_fund:bool",
        "cik:str",
        "ipo_date:str",
        "provider_id:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <CompanyProfile>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn price_target_consensus_columns() {
    let row: PriceTargetConsensus = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "target_high:f64",
        "target_low:f64",
        "target_consensus:f64",
        "target_median:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <PriceTargetConsensus>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn price_target_summary_columns() {
    let row: PriceTargetSummary = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "last_month_count:i64",
        "last_month_avg:f64",
        "last_quarter_count:i64",
        "last_quarter_avg:f64",
        "last_year_count:i64",
        "last_year_avg:f64",
        "all_time_count:i64",
        "all_time_avg:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <PriceTargetSummary>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn rating_consensus_columns() {
    let row: RatingConsensus = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "strong_buy:i64",
        "buy:i64",
        "hold:i64",
        "sell:i64",
        "strong_sell:i64",
        "consensus:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <RatingConsensus>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn default_key_statistics_columns() {
    let row: DefaultKeyStatistics = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "week_52_change:f64",
        "sand_p_52_week_change:f64",
        "beta:f64",
        "book_value:f64",
        "category:str",
        "date_short_interest:date",
        "earnings_quarterly_growth:f64",
        "enterprise_to_ebitda:f64",
        "enterprise_to_revenue:f64",
        "enterprise_value:i64",
        "float_shares:i64",
        "forward_eps:f64",
        "forward_pe:f64",
        "fund_family:str",
        "fund_inception_date:date",
        "held_percent_insiders:f64",
        "held_percent_institutions:f64",
        "implied_shares_outstanding:i64",
        "last_dividend_date:date",
        "last_dividend_value:f64",
        "last_fiscal_year_end:date",
        "last_split_date:date",
        "last_split_factor:str",
        "latest_funding_date:date",
        "latest_share_class:str",
        "lead_investor:str",
        "legal_type:str",
        "max_age:i64",
        "most_recent_quarter:date",
        "net_income_to_common:i64",
        "next_fiscal_year_end:date",
        "price_hint:i64",
        "price_to_book:f64",
        "profit_margins:f64",
        "shares_outstanding:i64",
        "shares_percent_shares_out:f64",
        "shares_short:i64",
        "shares_short_previous_month_date:date",
        "shares_short_prior_month:i64",
        "short_percent_of_float:f64",
        "short_ratio:f64",
        "trailing_eps:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <DefaultKeyStatistics>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn financial_data_columns() {
    let row: FinancialData = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "current_price:f64",
        "current_ratio:f64",
        "debt_to_equity:f64",
        "earnings_growth:f64",
        "ebitda:i64",
        "ebitda_margins:f64",
        "financial_currency:str",
        "free_cashflow:i64",
        "gross_margins:f64",
        "gross_profits:i64",
        "max_age:i64",
        "number_of_analyst_opinions:i64",
        "operating_cashflow:i64",
        "operating_margins:f64",
        "profit_margins:f64",
        "quick_ratio:f64",
        "recommendation_key:str",
        "recommendation_mean:f64",
        "return_on_assets:f64",
        "return_on_equity:f64",
        "revenue_growth:f64",
        "revenue_per_share:f64",
        "target_high_price:f64",
        "target_low_price:f64",
        "target_mean_price:f64",
        "target_median_price:f64",
        "total_cash:i64",
        "total_cash_per_share:f64",
        "total_debt:i64",
        "total_revenue:i64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <FinancialData>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn summary_detail_columns() {
    let row: SummaryDetail = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "all_time_high:f64",
        "all_time_low:f64",
        "ask:f64",
        "ask_size:i64",
        "average_daily_volume10_day:i64",
        "average_volume:i64",
        "average_volume10days:i64",
        "beta:f64",
        "bid:f64",
        "bid_size:i64",
        "currency:str",
        "day_high:f64",
        "day_low:f64",
        "dividend_rate:f64",
        "dividend_yield:f64",
        "ex_dividend_date:date",
        "expire_date:date",
        "fifty_day_average:f64",
        "fifty_two_week_high:f64",
        "fifty_two_week_low:f64",
        "five_year_avg_dividend_yield:f64",
        "forward_pe:f64",
        "from_currency:str",
        "last_market:str",
        "market_cap:i64",
        "max_age:i64",
        "nav_price:f64",
        "open:f64",
        "payout_ratio:f64",
        "previous_close:f64",
        "price_hint:i64",
        "price_to_sales_trailing12_months:f64",
        "regular_market_day_high:f64",
        "regular_market_day_low:f64",
        "regular_market_open:f64",
        "regular_market_previous_close:f64",
        "regular_market_volume:i64",
        "start_date:date",
        "to_currency:str",
        "total_assets:i64",
        "tradeable:bool",
        "trailing_annual_dividend_rate:f64",
        "trailing_annual_dividend_yield:f64",
        "trailing_pe:f64",
        "two_hundred_day_average:f64",
        "volume:i64",
        "yield_value:f64",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <SummaryDetail>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn etf_holding_columns() {
    let row: EtfHolding = serde_json::from_value(json!({})).unwrap();
    let expected = ["symbol:str", "description:str", "weight:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EtfHolding>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn etf_sector_weighting_columns() {
    let row: EtfSectorWeighting = serde_json::from_value(json!({})).unwrap();
    let expected = ["sector:str", "weight:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EtfSectorWeighting>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn etf_country_weighting_columns() {
    let row: EtfCountryWeighting = serde_json::from_value(json!({})).unwrap();
    let expected = ["country:str", "weight:f64"];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <EtfCountryWeighting>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn grading_action_columns() {
    let row: GradingAction = serde_json::from_value(json!({})).unwrap();
    let expected = [
        "symbol:str",
        "date:str",
        "grading_company:str",
        "previous_grade:str",
        "new_grade:str",
    ];
    let df = row.to_dataframe().unwrap();
    assert_eq!(columns(&df), expected);
    assert_eq!(df.height(), 1);

    let empty = <GradingAction>::vec_to_dataframe(&[]).unwrap();
    assert_eq!(columns(&empty), expected);
    assert_eq!(empty.height(), 0);
}

#[test]
fn etf_profile_nests_holdings_and_weightings() {
    let profile: EtfProfile = serde_json::from_value(json!({
        "symbol": "SPY",
        "holdings": [{"symbol": "AAPL", "weight": 0.07}, {"symbol": "MSFT"}],
        "sector_weightings": [{"sector": "Tech", "weight": 0.3}],
        "country_weightings": []
    }))
    .unwrap();
    let flat = profile.to_dataframe().unwrap();
    assert_eq!(flat.height(), 1);
    assert!(!columns(&flat).iter().any(|c| c.contains("holdings")));

    let holdings = profile.holdings_dataframe().unwrap();
    assert_eq!(columns(&holdings)[..2], ["symbol:str", "description:str"]);
    assert_eq!(holdings.height(), 2);
    assert_eq!(profile.sector_weightings_dataframe().unwrap().height(), 1);
    let countries = profile.country_weightings_dataframe().unwrap();
    assert_eq!(countries.height(), 0);
    assert_eq!(
        columns(&countries),
        columns(&EtfCountryWeighting::vec_to_dataframe(&[]).unwrap())
    );

    let nested = columns(&profile.to_nested_dataframe().unwrap());
    assert!(
        nested
            .iter()
            .any(|c| c.starts_with("holdings:list[struct["))
    );
    assert!(
        nested
            .iter()
            .any(|c| c.starts_with("country_weightings:list[struct["))
    );
}

#[test]
fn summary_detail_epoch_dates_are_utc_days() {
    let row: SummaryDetail = serde_json::from_value(json!({
        "exDividendDate": { "raw": 1794528000, "fmt": "2026-11-09" },
        "expireDate": { "raw": 1797984000 },
        "startDate": {}
    }))
    .unwrap();
    let df = row.to_dataframe().unwrap();
    assert_eq!(days(&df, "ex_dividend_date"), Some(20_770));
    assert_eq!(days(&df, "expire_date"), Some(20_810));
    assert_eq!(days(&df, "start_date"), None);
}

#[test]
fn default_key_statistics_epoch_dates_are_utc_days() {
    let row: DefaultKeyStatistics = serde_json::from_value(json!({
        "mostRecentQuarter": { "raw": 1759017600 },
        "fundInceptionDate": { "raw": 1011657600 }
    }))
    .unwrap();
    let df = row.to_dataframe().unwrap();
    assert_eq!(days(&df, "most_recent_quarter"), Some(20_359));
    assert_eq!(days(&df, "fund_inception_date"), Some(11_709));
}
