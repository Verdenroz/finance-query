//! Compile and runtime tests for docs/library/screeners.md
//!
//! Pure (no-network) tests verify that every type, variant, method, and
//! builder pattern documented in screeners.md actually exists and works.
//! Network tests are marked `#[ignore = "requires network access"]`.
//!
//! Run with: `cargo test --test doc_screeners`
//! Run network tests: `cargo test --test doc_screeners -- --ignored`

use finance_query::{
    EquityField, EquityScreenerQuery, FundField, FundScreenerQuery, LogicalOperator, Screener,
    ScreenerFieldExt, Sector, SortType,
};

// ---------------------------------------------------------------------------
// Predefined screener variants
// ---------------------------------------------------------------------------

#[test]
fn test_screener_variants_exist() {
    // All 15 Screener variants documented in screeners.md must exist.
    let _: Screener = Screener::DayGainers;
    let _: Screener = Screener::DayLosers;
    let _: Screener = Screener::MostActives;
    let _: Screener = Screener::AggressiveSmallCaps;
    let _: Screener = Screener::ConservativeForeignFunds;
    let _: Screener = Screener::GrowthTechnologyStocks;
    let _: Screener = Screener::HighYieldBond;
    let _: Screener = Screener::MostShortedStocks;
    let _: Screener = Screener::PortfolioAnchors;
    let _: Screener = Screener::SmallCapGainers;
    let _: Screener = Screener::SolidLargeGrowthFunds;
    let _: Screener = Screener::SolidMidcapGrowthFunds;
    let _: Screener = Screener::TopMutualFunds;
    let _: Screener = Screener::UndervaluedGrowthStocks;
    let _: Screener = Screener::UndervaluedLargeCaps;
}

// ---------------------------------------------------------------------------
// Sector enum variants
// ---------------------------------------------------------------------------

#[test]
fn test_sector_variants_exist() {
    // All 11 Sector variants documented in finance.md / screeners.md must exist.
    let _: Sector = Sector::BasicMaterials;
    let _: Sector = Sector::CommunicationServices;
    let _: Sector = Sector::ConsumerCyclical;
    let _: Sector = Sector::ConsumerDefensive;
    let _: Sector = Sector::Energy;
    let _: Sector = Sector::FinancialServices;
    let _: Sector = Sector::Healthcare;
    let _: Sector = Sector::Industrials;
    let _: Sector = Sector::RealEstate;
    let _: Sector = Sector::Technology;
    let _: Sector = Sector::Utilities;
}

// ---------------------------------------------------------------------------
// EquityField — every variant documented in screeners.md must compile
// ---------------------------------------------------------------------------

#[test]
fn test_equity_field_price_and_market_cap_variants() {
    let _: EquityField = EquityField::Ticker;
    let _: EquityField = EquityField::CompanyShortName;
    let _: EquityField = EquityField::EodPrice;
    let _: EquityField = EquityField::IntradayPrice;
    let _: EquityField = EquityField::IntradayPriceChange;
    let _: EquityField = EquityField::PercentChange;
    let _: EquityField = EquityField::Lastclose52WkHigh;
    let _: EquityField = EquityField::Lastclose52WkLow;
    let _: EquityField = EquityField::FiftyTwoWkPctChange;
    let _: EquityField = EquityField::IntradayMarketCap;
    let _: EquityField = EquityField::LastcloseMarketCap;
}

#[test]
fn test_equity_field_categorical_variants() {
    let _: EquityField = EquityField::Region;
    let _: EquityField = EquityField::Sector;
    let _: EquityField = EquityField::Industry;
    let _: EquityField = EquityField::Exchange;
    let _: EquityField = EquityField::PeerGroup;
}

#[test]
fn test_equity_field_trading_variants() {
    let _: EquityField = EquityField::Beta;
    let _: EquityField = EquityField::AvgDailyVol3M;
    let _: EquityField = EquityField::DayVolume;
    let _: EquityField = EquityField::EodVolume;
    let _: EquityField = EquityField::PctHeldInsider;
    let _: EquityField = EquityField::PctHeldInst;
}

#[test]
fn test_equity_field_short_interest_variants() {
    let _: EquityField = EquityField::ShortPctFloat;
    let _: EquityField = EquityField::ShortPctSharesOut;
    let _: EquityField = EquityField::ShortInterest;
    let _: EquityField = EquityField::DaysToCover;
    let _: EquityField = EquityField::ShortInterestPctChange;
}

