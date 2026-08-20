//! Live contract matrix for the routed Massive (Polygon) surface.
//!
//! Every check asserts that at least one semantically-required field arrived
//! populated. Because every DTO field is `Option<T>`, a path or key that moved
//! upstream deserializes to `None` rather than erroring, so asserting only
//! `Ok(_)` would pass on an entirely null response. A failure names the
//! endpoint and the field that was empty.
//!
//! Entitlement is treated separately from correctness: a check may only be
//! recorded as plan-limited when the upstream message actually says the plan
//! lacks the data. A bare invalid-key or not-authorized response is a failure,
//! since the client maps a wrong-but-plausible path to the same error variant.

use std::future::Future;

use ::futures::StreamExt;
use chrono::{Days, TimeZone, Utc};

use super::{
    chart, corporate, crypto, discovery, economic, filings, forex, fundamentals, futures, indices,
    market, options, quote,
};
use crate::{FinanceError, Frequency, Interval, StatementType, TimeRange, error::Result};

type Check = std::result::Result<(), String>;

/// Substrings Massive uses when the account is not entitled to a dataset.
/// A plan gap is the only reason an auth error counts as anything but failure.
const PLAN_MARKERS: [&str; 2] = ["not entitled", "upgrade your plan"];

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
    let redacted = redact("GET /v3/reference/tickers?apiKey=pg-live-1&limit=1");
    assert_eq!(
        redacted,
        "GET /v3/reference/tickers?apiKey=REDACTED&limit=1"
    );
    assert!(!redacted.contains("pg-live"));
}

#[test]
fn only_an_entitlement_message_counts_as_plan_limited() {
    assert!(is_plan_gap("You are not entitled to this data"));
    assert!(is_plan_gap("Please upgrade your plan"));
    assert!(!is_plan_gap("Unknown API Key"));
    assert!(!is_plan_gap("NOT_AUTHORIZED"));
}

fn first<'a, T>(rows: &'a [T], what: &str) -> std::result::Result<&'a T, String> {
    rows.first()
        .ok_or_else(|| format!("returned no {what} rows"))
}

fn results<'a, T>(
    page: &'a super::models::PaginatedResponseDTO<T>,
    what: &str,
) -> std::result::Result<&'a T, String> {
    first(page.results.as_deref().unwrap_or_default(), what)
}

fn text(value: Option<&str>, field: &str) -> Check {
    match value {
        Some(v) if !v.trim().is_empty() => Ok(()),
        Some(_) => Err(format!("{field} is empty")),
        None => Err(format!("{field} is null")),
    }
}

