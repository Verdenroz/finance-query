//! Every type named in a capability trait signature must be nameable from
//! outside the crate, or that trait cannot be implemented downstream.
//!
//! This file exists to fail compilation, not to assert at runtime. It uses no
//! feature gate deliberately: a type reachable only under `full` is unusable
//! for anyone depending on the crate with default features.

use finance_query::quote::{
    AssetProfile, CalendarEvents, CompanyOfficer, DefaultKeyStatistics, Earnings, EarningsHistory,
    EarningsTrend, EquityPerformance, FinancialData, FundOwnership, FundPerformance, FundProfile,
    IndexTrend, IndustryTrend, InsiderHolders, InsiderTransactions, InstitutionOwnership,
    MajorHoldersBreakdown, NetSharePurchaseActivity, Price, QuoteTypeData, RecommendationTrend,
    SecFilings, SectorTrend, SummaryDetail, SummaryProfile, TopHoldings, UpgradeDowngradeHistory,
};
use finance_query::{
    CapitalGain, ChartEvents, Dividend, IndustryPe, MoverDirection, MoverQuote,
    QuoteSummaryResponse, SectorPe, SectorPerformance, SectorPerformanceHistory, Split,
};

#[test]
fn quote_summary_modules_are_nameable() {
    fn accepts(response: &QuoteSummaryResponse) -> Option<&Price> {
        response.price.as_ref()
    }
    let mut response = QuoteSummaryResponse::default();
    response.symbol = "AAPL".to_string();
    assert!(accepts(&response).is_none());

    let _: Option<&AssetProfile> = response.asset_profile.as_ref();
    let _: Option<&CalendarEvents> = response.calendar_events.as_ref();
    let _: Option<&DefaultKeyStatistics> = response.default_key_statistics.as_ref();
    let _: Option<&Earnings> = response.earnings.as_ref();
    let _: Option<&EarningsHistory> = response.earnings_history.as_ref();
    let _: Option<&EarningsTrend> = response.earnings_trend.as_ref();
    let _: Option<&EquityPerformance> = response.equity_performance.as_ref();
    let _: Option<&FinancialData> = response.financial_data.as_ref();
    let _: Option<&FundOwnership> = response.fund_ownership.as_ref();
    let _: Option<&FundPerformance> = response.fund_performance.as_ref();
    let _: Option<&FundProfile> = response.fund_profile.as_ref();
    let _: Option<&IndexTrend> = response.index_trend.as_ref();
    let _: Option<&IndustryTrend> = response.industry_trend.as_ref();
    let _: Option<&InsiderHolders> = response.insider_holders.as_ref();
    let _: Option<&InsiderTransactions> = response.insider_transactions.as_ref();
    let _: Option<&InstitutionOwnership> = response.institution_ownership.as_ref();
    let _: Option<&MajorHoldersBreakdown> = response.major_holders_breakdown.as_ref();
    let _: Option<&NetSharePurchaseActivity> = response.net_share_purchase_activity.as_ref();
    let _: Option<&QuoteTypeData> = response.quote_type.as_ref();
    let _: Option<&RecommendationTrend> = response.recommendation_trend.as_ref();
    let _: Option<&SecFilings> = response.sec_filings.as_ref();
    let _: Option<&SectorTrend> = response.sector_trend.as_ref();
    let _: Option<&SummaryDetail> = response.summary_detail.as_ref();
    let _: Option<&SummaryProfile> = response.summary_profile.as_ref();
    let _: Option<&TopHoldings> = response.top_holdings.as_ref();
    let _: Option<&UpgradeDowngradeHistory> = response.upgrade_downgrade_history.as_ref();

    fn officers(profile: &AssetProfile) -> &Vec<CompanyOfficer> {
        &profile.company_officers
    }
    let _ = officers as fn(&AssetProfile) -> &Vec<CompanyOfficer>;
}

/// `MarketProvider` is compiled unconditionally, so its return types must be
/// nameable unconditionally too.
#[test]
fn market_provider_return_types_are_nameable() {
    fn accepts(
        _movers: Vec<MoverQuote>,
        _direction: MoverDirection,
        _sectors: Vec<SectorPerformance>,
        _history: Vec<SectorPerformanceHistory>,
        _sector_pe: Vec<SectorPe>,
        _industry_pe: Vec<IndustryPe>,
    ) {
    }
    let _ = accepts
        as fn(
            Vec<MoverQuote>,
            MoverDirection,
            Vec<SectorPerformance>,
            Vec<SectorPerformanceHistory>,
            Vec<SectorPe>,
            Vec<IndustryPe>,
        );
}

/// `CorporateProvider::fetch_events` returns [`ChartEvents`], so a downstream
/// implementation has to build one out of public values.
#[test]
fn chart_events_is_constructible_from_public_values() {
    let dividend: Dividend = serde_json::from_value(serde_json::json!({
        "timestamp": 1_700_000_000_i64,
        "amount": 0.24,
    }))
    .expect("Dividend");
    let split: Split = serde_json::from_value(serde_json::json!({
        "timestamp": 1_600_000_000_i64,
        "numerator": 4.0,
        "denominator": 1.0,
        "ratio": "4:1",
    }))
    .expect("Split");
    let gain: CapitalGain = serde_json::from_value(serde_json::json!({
        "timestamp": 1_500_000_000_i64,
        "amount": 1.5,
    }))
    .expect("CapitalGain");

    let events = ChartEvents::from_parts(vec![dividend], vec![split], vec![gain]);
    assert_eq!(events.to_dividends().len(), 1);
    assert_eq!(events.to_splits()[0].ratio, "4:1");
    assert_eq!(events.to_capital_gains()[0].amount, 1.5);
}

#[test]
fn from_parts_sorts_by_timestamp() {
    let later: Dividend = serde_json::from_value(serde_json::json!({
        "timestamp": 200_i64, "amount": 2.0,
    }))
    .expect("Dividend");
    let earlier: Dividend = serde_json::from_value(serde_json::json!({
        "timestamp": 100_i64, "amount": 1.0,
    }))
    .expect("Dividend");

    let events = ChartEvents::from_parts(vec![later, earlier], Vec::new(), Vec::new());
    let timestamps: Vec<i64> = events.to_dividends().iter().map(|d| d.timestamp).collect();
    assert_eq!(timestamps, [100, 200]);
}