#[test]
fn test_equity_field_valuation_variants() {
    let _: EquityField = EquityField::PeRatio;
    let _: EquityField = EquityField::PegRatio5Y;
    let _: EquityField = EquityField::PriceBookRatio;
    let _: EquityField = EquityField::PriceTangibleBook;
    let _: EquityField = EquityField::PriceEarnings;
    let _: EquityField = EquityField::BookValueShare;
    let _: EquityField = EquityField::MarketCapToRevenue;
    let _: EquityField = EquityField::TevToRevenue;
    let _: EquityField = EquityField::TevEbit;
    let _: EquityField = EquityField::TevEbitda;
}

#[test]
fn test_equity_field_profitability_variants() {
    let _: EquityField = EquityField::Roa;
    let _: EquityField = EquityField::Roe;
    let _: EquityField = EquityField::ReturnOnCapital;
    let _: EquityField = EquityField::ForwardDivYield;
    let _: EquityField = EquityField::ForwardDivPerShare;
    let _: EquityField = EquityField::ConsecutiveDivYears;
    let _: EquityField = EquityField::NetIncomeMargin;
    let _: EquityField = EquityField::GrossProfitMargin;
    let _: EquityField = EquityField::EbitdaMargin;
}

#[test]
fn test_equity_field_income_statement_variants() {
    let _: EquityField = EquityField::TotalRevenues;
    let _: EquityField = EquityField::GrossProfit;
    let _: EquityField = EquityField::Ebitda;
    let _: EquityField = EquityField::Ebitda1YrGrowth;
    let _: EquityField = EquityField::NetIncome;
    let _: EquityField = EquityField::NetIncome1YrGrowth;
    let _: EquityField = EquityField::Revenue1YrGrowth;
    let _: EquityField = EquityField::QuarterlyRevGrowth;
    let _: EquityField = EquityField::EpsGrowth;
    let _: EquityField = EquityField::DilutedEps1YrGrowth;
    let _: EquityField = EquityField::NetEpsBasic;
    let _: EquityField = EquityField::NetEpsDiluted;
    let _: EquityField = EquityField::OperatingIncome;
    let _: EquityField = EquityField::Ebit;
}

#[test]
fn test_equity_field_balance_sheet_variants() {
    let _: EquityField = EquityField::TotalAssets;
    let _: EquityField = EquityField::TotalDebt;
    let _: EquityField = EquityField::TotalEquity;
    let _: EquityField = EquityField::TotalCommonEquity;
    let _: EquityField = EquityField::TotalCurrentAssets;
    let _: EquityField = EquityField::TotalCurrentLiab;
    let _: EquityField = EquityField::CashAndStInvestments;
    let _: EquityField = EquityField::CommonSharesOut;
    let _: EquityField = EquityField::TotalSharesOut;
    let _: EquityField = EquityField::TotalDebtEquity;
    let _: EquityField = EquityField::LtDebtEquity;
    let _: EquityField = EquityField::TotalDebtEbitda;
    let _: EquityField = EquityField::NetDebtEbitda;
    let _: EquityField = EquityField::EbitInterestExp;
    let _: EquityField = EquityField::EbitdaInterestExp;
}

#[test]
fn test_equity_field_liquidity_variants() {
    let _: EquityField = EquityField::QuickRatio;
    let _: EquityField = EquityField::CurrentRatio;
    let _: EquityField = EquityField::AltmanZScore;
    let _: EquityField = EquityField::OcfToCurrentLiab;
}

#[test]
fn test_equity_field_cash_flow_variants() {
    let _: EquityField = EquityField::CashFromOps;
    let _: EquityField = EquityField::CashFromOps1YrGrowth;
    let _: EquityField = EquityField::LeveredFcf;
    let _: EquityField = EquityField::LeveredFcf1YrGrowth;
    let _: EquityField = EquityField::UnleveredFcf;
    let _: EquityField = EquityField::Capex;
}

#[test]
fn test_equity_field_esg_variants() {
    let _: EquityField = EquityField::EsgScore;
    let _: EquityField = EquityField::EnvironmentalScore;
    let _: EquityField = EquityField::GovernanceScore;
    let _: EquityField = EquityField::SocialScore;
    let _: EquityField = EquityField::HighestControversy;
}

// ---------------------------------------------------------------------------
// FundField variants
// ---------------------------------------------------------------------------

