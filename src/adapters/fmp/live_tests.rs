//! Live contract matrix for the routed FMP surface.
//!
//! Every check asserts that at least one semantically-required field arrived
//! populated. FMP's `/stable` tier re-keyed many fields relative to `/api/v3`,
//! and because every DTO field is `Option<T>` a stale key deserializes to
//! `None` instead of erroring. Asserting only `Ok(_)` would pass on a
//! response that is entirely null, so a failure names the endpoint and the
//! field that was empty.

use std::future::Future;
use std::sync::Arc;

use chrono::{Days, Utc};

use super::{
    Period, client::FmpClientBuilder, corporate, discovery, forex, fundamentals, indices, market,
    quote,
};
use crate::{
    StatementType,
    error::{FinanceError, Result},
    models::{
        calendar::market::CalendarKind, discovery::reference::ScreenerFilters, indices::MajorIndex,
        market::performance::MoverDirection,
    },
    rate_limiter::RateLimiter,
};

type Check = std::result::Result<(), String>;

/// Strip anything that follows an API-key query parameter so a failing run is
/// safe to paste into an issue.
fn redact(message: &str) -> String {
    let lowered = message.to_ascii_lowercase();
    let mut out = String::with_capacity(message.len());
    let mut cursor = 0;
    while let Some(value) = ["apikey=", "api_key="]
        .iter()
        .filter_map(|key| {
            lowered[cursor..]
                .find(key)
                .map(|at| cursor + at + key.len())
        })
        .min()
    {
        let end = message[value..]
            .find(['&', ' ', '"'])
            .map_or(message.len(), |offset| value + offset);
        out.push_str(&message[cursor..value]);
        out.push_str("REDACTED");
        cursor = end;
    }
    out.push_str(&message[cursor..]);
    out
}

#[test]
fn redact_hides_every_key_occurrence() {
    let redacted = redact("GET /stable/quote?apikey=sk-live-1&symbol=AAPL then api_key=sk-live-2");
    assert_eq!(
        redacted,
        "GET /stable/quote?apikey=REDACTED&symbol=AAPL then api_key=REDACTED"
    );
    assert!(!redacted.contains("sk-live"));
    assert_eq!(redact("no key here"), "no key here");
}

fn first<'a, T>(rows: &'a [T], what: &str) -> std::result::Result<&'a T, String> {
    rows.first()
        .ok_or_else(|| format!("returned no {what} rows"))
}

fn text(value: Option<&str>, field: &str) -> Check {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(()),
        Some(_) => Err(format!("{field} is empty")),
        None => Err(format!("{field} is null")),
    }
}

/// A number that must be present and meaningfully non-zero. Prices, counts and
/// totals are never legitimately `0.0` for the symbols this matrix probes, so a
/// zero here means the key stopped matching.
fn number(value: Option<f64>, field: &str) -> Check {
    match value {
        Some(v) if v.is_finite() && v != 0.0 => Ok(()),
        Some(v) => Err(format!("{field} is degenerate ({v})")),
        None => Err(format!("{field} is null")),
    }
}

fn count(value: Option<i64>, field: &str) -> Check {
    match value {
        Some(v) if v != 0 => Ok(()),
        Some(_) => Err(format!("{field} is zero")),
        None => Err(format!("{field} is null")),
    }
}

fn present<T>(value: Option<T>, field: &str) -> Check {
    value.map(|_| ()).ok_or_else(|| format!("{field} is null"))
}

fn non_empty<T>(rows: &[T], what: &str) -> Check {
    if rows.is_empty() {
        return Err(format!("returned no {what}"));
    }
    Ok(())
}

struct Matrix {
    label: &'static str,
    failures: Vec<String>,
    passed: usize,
    total: usize,
}

impl Matrix {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            failures: Vec::new(),
            passed: 0,
            total: 0,
        }
    }

    async fn check<T, F>(&mut self, name: &str, future: F, require: impl FnOnce(&T) -> Check)
    where
        F: Future<Output = Result<T>>,
    {
        self.total += 1;
        match future.await {
            Ok(value) => match require(&value) {
                Ok(()) => {
                    self.passed += 1;
                    println!("ok: {name}");
                }
                Err(problem) => self.fail(name, &problem),
            },
            Err(error) => self.fail(name, &error.to_string()),
        }
    }

    fn fail(&mut self, name: &str, problem: &str) {
        let line = format!("{name}: {}", redact(problem));
        eprintln!("failed: {line}");
        self.failures.push(line);
    }

    fn finish(self) {
        println!(
            "{} live matrix: {}/{} checks passed",
            self.label, self.passed, self.total
        );
        assert!(
            self.failures.is_empty(),
            "{} {} live checks failed:\n{}",
            self.failures.len(),
            self.label,
            self.failures.join("\n")
        );
    }
}