fn number(value: Option<f64>, field: &str) -> Check {
    match value {
        Some(v) if v.is_finite() && v != 0.0 => Ok(()),
        Some(v) => Err(format!("{field} is degenerate ({v})")),
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

/// Timestamps in Massive responses are nanoseconds or milliseconds depending on
/// the endpoint; either way a populated one is far past the epoch.
fn timestamp(value: Option<i64>, field: &str) -> Check {
    match value {
        Some(v) if v > 0 => Ok(()),
        Some(v) => Err(format!("{field} is degenerate ({v})")),
        None => Err(format!("{field} is null")),
    }
}

struct Matrix {
    failures: Vec<String>,
    passed: usize,
    plan_limited: usize,
    total: usize,
}

impl Matrix {
    fn new() -> Self {
        Self {
            failures: Vec::new(),
            passed: 0,
            plan_limited: 0,
            total: 0,
        }
    }

    async fn check<T, F>(&mut self, name: &str, future: F, require: impl FnOnce(&T) -> Check)
    where
        F: Future<Output = Result<T>>,
    {
        self.total += 1;
        self.record(name, future.await, require, false);
    }

    /// Like [`Self::check`], but tolerates an explicit plan-entitlement refusal.
    async fn check_plan<T, F>(&mut self, name: &str, future: F, require: impl FnOnce(&T) -> Check)
    where
        F: Future<Output = Result<T>>,
    {
        self.total += 1;
        self.record(name, future.await, require, true);
    }

    fn record<T>(
        &mut self,
        name: &str,
        outcome: Result<T>,
        require: impl FnOnce(&T) -> Check,
        allow_plan_gap: bool,
    ) {
        match outcome {
            Ok(value) => match require(&value) {
                Ok(()) => {
                    self.passed += 1;
                    println!("ok: {name}");
                }
                Err(problem) => self.fail(name, &problem),
            },
            Err(FinanceError::AuthenticationFailed { ref context })
                if allow_plan_gap && is_plan_gap(context) =>
            {
                self.plan_limited += 1;
                println!("plan-limited: {name}");
            }
            Err(error) => self.fail(name, &error.to_string()),
        }
    }

    fn fail(&mut self, name: &str, problem: &str) {
        let line = format!("{name}: {}", redact(problem));
        eprintln!("failed: {line}");
        self.failures.push(line);
    }

    fn skip(&mut self, name: &str, reason: &str) {
        self.total += 1;
        self.fail(name, reason);
    }

    fn finish(self) {
        println!(
            "Polygon live matrix: {} data checks passed, {} plan-limited, {} total",
            self.passed, self.plan_limited, self.total
        );
        assert!(
            self.failures.is_empty(),
            "{} Polygon live checks failed:\n{}",
            self.failures.len(),
            self.failures.join("\n")
        );
    }
}

fn is_plan_gap(context: &str) -> bool {
    let lowered = context.to_ascii_lowercase();
    PLAN_MARKERS.iter().any(|marker| lowered.contains(marker))
}

#[tokio::test]
#[ignore = "requires POLYGON_API_KEY and consumes live Massive API quota"]
#[allow(clippy::too_many_lines)]
async fn all_routed_polygon_endpoints_return_populated_data() {
    let api_key = std::env::var("POLYGON_API_KEY").expect("POLYGON_API_KEY must be set");
    super::init_with_timeout(api_key.clone(), std::time::Duration::from_secs(120))
        .expect("Polygon client must initialize");

    let today = Utc::now().date_naive();
    let from = today.checked_sub_days(Days::new(30)).unwrap().to_string();
    let to = today.checked_sub_days(Days::new(1)).unwrap().to_string();
    let midnight = |days| {
        Utc.from_utc_datetime(
            &today
                .checked_sub_days(Days::new(days))
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .timestamp()
    };
    let (from_ts, to_ts) = (midnight(30), midnight(1));

    let mut m = Matrix::new();

    // ---- CHART -------------------------------------------------------------
    m.check(
        "stock_aggregates",
        chart::stock_aggregates("AAPL", 1, super::models::Timespan::Day, &from, &to, None),
        |resp| {
            let bars = resp.results.as_deref().unwrap_or_default();
            let bar = first(bars, "aggregate bar")?;
            number(Some(bar.close), "close")?;
            number(Some(bar.volume), "volume")?;
            timestamp(Some(bar.timestamp), "timestamp")
        },
    )
    .await;

    m.check(
        "fetch_chart_response",
        chart::fetch_chart_response("AAPL", Interval::OneDay, TimeRange::OneMonth),
        |chart| {
            non_empty(&chart.candles, "candles")?;
            number(Some(chart.candles[0].close), "close")?;
            timestamp(Some(chart.candles[0].timestamp), "timestamp")
        },
    )
    .await;

    m.check(
        "fetch_chart_range_response",
        chart::fetch_chart_range_response("AAPL", Interval::OneDay, from_ts, to_ts),
        |chart| {
            non_empty(&chart.candles, "candles")?;
            number(Some(chart.candles[0].close), "close")
        },
    )
    .await;

    m.check(
        "stock_previous_close",
        chart::stock_previous_close("AAPL", Some(true)),
        |resp| {
            let bar = first(resp.results.as_deref().unwrap_or_default(), "previous bar")?;
            number(Some(bar.close), "close")
        },
    )
    .await;

    m.check(
        "stock_grouped_daily",
        chart::stock_grouped_daily(&to, Some(true)),
        |resp| {
            let bars = resp.results.as_deref().unwrap_or_default();
            let bar = first(bars, "grouped daily bar")?;
            text(bar.ticker.as_deref(), "ticker")?;
            number(Some(bar.close), "close")
        },
    )
    .await;

    m.check(
        "stock_daily_open_close",
        chart::stock_daily_open_close("AAPL", &to, Some(true)),
        |resp| {
            text(resp.symbol.as_deref(), "symbol")?;
            number(resp.close, "close")?;
            number(resp.open, "open")
        },
    )
    .await;

    // ---- CORPORATE ---------------------------------------------------------
    m.check(
        "stock_dividends",
        corporate::stock_dividends(&[("ticker", "AAPL"), ("limit", "2")]),
        |page| {
            let d = results(page, "dividend")?;
            text(d.ex_dividend_date.as_deref(), "ex_dividend_date")?;
            number(d.cash_amount, "cash_amount")
        },
    )
    .await;

    m.check(
        "stock_splits",
        corporate::stock_splits(&[("ticker", "AAPL"), ("limit", "2")]),
        |page| {
            let s = results(page, "split")?;
            text(s.execution_date.as_deref(), "execution_date")?;
            number(s.split_from, "split_from")?;
            number(s.split_to, "split_to")
        },
    )
    .await;

    m.check(
        "fetch_events_response",
        corporate::fetch_events_response("AAPL"),
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
        corporate::stock_news(&[("ticker", "AAPL"), ("limit", "2")]),
        |page| {
            let a = results(page, "news article")?;
            text(a.title.as_deref(), "title")?;
            text(a.article_url.as_deref(), "article_url")?;
            text(a.published_utc.as_deref(), "published_utc")
        },
    )
    .await;

    m.check(
        "fetch_news_response",
        corporate::fetch_news_response("AAPL"),
        |rows| {
            non_empty(rows, "news")?;
            text(Some(rows[0].title.as_str()), "title")?;
            text(Some(rows[0].link.as_str()), "link")
        },
    )
    .await;

    // ---- FUNDAMENTALS ------------------------------------------------------
    m.check(
        "income_statements",
        fundamentals::income_statements("AAPL", &[("timeframe", "annual"), ("limit", "2")]),
        |page| {
            let s = results(page, "income statement")?;
            text(s.period.period_end.as_deref(), "period_end")?;
            text(s.period.timeframe.as_deref(), "timeframe")?;
            number(s.revenue, "revenue")?;
            number(s.gross_profit, "gross_profit")?;
            number(s.operating_income, "operating_income")?;
            number(s.diluted_earnings_per_share, "diluted_earnings_per_share")
        },
    )
    .await;

    m.check(
        "balance_sheets",
        fundamentals::balance_sheets("AAPL", &[("timeframe", "annual"), ("limit", "2")]),
        |page| {
            let s = results(page, "balance sheet")?;
            text(s.period.period_end.as_deref(), "period_end")?;
            number(s.total_assets, "total_assets")?;
            number(s.total_liabilities, "total_liabilities")?;
            number(s.total_equity, "total_equity")
        },
    )
    .await;

    m.check(
        "cash_flow_statements",
        fundamentals::cash_flow_statements("AAPL", &[("timeframe", "annual"), ("limit", "2")]),
        |page| {
            let s = results(page, "cash flow statement")?;
            text(s.period.period_end.as_deref(), "period_end")?;
            number(s.net_income, "net_income")?;
            number(
                s.net_cash_from_operating_activities,
                "net_cash_from_operating_activities",
            )
        },
    )
    .await;

    for statement in [
        StatementType::Income,
        StatementType::Balance,
        StatementType::CashFlow,
    ] {
        m.check(
            "fetch_financials_response",
            fundamentals::fetch_financials_response("AAPL", statement, Frequency::Annual),
            |stmt| {
                if stmt.statement.is_empty() {
                    return Err("pivoted no metrics".to_string());
                }
                if stmt
                    .statement
                    .values()
                    .all(std::collections::HashMap::is_empty)
                {
                    return Err("every metric has no periods".to_string());
                }
                Ok(())
            },
        )
        .await;
    }

    m.check(
        "stock_short_interest",
        fundamentals::stock_short_interest("AAPL", &[("limit", "2")]),
        |page| {
            let s = results(page, "short interest")?;
            text(s.settlement_date.as_deref(), "settlement_date")?;
            number(s.short_interest, "short_interest")
        },
    )
    .await;

    m.check(
        "stock_short_volume",
        fundamentals::stock_short_volume("AAPL", &[("limit", "2")]),
        |page| {
            let s = results(page, "short volume")?;
            text(s.date.as_deref(), "date")?;
            number(s.short_volume, "short_volume")?;
            number(s.total_volume, "total_volume")
        },
    )
    .await;

    m.check("stock_float", fundamentals::stock_float("AAPL"), |page| {
        let f = results(page, "float")?;
        text(f.ticker.as_deref(), "ticker")?;
        number(f.free_float, "free_float")?;
        number(f.free_float_percent, "free_float_percent")?;
        text(f.effective_date.as_deref(), "effective_date")
    })
    .await;

    m.check(
        "fetch_short_interest_response",
        fundamentals::fetch_short_interest_response("AAPL"),
        |rows| {
            let s = first(rows, "short interest")?;
            number(s.short_interest, "short_interest")?;
            number(s.days_to_cover, "days_to_cover")
        },
    )
    .await;

    m.check(
        "fetch_short_volume_response",
        fundamentals::fetch_short_volume_response("AAPL"),
        |rows| {
            let s = first(rows, "short volume")?;
            number(s.short_volume, "short_volume")
        },
    )
    .await;

    m.check(
        "fetch_share_float_response",
        fundamentals::fetch_share_float_response("AAPL"),
        |f| {
            number(f.float_shares, "float_shares")?;
            number(f.float_percent, "float_percent")?;
            // Massive reports no shares-outstanding figure, so this route must
            // leave it null rather than deriving one.
            if f.outstanding_shares.is_some() {
                return Err("outstanding_shares is populated but unreported upstream".to_string());
            }
            text(f.date.as_deref(), "date")
        },
    )
    .await;

    // ---- DISCOVERY ---------------------------------------------------------
    m.check(
        "all_tickers",
        discovery::all_tickers(&[("ticker", "AAPL"), ("limit", "1")]),
        |page| {
            let t = results(page, "ticker")?;
            text(t.ticker.as_deref(), "ticker")?;
            text(t.name.as_deref(), "name")?;
            text(t.primary_exchange.as_deref(), "primary_exchange")
        },
    )
    .await;

    m.check(
        "ticker_details",
        discovery::ticker_details("AAPL"),
        |resp| present(resp.results.as_ref(), "results"),
    )
    .await;

    m.check(
        "ticker_types",
        discovery::ticker_types(&[("limit", "2")]),
        |page| {
            let t = results(page, "ticker type")?;
            text(t.code.as_deref(), "code")?;
            text(t.description.as_deref(), "description")
        },
    )
    .await;

    m.check(
        "related_tickers",
        discovery::related_tickers("AAPL"),
        |page| {
            let t = results(page, "related ticker")?;
            text(t.ticker.as_deref(), "ticker")
        },
    )
    .await;

    m.check(
        "fetch_similar_symbols_response",
        discovery::fetch_similar_symbols_response("AAPL", 5),
        |rows| {
            non_empty(rows, "similar symbols")?;
            text(Some(rows[0].symbol.as_str()), "symbol")
        },
    )
    .await;

    m.check(
        "fetch_symbol_search_response",
        discovery::fetch_symbol_search_response("Apple", 5),
        |rows| {
            let r = first(rows, "symbol match")?;
            text(Some(r.symbol.as_str()), "symbol")?;
            text(r.name.as_deref(), "name")
        },
    )
    .await;

    m.check(
        "fetch_symbol_details_response",
        discovery::fetch_symbol_details_response("AAPL"),
        |details| text(Some(details.symbol.as_str()), "symbol"),
    )
    .await;

    m.check(
        "exchanges",
        discovery::exchanges(&[("limit", "2")]),
        |page| {
            let e = results(page, "exchange")?;
            text(e.name.as_deref(), "name")?;
            text(e.asset_class.as_deref(), "asset_class")
        },
    )
    .await;

    m.check(
        "fetch_exchanges_response",
        discovery::fetch_exchanges_response(),
        |rows| {
            non_empty(rows, "exchanges")?;
            text(rows[0].name.as_deref(), "name")
        },
    )
    .await;

    m.check("market_holidays", discovery::market_holidays(), |rows| {
        let h = first(rows, "holiday")?;
        text(h.name.as_deref(), "name")?;
        text(h.date.as_deref(), "date")?;
        text(h.exchange.as_deref(), "exchange")
    })
    .await;

    m.check(
        "fetch_market_holidays_response",
        discovery::fetch_market_holidays_response(),
        |rows| {
            non_empty(rows, "holidays")?;
            text(rows[0].date.as_deref(), "date")
        },
    )
    .await;

    m.check("market_status", discovery::market_status(), |status| {
        present(status.exchanges.as_ref(), "exchanges")
    })
    .await;

    m.check(
        "condition_codes",
        discovery::condition_codes(&[("limit", "2")]),
        |page| {
            non_empty(
                page.results.as_deref().unwrap_or_default(),
                "condition codes",
            )
        },
    )
    .await;

    // ---- ECONOMIC ----------------------------------------------------------
    m.check(
        "inflation",
        economic::inflation(&[("limit", "2")]),
        |page| {
            let p = results(page, "inflation observation")?;
            text(p.date.as_deref(), "date")?;
            number(p.cpi.or(p.cpi_year_over_year), "cpi")
        },
    )
    .await;

    m.check(
        "inflation_expectations",
        economic::inflation_expectations(&[("limit", "2")]),
        |page| {
            let p = results(page, "inflation expectation")?;
            text(p.date.as_deref(), "date")
        },
    )
    .await;

    m.check(
        "labor_market",
        economic::labor_market(&[("limit", "2")]),
        |page| {
            let p = results(page, "labor market observation")?;
            text(p.date.as_deref(), "date")
        },
    )
    .await;

    m.check(
        "treasury_yields",
        economic::treasury_yields(&[("limit", "2")]),
        |page| {
            let p = results(page, "treasury yield")?;
            text(p.date.as_deref(), "date")
        },
    )
    .await;

    for series in [
        "inflation",
        "inflation_expectations",
        "labor_market",
        "treasury_yields",
    ] {
        m.check(
            "fetch_economic_series_response",
            economic::fetch_economic_series_response(series),
            |s| {
                non_empty(&s.observations, "observations")?;
                if s.observations.iter().all(|o| o.value.is_none()) {
                    return Err("every observation value is null".to_string());
                }
                Ok(())
            },
        )
        .await;
    }

    // ---- FILINGS -----------------------------------------------------------
    let ten_k_accession = {
        m.total += 1;
        match filings::sec_edgar_index(&[("ticker", "AAPL"), ("form_type", "10-K"), ("limit", "1")])
            .await
        {
            Ok(page) => match page
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.accession_number)
            {
                Some(accession) => {
                    m.passed += 1;
                    println!("ok: sec_edgar_index_10k");
                    Some(accession)
                }
                None => {
                    m.fail("sec_edgar_index_10k", "accession_number is null");
                    None
                }
            },
            Err(error) => {
                m.fail("sec_edgar_index_10k", &error.to_string());
                None
            }
        }
    };

    m.check(
        "fetch_filings_response",
        filings::fetch_filings_response("AAPL"),
        |f| {
            text(Some(f.symbol.as_str()), "symbol")?;
            non_empty(&f.filings, "filings")
        },
    )
    .await;

    m.check(
        "risk_factors",
        filings::risk_factors(&[("ticker", "AAPL"), ("limit", "2")]),
        |page| {
            let r = results(page, "risk factor")?;
            text(r.primary_category.as_deref(), "primary_category")?;
            text(r.supporting_text.as_deref(), "supporting_text")
        },
    )
    .await;

    m.check(
        "fetch_risk_factors_response",
        filings::fetch_risk_factors_response("AAPL"),
        |rows| non_empty(rows, "risk factors"),
    )
    .await;

    if let Some(accession) = ten_k_accession.as_deref() {
        m.check(
            "filing_10k_sections",
            filings::filing_10k_sections(accession, &[("ticker", "AAPL"), ("limit", "2")]),
            |page| {
                let s = first(
                    page.results.as_deref().unwrap_or_default(),
                    "filing section",
                )?;
                text(s.section.as_deref(), "section")?;
                text(s.content.as_deref(), "content")
            },
        )
        .await;

        m.check(
            "fetch_filing_sections_response",
            filings::fetch_filing_sections_response(
                accession,
                crate::models::filings::FilingSectionForm::TenK,
            ),
            |sections| non_empty(sections, "filing sections"),
        )
        .await;
    } else {
        m.skip("filing_10k_sections", "no 10-K accession returned");
        m.skip(
            "fetch_filing_sections_response",
            "no 10-K accession returned",
        );
    }

    // ---- FOREX -------------------------------------------------------------
    m.check(
        "forex_aggregates",
        forex::aggregates::forex_aggregates(
            "C:EURUSD",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        ),
        |resp| {
            let bar = first(resp.results.as_deref().unwrap_or_default(), "forex bar")?;
            number(Some(bar.close), "close")
        },
    )
    .await;

    m.check(
        "forex_previous_close",
        forex::aggregates::forex_previous_close("C:EURUSD", Some(true)),
        |resp| {
            let bar = first(resp.results.as_deref().unwrap_or_default(), "forex bar")?;
            number(Some(bar.close), "close")
        },
    )
    .await;

    m.check_plan(
        "fetch_forex_quote_response",
        forex::quotes::fetch_forex_quote_response("EUR", "USD"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.bid.or(q.price), "bid")
        },
    )
    .await;

    m.check_plan(
        "forex_last_quote",
        forex::quotes::forex_last_quote("EUR", "USD"),
        |resp| {
            let last = resp.last.as_ref().ok_or("last is null")?;
            number(last.bid, "bid")?;
            number(last.ask, "ask")
        },
    )
    .await;

    m.check_plan(
        "forex_snapshot",
        forex::snapshots::forex_snapshot("C:EURUSD"),
        |resp| {
            let t = resp.ticker.as_ref().ok_or("ticker is null")?;
            text(t.ticker.as_deref(), "ticker")
        },
    )
    .await;

    // ---- CRYPTO ------------------------------------------------------------
    m.check(
        "crypto_aggregates",
        crypto::aggregates::crypto_aggregates(
            "X:BTCUSD",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        ),
        |resp| {
            let bar = first(resp.results.as_deref().unwrap_or_default(), "crypto bar")?;
            number(Some(bar.close), "close")?;
            number(Some(bar.volume), "volume")
        },
    )
    .await;

    m.check(
        "crypto_daily_open_close",
        crypto::aggregates::crypto_daily_open_close("BTC", "USD", &to),
        |resp| number(resp.close, "close"),
    )
    .await;

    m.check_plan(
        "fetch_crypto_quote_response",
        crypto::snapshots::fetch_crypto_quote_response("BTC", "USD"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")
        },
    )
    .await;

    m.check_plan(
        "crypto_last_trade",
        crypto::trades::crypto_last_trade("BTC", "USD"),
        |resp| present(resp.last.as_ref(), "last"),
    )
    .await;

    // ---- INDICES -----------------------------------------------------------
    m.check_plan(
        "index_aggregates",
        indices::aggregates::index_aggregates(
            "I:SPX",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        ),
        |resp| {
            let bar = first(resp.results.as_deref().unwrap_or_default(), "index bar")?;
            number(Some(bar.close), "close")
        },
    )
    .await;

    m.check_plan(
        "index_snapshot",
        indices::snapshots::index_snapshot("I:SPX"),
        |resp| {
            let s = first(
                resp.results.as_deref().unwrap_or_default(),
                "index snapshot",
            )?;
            text(s.ticker.as_deref(), "ticker")?;
            number(s.value, "value")
        },
    )
    .await;

    m.check_plan(
        "fetch_indices_quote_response",
        indices::snapshots::fetch_indices_quote_response("I:SPX"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            number(q.price, "price")
        },
    )
    .await;

    // ---- FUTURES -----------------------------------------------------------
    let futures_ticker = {
        m.total += 1;
        match futures::reference::futures_contracts(&[
            ("product_code", "ES"),
            ("active", "true"),
            ("limit", "1"),
        ])
        .await
        {
            Ok(page) => match page
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.ticker)
            {
                Some(ticker) => {
                    m.passed += 1;
                    println!("ok: futures_contracts");
                    Some(ticker)
                }
                None => {
                    m.fail("futures_contracts", "ticker is null");
                    None
                }
            },
            Err(error) => {
                m.fail("futures_contracts", &error.to_string());
                None
            }
        }
    };

    m.check(
        "futures_products",
        futures::reference::futures_products(&[("product_code", "ES"), ("limit", "2")]),
        |page| {
            let p = results(page, "futures product")?;
            text(p.name.as_deref(), "name")?;
            text(p.asset_class.as_deref(), "asset_class")
        },
    )
    .await;

    m.check(
        "futures_exchanges",
        futures::reference::futures_exchanges(&[("limit", "2")]),
        |page| {
            let e = results(page, "futures exchange")?;
            text(e.name.as_deref(), "name")?;
            text(e.mic.as_deref(), "mic")
        },
    )
    .await;

    m.check(
        "futures_market_status",
        futures::reference::futures_market_status(&[("product_code", "ES"), ("limit", "2")]),
        |page| {
            let s = results(page, "futures market status")?;
            text(s.market_event.as_deref(), "market_event")?;
            text(s.product_code.as_deref(), "product_code")
        },
    )
    .await;

    if let Some(ticker) = futures_ticker.as_deref() {
        m.check(
            "futures_aggregates",
            futures::aggregates::futures_aggregates(ticker, "1session", &[("limit", "2")]),
            |page| {
                let bar = results(page, "futures bar")?;
                text(bar.ticker.as_deref(), "ticker")?;
                number(bar.close, "close")?;
                text(bar.session_end_date.as_deref(), "session_end_date")?;
                timestamp(bar.window_start, "window_start")
            },
        )
        .await;

        m.check_plan(
            "futures_trades",
            futures::trades::futures_trades(ticker, &[("limit", "2")]),
            |page| {
                let t = results(page, "futures trade")?;
                number(t.price, "price")?;
                timestamp(t.timestamp, "timestamp")
            },
        )
        .await;

        m.check_plan(
            "futures_snapshot",
            futures::snapshots::futures_snapshot(ticker),
            |page| {
                let s = results(page, "futures snapshot")?;
                text(s.ticker.as_deref(), "ticker")?;
                present(s.session.as_ref(), "session")
            },
        )
        .await;

        m.check_plan(
            "fetch_futures_quote_response",
            futures::snapshots::fetch_futures_quote_response(ticker),
            |q| {
                text(Some(q.symbol.as_str()), "symbol")?;
                number(q.price, "price")?;
                // Nanoseconds would put this ~10^9 beyond any plausible epoch
                // second, so the scaling regression shows up here.
                match q.timestamp {
                    Some(ts) if (1_600_000_000..4_000_000_000).contains(&ts) => Ok(()),
                    Some(ts) => Err(format!("timestamp {ts} is not a plausible epoch second")),
                    None => Err("timestamp is null".to_string()),
                }
            },
        )
        .await;
    } else {
        for name in [
            "futures_aggregates",
            "futures_trades",
            "futures_snapshot",
            "fetch_futures_quote_response",
        ] {
            m.skip(name, "no active ES contract returned");
        }
    }

    // ---- OPTIONS -----------------------------------------------------------
    let option_ticker = {
        m.total += 1;
        match options::reference::options_contracts(&[
            ("underlying_ticker", "AAPL"),
            ("expired", "false"),
            ("limit", "1"),
        ])
        .await
        {
            Ok(page) => match page
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.ticker)
            {
                Some(ticker) => {
                    m.passed += 1;
                    println!("ok: options_contracts");
                    Some(ticker)
                }
                None => {
                    m.fail("options_contracts", "ticker is null");
                    None
                }
            },
            Err(error) => {
                m.fail("options_contracts", &error.to_string());
                None
            }
        }
    };

    m.check_plan(
        "options_chain_snapshot",
        options::snapshots::options_chain_snapshot("AAPL", &[("limit", "1")]),
        |page| {
            non_empty(
                page.results.as_deref().unwrap_or_default(),
                "option contracts",
            )
        },
    )
    .await;

    m.check_plan(
        "fetch_options_response",
        options::snapshots::fetch_options_response("AAPL", None),
        |chain| non_empty(&chain.expiration_dates(), "expiration dates"),
    )
    .await;

    if let Some(ticker) = option_ticker.as_deref() {
        m.check_plan(
            "options_aggregates",
            options::aggregates::options_aggregates(
                ticker,
                1,
                super::models::Timespan::Day,
                &from,
                &to,
                None,
            ),
            |resp| {
                let bar = first(resp.results.as_deref().unwrap_or_default(), "option bar")?;
                number(Some(bar.close), "close")
            },
        )
        .await;

        m.check_plan(
            "options_last_trade",
            options::trades::options_last_trade(ticker),
            |resp| {
                let last = resp.results.as_ref().ok_or("results is null")?;
                number(last.price, "price")
            },
        )
        .await;

        m.check_plan(
            "options_contract_snapshot",
            options::snapshots::options_contract_snapshot("AAPL", ticker),
            |resp| present(resp.results.as_ref(), "results"),
        )
        .await;
    } else {
        for name in [
            "options_aggregates",
            "options_last_trade",
            "options_contract_snapshot",
        ] {
            m.skip(name, "no active AAPL contract returned");
        }
    }

    // ---- QUOTE -------------------------------------------------------------
    m.check_plan("stock_snapshot", quote::stock_snapshot("AAPL"), |resp| {
        let t = resp.ticker.as_ref().ok_or("ticker is null")?;
        text(t.ticker.as_deref(), "ticker")?;
        let day = t.day.as_ref().ok_or("day is null")?;
        number(day.close, "day.close")
    })
    .await;

    m.check_plan(
        "fetch_quote_response",
        quote::fetch_quote_response("AAPL"),
        |q| {
            text(Some(q.symbol.as_str()), "symbol")?;
            let price = q.price.as_ref().ok_or("price block is null")?;
            number(
                price.regular_market_price.as_ref().and_then(|v| v.raw),
                "price.regularMarketPrice",
            )
        },
    )
    .await;

    m.check_plan(
        "fetch_quotes_batch_response",
        quote::fetch_quotes_batch_response(&["AAPL", "MSFT"]),
        |rows| {
            non_empty(rows, "batch quotes")?;
            text(Some(rows[0].0.as_str()), "symbol")
        },
    )
    .await;

    m.check_plan(
        "stock_last_trade",
        quote::trades::stock_last_trade("AAPL"),
        |resp| {
            let last = resp.results.as_ref().ok_or("results is null")?;
            number(last.price, "price")?;
            timestamp(last.sip_timestamp, "sip_timestamp")
        },
    )
    .await;

    m.check_plan(
        "stock_last_quote",
        quote::trades::stock_last_quote("AAPL"),
        |resp| {
            let last = resp.results.as_ref().ok_or("results is null")?;
            number(last.bid_price, "bid_price")?;
            number(last.ask_price, "ask_price")
        },
    )
    .await;

    m.check_plan(
        "fetch_unified_snapshot_response",
        quote::unified::fetch_unified_snapshot_response(&["AAPL", "X:BTCUSD"]),
        |rows| {
            non_empty(rows, "unified snapshots")?;
            text(rows[0].symbol.as_deref(), "symbol")?;
            number(rows[0].last_price, "last_price")
        },
    )
    .await;

    // ---- ETF (partner data, unrouted but live) -----------------------------
    m.check_plan(
        "etf_constituents",
        market::etf_constituents("SPY", &[("limit", "1")]),
        |page| non_empty(page.results.as_deref().unwrap_or_default(), "constituents"),
    )
    .await;

    // ---- STREAMING ---------------------------------------------------------
    m.total += 1;
    match super::websocket::PolygonStream::builder(api_key)
        .map(|builder| builder.cluster(super::websocket::ClusterDTO::Stocks))
    {
        Ok(builder) => match builder.subscribe(&["AM.AAPL"]).build().await {
            Ok(mut stream) => {
                match tokio::time::timeout(std::time::Duration::from_secs(10), stream.next()).await
                {
                    Ok(Some(super::websocket::PolygonMessage::Status(status)))
                        if matches!(
                            status.get("status").and_then(serde_json::Value::as_str),
                            Some("not_authorized" | "auth_failed")
                        ) =>
                    {
                        m.plan_limited += 1;
                        println!("plan-limited: websocket_stocks");
                    }
                    Ok(Some(_)) => {
                        m.passed += 1;
                        println!("ok: websocket_stocks");
                    }
                    Ok(None) => m.fail("websocket_stocks", "stream closed"),
                    Err(_) => m.fail("websocket_stocks", "subscription timed out"),
                }
            }
            Err(FinanceError::AuthenticationFailed { ref context }) if is_plan_gap(context) => {
                m.plan_limited += 1;
                println!("plan-limited: websocket_stocks");
            }
            Err(error) => m.fail("websocket_stocks", &error.to_string()),
        },
        Err(error) => m.fail("websocket_stocks", &error.to_string()),
    }

    m.finish();
}