#[test]
fn test_fund_field_variants_exist() {
    let _: FundField = FundField::Ticker;
    let _: FundField = FundField::CompanyShortName;
    let _: FundField = FundField::EodPrice;
    let _: FundField = FundField::IntradayPrice;
    let _: FundField = FundField::IntradayPriceChange;
    let _: FundField = FundField::CategoryName;
    let _: FundField = FundField::PerformanceRating;
    let _: FundField = FundField::InitialInvestment;
    let _: FundField = FundField::AnnualReturnRank;
    let _: FundField = FundField::RiskRating;
    let _: FundField = FundField::Exchange;
}

// ---------------------------------------------------------------------------
// ScreenerFieldExt — fluent condition builder methods
// ---------------------------------------------------------------------------

#[test]
fn test_screener_field_ext_gt() {
    let cond = EquityField::AvgDailyVol3M.gt(200_000.0);
    assert_eq!(cond.field, EquityField::AvgDailyVol3M);
}

#[test]
fn test_screener_field_ext_lt() {
    let cond = EquityField::PeRatio.lt(30.0);
    assert_eq!(cond.field, EquityField::PeRatio);
}

#[test]
fn test_screener_field_ext_gte() {
    let cond = EquityField::EsgScore.gte(50.0);
    assert_eq!(cond.field, EquityField::EsgScore);
}

#[test]
fn test_screener_field_ext_lte() {
    let cond = FundField::RiskRating.lte(3.0);
    assert_eq!(cond.field, FundField::RiskRating);
}

#[test]
fn test_screener_field_ext_eq_num() {
    let cond = EquityField::Beta.eq_num(1.0);
    assert_eq!(cond.field, EquityField::Beta);
}

#[test]
fn test_screener_field_ext_between() {
    let cond = EquityField::PeRatio.between(10.0, 25.0);
    assert_eq!(cond.field, EquityField::PeRatio);
}

#[test]
fn test_screener_field_ext_eq_str() {
    let cond = EquityField::Region.eq_str("us");
    assert_eq!(cond.field, EquityField::Region);
}

// ---------------------------------------------------------------------------
// EquityScreenerQuery builder — the main example from screeners.md
// ---------------------------------------------------------------------------

#[test]
fn test_equity_screener_query_builder() {
    let query = EquityScreenerQuery::new()
        .size(50)
        .sort_by(EquityField::IntradayMarketCap, false)
        .add_condition(EquityField::Region.eq_str("us"))
        .add_condition(EquityField::AvgDailyVol3M.gt(500_000.0))
        .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0))
        .add_condition(EquityField::PeRatio.between(10.0, 25.0));

    assert_eq!(query.size, 50);
    assert_eq!(query.sort_field, EquityField::IntradayMarketCap);
    assert_eq!(query.sort_type, SortType::Desc);
    assert_eq!(query.query.operands.len(), 4);
}

#[test]
fn test_equity_screener_query_offset_and_include_fields() {
    let query = EquityScreenerQuery::new()
        .size(100)
        .offset(50)
        .sort_by(EquityField::PeRatio, true)
        .include_fields(vec![
            EquityField::Ticker,
            EquityField::CompanyShortName,
            EquityField::IntradayPrice,
            EquityField::PeRatio,
            EquityField::IntradayMarketCap,
        ]);

    assert_eq!(query.size, 100);
    assert_eq!(query.offset, 50);
    assert_eq!(query.sort_field, EquityField::PeRatio);
    assert_eq!(query.sort_type, SortType::Asc);
    assert_eq!(query.include_fields.len(), 5);
}

#[test]
fn test_equity_screener_query_size_capped_at_250() {
    let query = EquityScreenerQuery::new().size(9999);
    assert_eq!(query.size, 250);
}

#[test]
fn test_equity_screener_query_top_operator() {
    let query = EquityScreenerQuery::new().top_operator(LogicalOperator::Or);
    assert_eq!(query.top_operator, LogicalOperator::Or);
}

// ---------------------------------------------------------------------------
// add_or_conditions — OR logic
// ---------------------------------------------------------------------------

#[test]
fn test_add_or_conditions() {
    let query = EquityScreenerQuery::new()
        .add_or_conditions(vec![
            EquityField::Region.eq_str("us"),
            EquityField::Region.eq_str("ca"),
        ])
        .add_condition(EquityField::IntradayMarketCap.gt(5_000_000_000.0));

    // The top-level AND group should have 2 operands: an OR sub-group and a condition
    assert_eq!(query.query.operands.len(), 2);
}

// ---------------------------------------------------------------------------
// Preset constructors
// ---------------------------------------------------------------------------