#[tokio::test]
#[ignore = "requires FMP_API_KEY and consumes live API quota and bandwidth"]
#[allow(clippy::too_many_lines)]
async fn all_routed_fmp_endpoints_return_populated_data() {
    super::init_with_timeout(
        std::env::var("FMP_API_KEY").expect("FMP_API_KEY must be set"),
        std::time::Duration::from_secs(120),
    )
    .expect("FMP client must initialize");

    let today = Utc::now().date_naive();
    let from = today.checked_sub_days(Days::new(365)).unwrap().to_string();
    let recent_from = today.checked_sub_days(Days::new(30)).unwrap().to_string();
    let to = today.to_string();
    let history = quote::prices::HistoricalPriceParams {
        from: Some(recent_from.clone()),
        to: Some(to.clone()),
    };
    let screener = ScreenerFilters::new()
        .market_cap(Some(100_000_000_000.0), None)
        .limit(5);

    let mut m = Matrix::new("FMP");

    // ---- QUOTE -------------------------------------------------------------
    m.check("quote", quote::prices::quote("AAPL"), |rows| {
        let q = first(rows, "quote")?;
        text(Some(q.symbol.as_str()), "symbol")?;
        number(q.price, "price")?;
        number(q.market_cap, "marketCap")?;
        text(q.exchange.as_deref(), "exchange")
    })
    .await;

    m.check(
        "batch_quote",
        quote::prices::batch_quote(&["AAPL", "MSFT"]),
        |rows| {
            if rows.len() < 2 {
                return Err(format!("expected 2 quotes, got {}", rows.len()));
            }
            number(rows[0].price, "price")?;
            number(rows[1].price, "price")
        },
    )
    .await;

    m.check(
        "fetch_canonical_quote",
        quote::prices::fetch_canonical_quote("AAPL"),
        |q| {
            let price = q.price.as_ref().ok_or("price block is null")?;
            number(
                price.regular_market_price.as_ref().and_then(|v| v.raw),
                "price.regularMarketPrice",
            )?;
            present(
                price.market_cap.as_ref().and_then(|v| v.raw),
                "price.marketCap",
            )
        },
    )
    .await;

    m.check(
        "fetch_canonical_quotes_batch",
        quote::prices::fetch_canonical_quotes_batch(&["AAPL", "MSFT"]),
        |rows| {
            non_empty(rows, "batch quotes")?;
            let (symbol, quote) = &rows[0];
            text(Some(symbol.as_str()), "symbol")?;
            let price = quote.price.as_ref().ok_or("price block is null")?;
            number(
                price.regular_market_price.as_ref().and_then(|v| v.raw),
                "price.regularMarketPrice",
            )
        },
    )
    .await;

    // ---- CHART -------------------------------------------------------------
    m.check(
        "historical_price_daily",
        quote::prices::historical_price_daily("AAPL", Some(history.clone())),
        |resp| {
            let bar = first(&resp.historical, "daily price")?;
            text(bar.date.as_deref(), "date")?;
            number(bar.close, "close")?;
            number(bar.volume, "volume")
        },
    )
    .await;

    m.check(
        "historical_price_intraday",
        quote::prices::historical_price_intraday("AAPL", "1hour", Some(history.clone())),
        |rows| {
            let bar = first(rows, "intraday price")?;
            text(bar.date.as_deref(), "date")?;
            number(bar.close, "close")
        },
    )
    .await;

    m.check(
        "fetch_daily_chart_candles",
        quote::prices::fetch_daily_chart_candles("AAPL", Some(history.clone())),
        |candles| {
            non_empty(candles, "daily candles")?;
            let c = &candles[0];
            number(Some(c.close), "close")?;
            number(Some(c.timestamp as f64), "timestamp")?;
            if candles.windows(2).any(|w| w[0].timestamp > w[1].timestamp) {
                return Err("candles are not ascending by timestamp".to_string());
            }
            Ok(())
        },
    )
    .await;

    m.check(
        "fetch_intraday_chart_candles",
        quote::prices::fetch_intraday_chart_candles("AAPL", "1hour", Some(history)),
        |candles| {
            non_empty(candles, "intraday candles")?;
            number(Some(candles[0].close), "close")
        },
    )
    .await;

    // ---- FUNDAMENTALS ------------------------------------------------------
    m.check(
        "income_statement",
        fundamentals::core::income_statement("AAPL", Period::Annual, Some(2)),
        |rows| {
            let s = first(rows, "income statement")?;
            text(s.date.as_deref(), "date")?;
            text(s.filling_date.as_deref(), "filingDate")?;
            text(s.calendar_year.as_deref(), "fiscalYear")?;
            number(s.revenue, "revenue")?;
            number(s.net_income, "netIncome")?;
            number(s.eps_diluted, "epsDiluted")
        },
    )
    .await;

    m.check(
        "balance_sheet",
        fundamentals::core::balance_sheet("AAPL", Period::Annual, Some(2)),
        |rows| {
            let s = first(rows, "balance sheet")?;
            text(s.filling_date.as_deref(), "filingDate")?;
            text(s.calendar_year.as_deref(), "fiscalYear")?;
            number(s.total_assets, "totalAssets")?;
            number(s.total_liabilities, "totalLiabilities")?;
            number(s.total_stockholders_equity, "totalStockholdersEquity")
        },
    )
    .await;

    m.check(
        "cash_flow",
        fundamentals::core::cash_flow("AAPL", Period::Annual, Some(2)),
        |rows| {
            let s = first(rows, "cash flow")?;
            text(s.filling_date.as_deref(), "filingDate")?;
            number(s.net_cash_provided_by_operating_activities, "operatingCash")?;
            number(
                s.net_cash_used_for_investing_activities,
                "netCashProvidedByInvestingActivities",
            )?;
            number(
                s.net_cash_used_provided_by_financing_activities,
                "netCashProvidedByFinancingActivities",
            )?;
            number(s.debt_repayment, "netDebtIssuance")?;
            number(s.dividends_paid, "netDividendsPaid")?;
            number(s.free_cash_flow, "freeCashFlow")
        },
    )
    .await;

    for statement in [
        StatementType::Income,
        StatementType::Balance,
        StatementType::CashFlow,
    ] {
        m.check(
            "fetch_financials_data",
            fundamentals::core::fetch_financials_data("AAPL", statement, Period::Annual, Some(2)),
            |data| {
                if data.is_empty() {
                    return Err("pivoted no metrics".to_string());
                }
                if data.values().all(std::collections::HashMap::is_empty) {
                    return Err("every metric has no periods".to_string());
                }
                Ok(())
            },
        )
        .await;
    }

    m.check(
        "key_metrics_ttm",
        fundamentals::ttm::key_metrics_ttm("AAPL"),
        |rows| {
            let k = first(rows, "TTM key metrics")?;
            number(k.market_cap, "marketCap")?;
            number(k.enterprise_value, "enterpriseValueTTM")?;
            number(k.return_on_equity, "returnOnEquityTTM")?;
            number(k.return_on_invested_capital, "returnOnInvestedCapitalTTM")?;
            number(k.return_on_assets, "returnOnAssetsTTM")?;
            number(k.ev_to_ebitda, "evToEBITDATTM")
        },
    )
    .await;

    m.check(
        "ratios_ttm",
        fundamentals::ttm::ratios_ttm("AAPL"),
        |rows| {
            let r = first(rows, "TTM ratios")?;
            number(r.price_earnings_ratio, "priceToEarningsRatioTTM")?;
            number(r.debt_ratio, "debtToAssetsRatioTTM")?;
            number(r.debt_equity_ratio, "debtToEquityRatioTTM")?;
            number(r.interest_coverage, "interestCoverageRatioTTM")?;
            number(r.revenue_per_share, "revenuePerShareTTM")?;
            number(r.free_cash_flow_per_share, "freeCashFlowPerShareTTM")?;
            number(r.gross_profit_margin, "grossProfitMarginTTM")
        },
    )
    .await;

    m.check(
        "fetch_key_metrics_ttm_response",
        fundamentals::ttm::fetch_key_metrics_ttm_response("AAPL"),
        |k| {
            text(k.symbol.as_deref(), "symbol")?;
            number(k.market_cap, "marketCap")?;
            number(k.return_on_equity, "returnOnEquityTTM")
        },
    )
    .await;

    m.check(
        "fetch_ratios_ttm_response",
        fundamentals::ttm::fetch_ratios_ttm_response("AAPL"),
        |r| {
            text(r.symbol.as_deref(), "symbol")?;
            number(r.price_earnings_ratio, "priceToEarningsRatioTTM")?;
            number(r.net_profit_margin, "netProfitMarginTTM")
        },
    )
    .await;

    m.check(
        "shares_float",
        fundamentals::float::shares_float("AAPL"),
        |rows| {
            let f = first(rows, "share float")?;
            number(f.float_shares, "floatShares")?;
            number(f.outstanding_shares, "outstandingShares")?;
            number(f.free_float, "freeFloat")
        },
    )
    .await;

    m.check(
        "fetch_share_float_response",
        fundamentals::float::fetch_share_float_response("AAPL"),
        |f| {
            number(f.float_shares, "float_shares")?;
            number(f.outstanding_shares, "outstanding_shares")?;
            number(f.float_percent, "float_percent")?;
            text(f.date.as_deref(), "date")
        },
    )
    .await;

    // ---- ANALYST CONSENSUS AND ESTIMATES -----------------------------------
    m.check(
        "analyst_estimates",
        fundamentals::estimates::analyst_estimates("AAPL", Period::Annual, 2),
        |rows| {
            let e = first(rows, "analyst estimate")?;
            text(e.date.as_deref(), "date")?;
            number(e.estimated_revenue_avg, "revenueAvg")?;
            number(e.estimated_revenue_low, "revenueLow")?;
            number(e.estimated_revenue_high, "revenueHigh")?;
            number(e.estimated_ebitda_avg, "ebitdaAvg")?;
            number(e.estimated_eps_avg, "epsAvg")?;
            number(e.estimated_eps_low, "epsLow")?;
            number(e.estimated_eps_high, "epsHigh")?;
            count(
                e.number_analyst_estimated_revenue.map(i64::from),
                "numAnalystsRevenue",
            )?;
            count(
                e.number_analysts_estimated_eps.map(i64::from),
                "numAnalystsEps",
            )
        },
    )
    .await;

    m.check(
        "analyst_recommendations",
        fundamentals::estimates::analyst_recommendations("AAPL"),
        |rows| {
            let r = first(rows, "analyst recommendation")?;
            text(r.date.as_deref(), "date")?;
            let total = [
                r.analyst_ratings_buy,
                r.analyst_ratings_hold,
                r.analyst_ratings_sell,
                r.analyst_ratings_strong_buy,
                r.analyst_ratings_strong_sell,
            ]
            .into_iter()
            .flatten()
            .sum::<i32>();
            count(Some(i64::from(total)), "analystRatings* (all five)")
        },
    )
    .await;

    m.check(
        "price_target_consensus",
        fundamentals::consensus::price_target_consensus("AAPL"),
        |rows| {
            let c = first(rows, "price target consensus")?;
            number(c.target_high, "targetHigh")?;
            number(c.target_low, "targetLow")?;
            number(c.target_consensus, "targetConsensus")?;
            number(c.target_median, "targetMedian")
        },
    )
    .await;

    m.check(
        "price_target_summary",
        fundamentals::consensus::price_target_summary("AAPL"),
        |rows| {
            let s = first(rows, "price target summary")?;
            count(s.all_time, "allTimeCount")?;
            count(s.last_year, "lastYearCount")?;
            number(s.all_time_avg_price_target, "allTimeAvgPriceTarget")?;
            number(s.last_year_avg_price_target, "lastYearAvgPriceTarget")
        },
    )
    .await;

    m.check(
        "upgrades_downgrades_consensus",
        fundamentals::consensus::upgrades_downgrades_consensus("AAPL"),
        |rows| {
            let c = first(rows, "rating consensus")?;
            text(c.consensus.as_deref(), "consensus")?;
            let total = [c.strong_buy, c.buy, c.hold, c.sell, c.strong_sell]
                .into_iter()
                .flatten()
                .sum::<i64>();
            count(Some(total), "grade counts (all five)")
        },
    )
    .await;

    m.check(
        "fetch_price_target_consensus_response",
        fundamentals::consensus::fetch_price_target_consensus_response("AAPL"),
        |c| {
            text(c.symbol.as_deref(), "symbol")?;
            number(c.target_consensus, "target_consensus")
        },
    )
    .await;

    m.check(
        "fetch_price_target_summary_response",
        fundamentals::consensus::fetch_price_target_summary_response("AAPL"),
        |s| {
            count(s.all_time_count, "all_time_count")?;
            number(s.all_time_avg, "all_time_avg")
        },
    )
    .await;

    m.check(
        "fetch_rating_consensus_response",
        fundamentals::consensus::fetch_rating_consensus_response("AAPL"),
        |c| text(c.consensus.as_deref(), "consensus"),
    )
    .await;

    // ---- CORPORATE ---------------------------------------------------------
    m.check(
        "historical_dividends",
        corporate::dividends_splits::historical_dividends("AAPL"),
        |resp| {
            let d = first(&resp.historical, "dividend")?;
            text(d.date.as_deref(), "date")?;
            number(d.dividend, "dividend")
        },
    )
    .await;

    m.check(
        "historical_splits",
        corporate::dividends_splits::historical_splits("AAPL"),
        |resp| {
            let s = first(&resp.historical, "split")?;
            text(s.date.as_deref(), "date")?;
            number(s.numerator, "numerator")?;
            number(s.denominator, "denominator")
        },
    )
    .await;

    m.check(
        "fetch_canonical_events",
        corporate::dividends_splits::fetch_canonical_events("AAPL"),
        |events| {
            if events.dividends.is_empty() && events.splits.is_empty() {
                return Err("no dividends and no splits".to_string());
            }
            Ok(())
        },
    )
    .await;

    m.check(
        "stock_news",
        corporate::news::stock_news("AAPL", 5),
        |rows| {
            let a = first(rows, "news")?;
            text(a.title.as_deref(), "title")?;
            text(a.url.as_deref(), "url")?;
            text(a.published_date.as_deref(), "publishedDate")?;
            text(a.site.as_deref().or(a.publisher.as_deref()), "site")
        },
    )
    .await;

    m.check(
        "fetch_canonical_news",
        corporate::news::fetch_canonical_news("AAPL", 5),
        |rows| {
            non_empty(rows, "news")?;
            let a = &rows[0];
            text(Some(a.title.as_str()), "title")?;
            text(Some(a.link.as_str()), "link")?;
            text(Some(a.time.as_str()), "time")?;
            text(Some(a.img.as_str()), "img")
        },
    )
    .await;

    m.check(
        "press_releases",
        corporate::news::press_releases("AAPL", 5),
        |rows| {
            let p = first(rows, "press release")?;
            text(p.title.as_deref(), "title")?;
            text(p.date.as_deref(), "publishedDate")?;
            text(p.text.as_deref(), "text")
        },
    )
    .await;

    m.check(
        "fetch_press_releases_response",
        corporate::news::fetch_press_releases_response("AAPL", 5),
        |rows| {
            let p = first(rows, "press release")?;
            text(p.date.as_deref(), "date")?;
            text(p.title.as_deref(), "title")
        },
    )
    .await;

    m.check(
        "insider_trading",
        corporate::insider_trading::insider_trading("AAPL", 5),
        |rows| {
            let t = first(rows, "insider trade")?;
            text(t.reporting_name.as_deref(), "reportingName")?;
            text(t.transaction_date.as_deref(), "transactionDate")?;
            text(t.transaction_type.as_deref(), "transactionType")?;
            text(t.link.as_deref(), "url")?;
            number(t.securities_transacted, "securitiesTransacted")
        },
    )
    .await;

    m.check(
        "executive_compensation",
        corporate::ownership::executive_compensation("AAPL"),
        |rows| {
            let c = first(rows, "executive compensation")?;
            text(c.name_and_position.as_deref(), "nameAndPosition")?;
            number(c.salary, "salary")?;
            number(c.stock_award, "stockAward")?;
            number(c.incentive_plan_compensation, "incentivePlanCompensation")?;
            number(c.all_other_compensation, "allOtherCompensation")?;
            number(c.total, "total")?;
            text(c.url.as_deref(), "link")
        },
    )
    .await;

    m.check(
        "fetch_executive_compensation_response",
        corporate::ownership::fetch_executive_compensation_response("AAPL"),
        |rows| {
            let c = first(rows, "executive compensation")?;
            number(c.total, "total")?;
            number(c.stock_award, "stock_award")?;
            text(c.url.as_deref(), "url")
        },
    )
    .await;

    m.check(
        "employee_count",
        corporate::ownership::employee_count("AAPL"),
        |rows| {
            let e = first(rows, "employee count")?;
            count(e.employee_count, "employeeCount")?;
            text(e.period_of_report.as_deref(), "periodOfReport")?;
            text(e.form_type.as_deref(), "formType")
        },
    )
    .await;

    m.check(
        "fetch_employee_count_response",
        corporate::ownership::fetch_employee_count_response("AAPL"),
        |rows| {
            let e = first(rows, "employee count")?;
            count(e.employee_count, "employee_count")?;
            text(e.period_of_report.as_deref(), "period_of_report")
        },
    )
    .await;

    m.check("stock_peers", quote::company::stock_peers("AAPL"), |rows| {
        let p = first(rows, "peer")?;
        text(p.symbol.as_deref(), "symbol")
    })
    .await;

    m.check(
        "fetch_canonical_similar_symbols",
        quote::company::fetch_canonical_similar_symbols("AAPL", 5),
        |rows| {
            non_empty(rows, "similar symbols")?;
            text(Some(rows[0].symbol.as_str()), "symbol")
        },
    )
    .await;

    m.check(
        "company_profile",
        quote::company::company_profile("AAPL"),
        |rows| {
            let p = first(rows, "profile")?;
            text(p.company_name.as_deref(), "companyName")?;
            number(p.mkt_cap, "marketCap")?;
            number(p.vol_avg, "averageVolume")?;
            number(p.price, "price")?;
            text(p.exchange.as_deref(), "exchange")?;
            text(p.exchange_full_name.as_deref(), "exchangeFullName")?;
            text(p.sector.as_deref(), "sector")
        },
    )
    .await;

    // ---- DISCOVERY ---------------------------------------------------------
    m.check(
        "stock_screener",
        discovery::screener::stock_screener(&[
            ("marketCapMoreThan", "100000000000"),
            ("limit", "5"),
        ]),
        |rows| {
            let r = first(rows, "screener match")?;
            text(r.symbol.as_deref(), "symbol")?;
            text(r.company_name.as_deref(), "companyName")?;
            number(r.market_cap, "marketCap")?;
            text(r.exchange_short_name.as_deref(), "exchangeShortName")
        },
    )
    .await;

    m.check(
        "symbol_search",
        discovery::screener::symbol_search("Apple", Some(5), None),
        |rows| {
            let r = first(rows, "search hit")?;
            text(r.symbol.as_deref(), "symbol")?;
            text(r.name.as_deref(), "name")?;
            text(r.exchange.as_deref(), "exchange")?;
            text(r.exchange_full_name.as_deref(), "exchangeFullName")
        },
    )
    .await;

    m.check(
        "fetch_symbol_search_response",
        discovery::screener::fetch_symbol_search_response("Apple", 5),
        |rows| {
            let r = first(rows, "symbol match")?;
            text(Some(r.symbol.as_str()), "symbol")?;
            text(r.exchange.as_deref(), "exchange")?;
            text(r.name.as_deref(), "name")
        },
    )
    .await;

    m.check(
        "fetch_screener_response",
        discovery::screener::fetch_screener_response(&screener),
        |rows| {
            let r = first(rows, "screener match")?;
            text(Some(r.symbol.as_str()), "symbol")?;
            number(r.market_cap, "market_cap")?;
            text(r.exchange.as_deref(), "exchange")
        },
    )
    .await;

    // ---- CALENDAR ----------------------------------------------------------
    m.check(
        "earnings_calendar",
        market::calendars::earnings_calendar(&from, &to),
        |rows| {
            non_empty(rows, "earnings calendar entries")?;
            let reported = rows
                .iter()
                .find(|e| e.eps.is_some())
                .ok_or("epsActual is null on every entry")?;
            number(reported.eps, "epsActual")?;
            let with_revenue = rows
                .iter()
                .find(|e| e.revenue.is_some())
                .ok_or("revenueActual is null on every entry")?;
            number(with_revenue.revenue, "revenueActual")?;
            text(rows[0].symbol.as_deref(), "symbol")
        },
    )
    .await;

    m.check(
        "ipo_calendar",
        market::calendars::ipo_calendar(&from, &to),
        |rows| {
            let e = first(rows, "IPO calendar entry")?;
            text(e.symbol.as_deref(), "symbol")?;
            text(e.company.as_deref(), "company")?;
            text(e.exchange.as_deref(), "exchange")
        },
    )
    .await;

    m.check(
        "stock_split_calendar",
        market::calendars::stock_split_calendar(&from, &to),
        |rows| {
            let e = first(rows, "split calendar entry")?;
            text(e.symbol.as_deref(), "symbol")?;
            number(e.numerator, "numerator")?;
            number(e.denominator, "denominator")
        },
    )
    .await;

    m.check(
        "dividend_calendar",
        market::calendars::dividend_calendar(&from, &to),
        |rows| {
            let e = first(rows, "dividend calendar entry")?;
            text(e.symbol.as_deref(), "symbol")?;
            number(e.dividend, "dividend")?;
            text(e.record_date.as_deref(), "recordDate")
        },
    )
    .await;

    m.check(
        "economic_calendar",
        market::calendars::economic_calendar(&from, &to),
        |rows| {
            let e = first(rows, "economic calendar entry")?;
            text(e.event.as_deref(), "event")?;
            text(e.country.as_deref(), "country")?;
            text(e.impact.as_deref(), "impact")
        },
    )
    .await;

    for kind in [
        CalendarKind::Earnings,
        CalendarKind::Ipo,
        CalendarKind::Dividend,
        CalendarKind::Split,
        CalendarKind::Economic,
    ] {
        m.check(
            "fetch_market_calendar_response",
            market::calendars::fetch_market_calendar_response(kind, &from, &to),
            |rows| {
                non_empty(rows, "calendar entries")?;
                text(rows[0].date.as_deref(), "date")
            },
        )
        .await;
    }

    // ---- MARKET ------------------------------------------------------------
    m.check(
        "sectors_pe",
        market::market_performance::sectors_pe(),
        |rows| {
            let s = first(rows, "sector PE")?;
            text(s.sector.as_deref(), "sector")?;
            text(s.exchange.as_deref(), "exchange")?;
            number(s.pe, "pe")
        },
    )
    .await;

    m.check(
        "industries_pe",
        market::market_performance::industries_pe(),
        |rows| {
            let i = first(rows, "industry PE")?;
            text(i.industry.as_deref(), "industry")?;
            text(i.exchange.as_deref(), "exchange")?;
            number(i.pe, "pe")
        },
    )
    .await;

    m.check(
        "sector_performance",
        market::market_performance::sector_performance(),
        |rows| {
            let s = first(rows, "sector performance")?;
            text(s.sector.as_deref(), "sector")?;
            text(s.exchange.as_deref(), "exchange")?;
            text(s.date.as_deref(), "date")?;
            present(s.average_change, "averageChange")
        },
    )
    .await;

    m.check(
        "historical_sector_performance",
        market::market_performance::historical_sector_performance(&recent_from, &to),
        |rows| {
            let s = first(rows, "historical sector performance")?;
            text(s.date.as_deref(), "date")?;
            text(s.sector.as_deref(), "sector")?;
            text(s.exchange.as_deref(), "exchange")?;
            present(s.average_change, "averageChange")
        },
    )
    .await;

    m.check(
        "fetch_sector_performance_response",
        market::market_performance::fetch_sector_performance_response(),
        |rows| {
            non_empty(rows, "sector performance")?;
            let distinct: std::collections::HashSet<_> = rows
                .iter()
                .map(|s| (s.sector.as_str(), s.exchange.as_deref()))
                .collect();
            if distinct.len() != rows.len() {
                return Err(format!(
                    "{} rows collapse to {} distinct (sector, exchange) pairs",
                    rows.len(),
                    distinct.len()
                ));
            }
            text(rows[0].exchange.as_deref(), "exchange")
        },
    )
    .await;

    m.check(
        "fetch_sector_performance_history_response",
        market::market_performance::fetch_sector_performance_history_response(5),
        |days| {
            non_empty(days, "sector performance history")?;
            let day = &days[0];
            text(day.date.as_deref(), "date")?;
            // Upstream reports each sector once per exchange. If a date holds
            // no more entries than it holds sectors, one exchange row
            // overwrote the others on the way through.
            let sectors: std::collections::HashSet<_> =
                day.sectors.iter().map(|s| s.sector.as_str()).collect();
            if day.sectors.len() <= sectors.len() {
                return Err(format!(
                    "date {:?} carries {} rows for {} sectors, so the exchange dimension was collapsed",
                    day.date,
                    day.sectors.len(),
                    sectors.len()
                ));
            }
            text(day.sectors[0].exchange.as_deref(), "exchange")
        },
    )
    .await;

    for direction in [
        MoverDirection::Gainers,
        MoverDirection::Losers,
        MoverDirection::MostActive,
    ] {
        m.check(
            "fetch_market_movers_response",
            market::market_performance::fetch_market_movers_response(direction),
            |rows| {
                let q = first(rows, "mover")?;
                text(Some(q.symbol.as_str()), "symbol")?;
                number(q.price, "price")?;
                present(q.change_percent, "change_percent")?;
                text(q.exchange.as_deref(), "exchange")
            },
        )
        .await;
    }

    m.check(
        "fetch_sector_pe_response",
        market::market_performance::fetch_sector_pe_response(),
        |rows| {
            let s = first(rows, "sector PE")?;
            number(s.pe, "pe")?;
            text(s.exchange.as_deref(), "exchange")
        },
    )
    .await;

    m.check(
        "fetch_industry_pe_response",
        market::market_performance::fetch_industry_pe_response(),
        |rows| {
            let i = first(rows, "industry PE")?;
            number(i.pe, "pe")?;
            text(i.exchange.as_deref(), "exchange")
        },
    )
    .await;

    // ---- INDICES, FOREX, COMMODITIES, CRYPTO -------------------------------
    m.check(
        "fetch_canonical_index_quote",
        indices::fetch_canonical_index_quote("^GSPC"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")
        },
    )
    .await;

    m.check(
        "sp500_constituents",
        indices::sp500_constituents(),
        |rows| {
            let c = first(rows, "constituent")?;
            text(c.symbol.as_deref(), "symbol")?;
            text(c.name.as_deref(), "name")?;
            text(c.sector.as_deref(), "sector")
        },
    )
    .await;

    for index in [
        MajorIndex::Sp500,
        MajorIndex::Nasdaq100,
        MajorIndex::DowJones,
    ] {
        m.check(
            "fetch_index_constituents_response",
            indices::fetch_index_constituents_response(index),
            |rows| {
                let c = first(rows, "constituent")?;
                text(Some(c.symbol.as_str()), "symbol")?;
                text(c.name.as_deref(), "name")
            },
        )
        .await;
    }

    m.check(
        "fetch_index_constituent_changes_response",
        indices::fetch_index_constituent_changes_response(MajorIndex::Sp500),
        |rows| {
            let c = first(rows, "constituent change")?;
            text(c.date.as_deref(), "date")?;
            text(c.symbol.as_deref(), "symbol")
        },
    )
    .await;

    m.check(
        "fetch_canonical_forex_quote",
        forex::fetch_canonical_forex_quote("EUR", "USD"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")?;
            present(q.timestamp, "timestamp")
        },
    )
    .await;

    m.check(
        "fetch_canonical_commodity_quote",
        super::commodities::fetch_canonical_commodity_quote("GCUSD"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")
        },
    )
    .await;

    m.check(
        "fetch_canonical_crypto_quote",
        super::crypto::fetch_canonical_crypto_quote("BTC", "USD"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")
        },
    )
    .await;

    // ---- AUTH --------------------------------------------------------------
    m.total += 1;
    let invalid_client = FmpClientBuilder::new("invalid-live-test-key")
        .build_with_limiter(Arc::new(RateLimiter::new(5.0)))
        .expect("invalid-key test client must build");
    match invalid_client
        .get_raw("/stable/quote", &[("symbol", "AAPL")])
        .await
    {
        Err(FinanceError::AuthenticationFailed { .. }) => {
            m.passed += 1;
            println!("ok: invalid_api_key_error");
        }
        result => {
            let problem = format!("expected AuthenticationFailed, got {result:?}");
            m.fail("invalid_api_key_error", &problem);
        }
    }

    m.finish();
}
