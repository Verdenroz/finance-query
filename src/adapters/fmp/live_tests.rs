use std::sync::Arc;

use chrono::{Datelike, Days, Utc};

use super::{
    Period, client::FmpClientBuilder, commodities, corporate, crypto, discovery, forex,
    fundamentals, indices, market, quote,
};
use crate::{
    StatementType,
    error::FinanceError,
    models::{
        calendar::market::CalendarKind, discovery::reference::ScreenerFilters, indices::MajorIndex,
        market::performance::MoverDirection,
    },
    rate_limiter::RateLimiter,
};

#[tokio::test]
#[ignore = "requires FMP_API_KEY and consumes live API quota and bandwidth"]
#[allow(clippy::too_many_lines)]
async fn all_supported_fmp_functions_work_live() {
    super::init_with_timeout(
        std::env::var("FMP_API_KEY").expect("FMP_API_KEY must be set"),
        std::time::Duration::from_secs(120),
    )
    .expect("FMP client must initialize");

    let today = Utc::now().date_naive();
    let from = today.checked_sub_days(Days::new(365)).unwrap().to_string();
    let recent_from = today.checked_sub_days(Days::new(30)).unwrap().to_string();
    let to = today.to_string();
    let latest_quarter_end = match today.month() {
        1..=3 => format!("{}-12-31", today.year() - 1),
        4..=6 => format!("{}-03-31", today.year()),
        7..=9 => format!("{}-06-30", today.year()),
        _ => format!("{}-09-30", today.year()),
    };
    let history = quote::prices::HistoricalPriceParams {
        from: Some(recent_from.clone()),
        to: Some(to.clone()),
    };
    let history_pairs = [("from", recent_from.as_str()), ("to", to.as_str())];
    let screener = ScreenerFilters::new()
        .market_cap(Some(100_000_000_000.0), None)
        .limit(5);
    let mut failures = Vec::new();
    let mut passed = 0_usize;
    let mut total = 0_usize;

    macro_rules! check {
        ($name:literal, $future:expr) => {{
            total += 1;
            match $future.await {
                Ok(_) => {
                    passed += 1;
                    println!("ok: {}", $name);
                }
                Err(error) => {
                    eprintln!("failed: {}: {}", $name, error);
                    failures.push(format!("{}: {}", $name, error));
                }
            }
        }};
    }

    check!("commodity_quote", commodities::commodity_quote("GCUSD"));
    check!(
        "fetch_canonical_commodity_quote",
        commodities::fetch_canonical_commodity_quote("GCUSD")
    );
    check!("crypto_quote", crypto::crypto_quote("BTCUSD"));
    check!(
        "fetch_canonical_crypto_quote",
        crypto::fetch_canonical_crypto_quote("BTC", "USD")
    );
    check!("forex_quote", forex::forex_quote("EURUSD"));
    check!(
        "fetch_canonical_forex_quote",
        forex::fetch_canonical_forex_quote("EUR", "USD")
    );

    check!("quote", quote::prices::quote("AAPL"));
    check!("batch_quote", quote::prices::batch_quote(&["AAPL", "MSFT"]));
    check!(
        "fetch_canonical_quote",
        quote::prices::fetch_canonical_quote("AAPL")
    );
    check!(
        "fetch_canonical_quotes_batch",
        quote::prices::fetch_canonical_quotes_batch(&["AAPL", "MSFT"])
    );
    check!(
        "historical_price_daily",
        quote::prices::historical_price_daily("AAPL", Some(history.clone()))
    );
    check!(
        "historical_price_intraday",
        quote::prices::historical_price_intraday("AAPL", "1hour", Some(history.clone()))
    );
    check!(
        "fetch_daily_chart_candles",
        quote::prices::fetch_daily_chart_candles("AAPL", Some(history.clone()))
    );
    check!(
        "fetch_intraday_chart_candles",
        quote::prices::fetch_intraday_chart_candles("AAPL", "1hour", Some(history.clone()))
    );
    check!("quote_short", quote::extended::quote_short("AAPL"));
    check!(
        "batch_quote_short",
        quote::extended::batch_quote_short(&["AAPL", "MSFT"])
    );
    check!(
        "aftermarket_trade",
        quote::extended::aftermarket_trade("AAPL")
    );
    check!(
        "batch_aftermarket_trade",
        quote::extended::batch_aftermarket_trade(&["AAPL", "MSFT"])
    );
    check!(
        "aftermarket_quote",
        quote::extended::aftermarket_quote("AAPL")
    );
    check!(
        "batch_aftermarket_quote",
        quote::extended::batch_aftermarket_quote(&["AAPL", "MSFT"])
    );
    check!(
        "stock_price_change",
        quote::extended::stock_price_change("AAPL")
    );

    check!("company_profile", quote::company::company_profile("AAPL"));
    check!("key_executives", quote::company::key_executives("AAPL"));
    check!("market_cap", quote::company::market_cap("AAPL"));
    check!(
        "historical_market_cap",
        quote::company::historical_market_cap("AAPL", Some(5))
    );
    check!("stock_peers", quote::company::stock_peers("AAPL"));
    check!(
        "fetch_canonical_similar_symbols",
        quote::company::fetch_canonical_similar_symbols("AAPL", 5)
    );
    check!(
        "delisted_companies",
        quote::company::delisted_companies(Some(5))
    );

    check!("sic_codes", quote::advanced::sic_codes());
    check!("sic_by_code", quote::advanced::sic_by_code("3571"));
    let cot_symbol = {
        total += 1;
        match quote::advanced::cot_symbols().await {
            Ok(rows) => {
                passed += 1;
                println!("ok: cot_symbols");
                rows.into_iter().find_map(|row| row.trading_symbol)
            }
            Err(error) => {
                eprintln!("failed: cot_symbols: {error}");
                failures.push(format!("cot_symbols: {error}"));
                None
            }
        }
    };
    if let Some(cot_symbol) = cot_symbol.as_deref() {
        check!("cot_report", quote::advanced::cot_report(cot_symbol));
        check!("cot_analysis", quote::advanced::cot_analysis(cot_symbol));
    } else {
        total += 2;
        failures.push("cot_report: no symbol returned by cot_symbols".to_string());
        failures.push("cot_analysis: no symbol returned by cot_symbols".to_string());
    }

    check!("name_search", discovery::reference::name_search("Apple"));
    check!("cik_search", discovery::reference::cik_search("320193"));
    check!(
        "cusip_search",
        discovery::reference::cusip_search("037833100")
    );
    check!(
        "isin_search",
        discovery::reference::isin_search("US0378331005")
    );
    check!(
        "exchange_variants",
        discovery::reference::exchange_variants("AAPL")
    );
    check!(
        "stock_screener",
        discovery::screener::stock_screener(&[
            ("marketCapMoreThan", "100000000000"),
            ("limit", "5")
        ])
    );
    check!(
        "symbol_search",
        discovery::screener::symbol_search("Apple", Some(5), None)
    );
    check!(
        "fetch_symbol_search_response",
        discovery::screener::fetch_symbol_search_response("Apple", 5)
    );
    check!(
        "fetch_screener_response",
        discovery::screener::fetch_screener_response(&screener)
    );

    check!(
        "income_statement",
        fundamentals::core::income_statement("AAPL", Period::Annual, Some(2))
    );
    check!(
        "balance_sheet",
        fundamentals::core::balance_sheet("AAPL", Period::Annual, Some(2))
    );
    check!(
        "cash_flow",
        fundamentals::core::cash_flow("AAPL", Period::Annual, Some(2))
    );
    for statement in [
        StatementType::Income,
        StatementType::Balance,
        StatementType::CashFlow,
    ] {
        check!(
            "fetch_financials_data",
            fundamentals::core::fetch_financials_data("AAPL", statement, Period::Annual, Some(2),)
        );
    }
    check!(
        "financial_ratios",
        fundamentals::analysis::financial_ratios("AAPL", Period::Annual, Some(2))
    );
    check!(
        "key_metrics",
        fundamentals::analysis::key_metrics("AAPL", Period::Annual, Some(2))
    );
    check!(
        "enterprise_value",
        fundamentals::analysis::enterprise_value("AAPL", Period::Annual, Some(2))
    );
    check!(
        "discounted_cash_flow",
        fundamentals::analysis::discounted_cash_flow("AAPL")
    );
    check!(
        "company_rating",
        fundamentals::analysis::company_rating("AAPL")
    );
    check!(
        "historical_rating",
        fundamentals::analysis::historical_rating("AAPL", Some(5))
    );
    check!(
        "financial_growth",
        fundamentals::analysis::financial_growth("AAPL", Period::Annual, Some(2))
    );
    check!(
        "key_metrics_ttm",
        fundamentals::ttm::key_metrics_ttm("AAPL")
    );
    check!("ratios_ttm", fundamentals::ttm::ratios_ttm("AAPL"));
    check!(
        "fetch_key_metrics_ttm_response",
        fundamentals::ttm::fetch_key_metrics_ttm_response("AAPL")
    );
    check!(
        "fetch_ratios_ttm_response",
        fundamentals::ttm::fetch_ratios_ttm_response("AAPL")
    );
    check!(
        "financial_scores",
        fundamentals::health::financial_scores("AAPL")
    );
    check!(
        "owner_earnings",
        fundamentals::health::owner_earnings("AAPL")
    );
    check!("shares_float", fundamentals::float::shares_float("AAPL"));
    check!(
        "fetch_share_float_response",
        fundamentals::float::fetch_share_float_response("AAPL")
    );

    check!(
        "analyst_estimates",
        fundamentals::estimates::analyst_estimates("AAPL", Period::Annual, 2)
    );
    check!(
        "analyst_recommendations",
        fundamentals::estimates::analyst_recommendations("AAPL")
    );
    check!(
        "earnings_surprises",
        fundamentals::estimates::earnings_surprises("AAPL")
    );
    check!(
        "stock_grade",
        fundamentals::estimates::stock_grade("AAPL", 5)
    );
    check!(
        "earnings_transcript",
        fundamentals::estimates::earnings_transcript("AAPL", 3, today.year() as u32)
    );
    check!(
        "earnings_transcript_list",
        fundamentals::estimates::earnings_transcript_list("AAPL")
    );
    check!(
        "price_target_consensus",
        fundamentals::consensus::price_target_consensus("AAPL")
    );
    check!(
        "price_target_summary",
        fundamentals::consensus::price_target_summary("AAPL")
    );
    check!(
        "upgrades_downgrades_consensus",
        fundamentals::consensus::upgrades_downgrades_consensus("AAPL")
    );
    check!(
        "fetch_price_target_consensus_response",
        fundamentals::consensus::fetch_price_target_consensus_response("AAPL")
    );
    check!(
        "fetch_price_target_summary_response",
        fundamentals::consensus::fetch_price_target_summary_response("AAPL")
    );
    check!(
        "fetch_rating_consensus_response",
        fundamentals::consensus::fetch_rating_consensus_response("AAPL")
    );

    check!(
        "etf_quote",
        fundamentals::etf_mutual_funds::etf_quote("SPY")
    );
    check!(
        "etf_available",
        fundamentals::etf_mutual_funds::etf_available()
    );
    check!(
        "etf_historical",
        fundamentals::etf_mutual_funds::etf_historical("SPY", &history_pairs)
    );
    check!(
        "mutual_fund_quote",
        fundamentals::etf_mutual_funds::mutual_fund_quote("VFIAX")
    );
    check!(
        "mutual_fund_historical",
        fundamentals::etf_mutual_funds::mutual_fund_historical("VFIAX", &history_pairs)
    );
    check!(
        "etf_sector_weightings",
        fundamentals::fund_holdings::etf_sector_weightings("SPY")
    );
    check!(
        "etf_country_weightings",
        fundamentals::fund_holdings::etf_country_weightings("SPY")
    );
    check!(
        "etf_holdings",
        fundamentals::fund_holdings::etf_holdings("SPY")
    );

    check!(
        "earnings_calendar",
        market::calendars::earnings_calendar(&from, &to)
    );
    check!("ipo_calendar", market::calendars::ipo_calendar(&from, &to));
    check!(
        "stock_split_calendar",
        market::calendars::stock_split_calendar(&from, &to)
    );
    check!(
        "dividend_calendar",
        market::calendars::dividend_calendar(&from, &to)
    );
    check!(
        "economic_calendar",
        market::calendars::economic_calendar(&from, &to)
    );
    for kind in [
        CalendarKind::Earnings,
        CalendarKind::Ipo,
        CalendarKind::Dividend,
        CalendarKind::Split,
        CalendarKind::Economic,
    ] {
        check!(
            "fetch_market_calendar_response",
            market::calendars::fetch_market_calendar_response(kind, &from, &to)
        );
    }
    check!(
        "treasury_rates",
        market::economics::treasury_rates(Some(&from), Some(&to))
    );
    check!(
        "economic_indicators",
        market::economics::economic_indicators("GDP")
    );
    check!(
        "market_risk_premium",
        market::economics::market_risk_premium()
    );
    check!("sectors_pe", market::market_performance::sectors_pe());
    check!("industries_pe", market::market_performance::industries_pe());
    check!(
        "sector_performance",
        market::market_performance::sector_performance()
    );
    check!(
        "historical_sector_performance",
        market::market_performance::historical_sector_performance(5)
    );
    check!(
        "stock_market_gainers",
        market::market_performance::stock_market_gainers()
    );
    check!(
        "stock_market_losers",
        market::market_performance::stock_market_losers()
    );
    check!(
        "stock_market_most_active",
        market::market_performance::stock_market_most_active()
    );
    check!(
        "fetch_sector_performance_response",
        market::market_performance::fetch_sector_performance_response()
    );
    for direction in [
        MoverDirection::Gainers,
        MoverDirection::Losers,
        MoverDirection::MostActive,
    ] {
        check!(
            "fetch_market_movers_response",
            market::market_performance::fetch_market_movers_response(direction)
        );
    }
    check!(
        "fetch_sector_pe_response",
        market::market_performance::fetch_sector_pe_response()
    );
    check!(
        "fetch_industry_pe_response",
        market::market_performance::fetch_industry_pe_response()
    );
    check!(
        "fetch_sector_performance_history_response",
        market::market_performance::fetch_sector_performance_history_response(5)
    );

    check!("major_indexes_quote", indices::major_indexes_quote());
    check!(
        "fetch_canonical_index_quote",
        indices::fetch_canonical_index_quote("^GSPC")
    );
    check!("sp500_constituents", indices::sp500_constituents());
    check!("nasdaq_constituents", indices::nasdaq_constituents());
    check!("dow_constituents", indices::dow_constituents());
    check!("historical_sp500", indices::historical_sp500());
    for index in [
        MajorIndex::Sp500,
        MajorIndex::Nasdaq100,
        MajorIndex::DowJones,
    ] {
        check!(
            "fetch_index_constituents_response",
            indices::fetch_index_constituents_response(index)
        );
    }
    check!(
        "fetch_index_constituent_changes_response",
        indices::fetch_index_constituent_changes_response(MajorIndex::Sp500)
    );

    check!(
        "historical_dividends",
        corporate::dividends_splits::historical_dividends("AAPL")
    );
    check!(
        "historical_splits",
        corporate::dividends_splits::historical_splits("AAPL")
    );
    check!(
        "fetch_canonical_events",
        corporate::dividends_splits::fetch_canonical_events("AAPL")
    );
    check!("stock_news", corporate::news::stock_news("AAPL", 5));
    check!("press_releases", corporate::news::press_releases("AAPL", 5));
    check!("crypto_news", corporate::news::crypto_news(5));
    check!("forex_news", corporate::news::forex_news(5));
    check!(
        "fetch_canonical_news",
        corporate::news::fetch_canonical_news("AAPL", 5)
    );
    check!(
        "fetch_press_releases_response",
        corporate::news::fetch_press_releases_response("AAPL", 5)
    );
    check!(
        "insider_trading",
        corporate::insider_trading::insider_trading("AAPL", 5)
    );
    check!(
        "insider_trading_rss",
        corporate::insider_trading::insider_trading_rss(5)
    );
    check!("cik_mapper", corporate::insider_trading::cik_mapper("Cook"));
    check!(
        "cik_mapper_by_company",
        corporate::insider_trading::cik_mapper_by_company("Apple")
    );
    check!(
        "congressional_trading",
        corporate::insider_trading::congressional_trading("AAPL")
    );
    check!(
        "institutional_holders",
        corporate::institutional::institutional_holders("AAPL")
    );
    check!("etf_holders", corporate::institutional::etf_holders("SPY"));
    check!(
        "mutual_fund_holders",
        corporate::institutional::mutual_fund_holders("AAPL")
    );
    check!(
        "form_13f",
        corporate::institutional::form_13f("0001067983", &latest_quarter_end)
    );
    check!(
        "executive_compensation",
        corporate::ownership::executive_compensation("AAPL")
    );
    check!(
        "employee_count",
        corporate::ownership::employee_count("AAPL")
    );
    check!(
        "fetch_executive_compensation_response",
        corporate::ownership::fetch_executive_compensation_response("AAPL")
    );
    check!(
        "fetch_employee_count_response",
        corporate::ownership::fetch_employee_count_response("AAPL")
    );
    check!(
        "sec_filings_by_symbol",
        corporate::filings::sec_filings_by_symbol("AAPL", &from, &to, 0, 5)
    );
    check!(
        "sec_filings_by_form_type",
        corporate::filings::sec_filings_by_form_type("10-Q", &from, &to, 0, 5)
    );
    check!(
        "sec_filings_by_cik",
        corporate::filings::sec_filings_by_cik("0000320193", &from, &to, 0, 5)
    );

    check!(
        "bulk_income_statements",
        quote::bulk::bulk_income_statements("2025", "FY")
    );
    check!(
        "bulk_balance_sheets",
        quote::bulk::bulk_balance_sheets("2025", "FY")
    );
    check!("bulk_cash_flow", quote::bulk::bulk_cash_flow("2025", "FY"));
    check!("bulk_ratios_ttm", quote::bulk::bulk_ratios_ttm());
    check!("bulk_key_metrics_ttm", quote::bulk::bulk_key_metrics_ttm());
    check!("bulk_profiles", quote::bulk::bulk_profiles());

    total += 1;
    let invalid_client = FmpClientBuilder::new("invalid-live-test-key")
        .build_with_limiter(Arc::new(RateLimiter::new(5.0)))
        .expect("invalid-key test client must build");
    match invalid_client
        .get_raw("/stable/quote", &[("symbol", "AAPL")])
        .await
    {
        Err(FinanceError::AuthenticationFailed { .. }) => {
            passed += 1;
            println!("ok: invalid_api_key_error");
        }
        result => failures.push(format!(
            "invalid_api_key_error: expected AuthenticationFailed, got {result:?}"
        )),
    }

    println!("FMP live matrix: {passed}/{total} checks passed");
    assert!(
        failures.is_empty(),
        "{} FMP live checks failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