#[test]
fn test_preset_most_shorted() {
    let query = EquityScreenerQuery::most_shorted();
    assert_eq!(query.sort_field, EquityField::ShortPctFloat);
    assert_eq!(query.sort_type, SortType::Desc);
    assert_eq!(query.query.operands.len(), 2);
}

#[test]
fn test_preset_high_dividend() {
    let query = EquityScreenerQuery::high_dividend();
    assert_eq!(query.sort_field, EquityField::ForwardDivYield);
    assert_eq!(query.query.operands.len(), 3);
}

#[test]
fn test_preset_large_cap_growth() {
    let query = EquityScreenerQuery::large_cap_growth();
    assert_eq!(query.sort_field, EquityField::IntradayMarketCap);
    assert_eq!(query.query.operands.len(), 3);
}

// ---------------------------------------------------------------------------
// FundScreenerQuery builder
// ---------------------------------------------------------------------------

#[test]
fn test_fund_screener_query_builder() {
    let query = FundScreenerQuery::new()
        .size(25)
        .sort_by(FundField::PerformanceRating, false)
        .add_condition(FundField::RiskRating.lte(3.0))
        .include_fields(vec![
            FundField::Ticker,
            FundField::CompanyShortName,
            FundField::IntradayPrice,
            FundField::CategoryName,
            FundField::PerformanceRating,
            FundField::RiskRating,
        ]);

    assert_eq!(query.size, 25);
    assert_eq!(query.sort_field, FundField::PerformanceRating);
    assert_eq!(query.sort_type, SortType::Desc);
    assert_eq!(query.query.operands.len(), 1);
    assert_eq!(query.include_fields.len(), 6);
}

// ---------------------------------------------------------------------------
// Network tests (screeners.md examples that hit Yahoo Finance)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires network access"]
async fn test_predefined_screener_day_gainers() {
    use finance_query::finance;

    let gainers = finance::screener(Screener::DayGainers, 25).await.unwrap();
    assert!(!gainers.quotes.is_empty());

    for quote in &gainers.quotes {
        let change_pct = quote.regular_market_change_percent.raw.unwrap_or(0.0);
        println!("{}: {:+.2}%", quote.symbol, change_pct);
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_screener_equity() {
    use finance_query::finance;

    let query = EquityScreenerQuery::new()
        .size(10)
        .sort_by(EquityField::IntradayMarketCap, false)
        .add_condition(EquityField::Region.eq_str("us"))
        .add_condition(EquityField::AvgDailyVol3M.gt(1_000_000.0))
        .add_condition(EquityField::IntradayMarketCap.gt(10_000_000_000.0));

    let results = finance::custom_screener(query).await.unwrap();
    assert!(!results.quotes.is_empty());
    println!("Found {} stocks", results.quotes.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_screener_preset_high_dividend() {
    use finance_query::finance;

    let results = finance::custom_screener(EquityScreenerQuery::high_dividend())
        .await
        .unwrap();
    println!("High dividend stocks: {}", results.quotes.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_predefined_screener_most_actives() {
    use finance_query::finance;

    // From screeners.md "Predefined Screeners" section
    let actives = finance::screener(Screener::MostActives, 50).await.unwrap();
    assert!(!actives.quotes.is_empty());
    println!("Most actives: {}", actives.quotes.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_screener_preset_most_shorted() {
    use finance_query::finance;

    // From screeners.md "Preset Constructors" section
    let results = finance::custom_screener(EquityScreenerQuery::most_shorted())
        .await
        .unwrap();
    println!("Most shorted stocks: {}", results.quotes.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_custom_screener_preset_large_cap_growth() {
    use finance_query::finance;

    // From screeners.md "Preset Constructors" section
    let results = finance::custom_screener(EquityScreenerQuery::large_cap_growth())
        .await
        .unwrap();
    println!("Large cap growth stocks: {}", results.quotes.len());
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fund_screener() {
    use finance_query::finance;

    // From screeners.md "Mutual Fund Screener" section
    let query = FundScreenerQuery::new()
        .size(25)
        .sort_by(FundField::PerformanceRating, false)
        .add_condition(FundField::RiskRating.lte(3.0))
        .include_fields(vec![
            FundField::Ticker,
            FundField::CompanyShortName,
            FundField::IntradayPrice,
            FundField::CategoryName,
            FundField::PerformanceRating,
            FundField::RiskRating,
        ]);

    let results = finance::custom_screener(query).await.unwrap();
    println!("Fund results: {}", results.quotes.len());
}
