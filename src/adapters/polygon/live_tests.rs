use ::futures::StreamExt;
use chrono::{Days, TimeZone, Utc};

use super::{
    chart, corporate, crypto, discovery, economic, filings, forex, fundamentals, futures, indices,
    market, options, quote, technical,
};
use crate::{FinanceError, Frequency, Interval, StatementType, TimeRange};

#[tokio::test]
#[ignore = "requires POLYGON_API_KEY and consumes live Massive API quota"]
#[allow(clippy::too_many_lines)]
async fn all_supported_polygon_functions_work_live() {
    let api_key = std::env::var("POLYGON_API_KEY").expect("POLYGON_API_KEY must be set");
    super::init_with_timeout(api_key.clone(), std::time::Duration::from_secs(120))
        .expect("Polygon client must initialize");

    let today = Utc::now().date_naive();
    let from = today.checked_sub_days(Days::new(30)).unwrap().to_string();
    let to = today.checked_sub_days(Days::new(1)).unwrap().to_string();
    let from_ts = Utc
        .from_utc_datetime(
            &today
                .checked_sub_days(Days::new(30))
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .timestamp();
    let to_ts = Utc
        .from_utc_datetime(
            &today
                .checked_sub_days(Days::new(1))
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .timestamp();
    let mut failures = Vec::new();
    let mut passed = 0_usize;
    let mut plan_limited = 0_usize;
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

    macro_rules! check_plan {
        ($name:literal, $future:expr) => {{
            total += 1;
            match $future.await {
                Ok(_) => {
                    passed += 1;
                    println!("ok: {}", $name);
                }
                Err(FinanceError::AuthenticationFailed { .. }) => {
                    plan_limited += 1;
                    println!("plan-limited: {}", $name);
                }
                Err(error) => {
                    eprintln!("failed: {}: {}", $name, error);
                    failures.push(format!("{}: {}", $name, error));
                }
            }
        }};
    }

    check!(
        "stock_aggregates",
        chart::stock_aggregates("AAPL", 1, super::models::Timespan::Day, &from, &to, None,)
    );
    check!(
        "fetch_chart_response",
        chart::fetch_chart_response("AAPL", Interval::OneDay, TimeRange::OneMonth)
    );
    check!(
        "fetch_chart_range_response",
        chart::fetch_chart_range_response("AAPL", Interval::OneDay, from_ts, to_ts)
    );
    check!(
        "stock_previous_close",
        chart::stock_previous_close("AAPL", Some(true))
    );
    check!(
        "stock_grouped_daily",
        chart::stock_grouped_daily(&to, Some(true))
    );
    check!(
        "stock_daily_open_close",
        chart::stock_daily_open_close("AAPL", &to, Some(true))
    );

    check!(
        "stock_dividends",
        corporate::stock_dividends(&[("ticker", "AAPL"), ("limit", "2")])
    );
    check!(
        "stock_splits",
        corporate::stock_splits(&[("ticker", "AAPL"), ("limit", "2")])
    );
    check!(
        "fetch_events_response",
        corporate::fetch_events_response("AAPL")
    );
    check!(
        "stock_news",
        corporate::stock_news(&[("ticker", "AAPL"), ("limit", "2")])
    );
    check!(
        "fetch_news_response",
        corporate::fetch_news_response("AAPL")
    );

    use corporate::benzinga;
    check_plan!(
        "analyst_ratings",
        benzinga::analyst_ratings(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check_plan!(
        "analyst_insights",
        benzinga::analyst_insights(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check_plan!(
        "analyst_details",
        benzinga::analyst_details(&[("limit", "1")])
    );
    check_plan!(
        "bulls_bears",
        benzinga::bulls_bears(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check_plan!(
        "consensus_ratings",
        benzinga::consensus_ratings("AAPL", &[("limit", "1")])
    );
    check_plan!(
        "corporate_guidance",
        benzinga::corporate_guidance(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check_plan!(
        "benzinga_earnings",
        benzinga::benzinga_earnings(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check_plan!("firm_details", benzinga::firm_details(&[("limit", "1")]));

    check!(
        "crypto_aggregates",
        crypto::aggregates::crypto_aggregates(
            "X:BTCUSD",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        )
    );
    check!(
        "crypto_previous_close",
        crypto::aggregates::crypto_previous_close("X:BTCUSD", Some(true))
    );
    check!(
        "crypto_grouped_daily",
        crypto::aggregates::crypto_grouped_daily(&to, Some(true))
    );
    check!(
        "crypto_daily_open_close",
        crypto::aggregates::crypto_daily_open_close("BTC", "USD", &to)
    );
    check_plan!(
        "crypto_last_trade",
        crypto::trades::crypto_last_trade("BTC", "USD")
    );
    check_plan!(
        "crypto_trades",
        crypto::trades::crypto_trades("X:BTCUSD", &[("limit", "2")])
    );
    check_plan!(
        "crypto_snapshots_all",
        crypto::snapshots::crypto_snapshots_all(Some("X:BTCUSD"))
    );
    check_plan!(
        "crypto_snapshot",
        crypto::snapshots::crypto_snapshot("X:BTCUSD")
    );
    check_plan!(
        "fetch_crypto_quote_response",
        crypto::snapshots::fetch_crypto_quote_response("BTC", "USD")
    );
    check_plan!(
        "crypto_top_movers",
        crypto::snapshots::crypto_top_movers("gainers")
    );

    check!(
        "all_tickers",
        discovery::all_tickers(&[("ticker", "AAPL"), ("limit", "1")])
    );
    check!("ticker_details", discovery::ticker_details("AAPL"));
    check!("ticker_types", discovery::ticker_types(&[("limit", "2")]));
    check!("related_tickers", discovery::related_tickers("AAPL"));
    check!(
        "fetch_similar_symbols_response",
        discovery::fetch_similar_symbols_response("AAPL", 5)
    );
    check!(
        "fetch_symbol_search_response",
        discovery::fetch_symbol_search_response("Apple", 5)
    );
    check!(
        "fetch_symbol_details_response",
        discovery::fetch_symbol_details_response("AAPL")
    );
    check!("exchanges", discovery::exchanges(&[("limit", "2")]));
    check!(
        "fetch_exchanges_response",
        discovery::fetch_exchanges_response()
    );
    check!("market_holidays", discovery::market_holidays());
    check!(
        "fetch_market_holidays_response",
        discovery::fetch_market_holidays_response()
    );
    check!("market_status", discovery::market_status());
    check!(
        "condition_codes",
        discovery::condition_codes(&[("limit", "2")])
    );

    check!("inflation", economic::inflation(&[("limit", "2")]));
    check!(
        "inflation_expectations",
        economic::inflation_expectations(&[("limit", "2")])
    );
    check!("labor_market", economic::labor_market(&[("limit", "2")]));
    check!(
        "treasury_yields",
        economic::treasury_yields(&[("limit", "2")])
    );
    for series in [
        "inflation",
        "inflation_expectations",
        "labor_market",
        "treasury_yields",
    ] {
        check!(
            "fetch_economic_series_response",
            economic::fetch_economic_series_response(series)
        );
    }

    let ten_k_index =
        filings::sec_edgar_index(&[("ticker", "AAPL"), ("form_type", "10-K"), ("limit", "1")])
            .await;
    total += 1;
    let ten_k_accession = match ten_k_index {
        Ok(response) => {
            passed += 1;
            println!("ok: sec_edgar_index_10k");
            response
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.accession_number)
        }
        Err(error) => {
            failures.push(format!("sec_edgar_index_10k: {error}"));
            None
        }
    };
    check!(
        "fetch_filings_response",
        filings::fetch_filings_response("AAPL")
    );
    check!(
        "risk_factors",
        filings::risk_factors(&[("ticker", "AAPL"), ("limit", "2")])
    );
    check!("risk_categories", filings::risk_categories());
    check!(
        "fetch_risk_factors_response",
        filings::fetch_risk_factors_response("AAPL")
    );
    if let Some(accession) = ten_k_accession.as_deref() {
        check!(
            "filing_10k_sections",
            filings::filing_10k_sections(accession, &[("ticker", "AAPL"), ("limit", "2")])
        );
        check!(
            "fetch_filing_sections_response",
            filings::fetch_filing_sections_response(
                accession,
                crate::models::filings::FilingSectionForm::TenK,
            )
        );
    } else {
        failures.push("filing_10k_sections: no 10-K accession returned".to_string());
        failures.push("fetch_filing_sections_response: no 10-K accession returned".to_string());
    }
    let eight_k_index =
        filings::sec_edgar_index(&[("ticker", "AAPL"), ("form_type", "8-K"), ("limit", "1")]).await;
    total += 1;
    let eight_k_accession = match eight_k_index {
        Ok(response) => {
            passed += 1;
            println!("ok: sec_edgar_index_8k");
            response
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.accession_number)
        }
        Err(error) => {
            failures.push(format!("sec_edgar_index_8k: {error}"));
            None
        }
    };
    if let Some(accession) = eight_k_accession.as_deref() {
        check!(
            "filing_8k_text",
            filings::filing_8k_text(accession, &[("ticker", "AAPL"), ("limit", "2")])
        );
    } else {
        failures.push("filing_8k_text: no 8-K accession returned".to_string());
    }

    check!(
        "forex_aggregates",
        forex::aggregates::forex_aggregates(
            "C:EURUSD",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        )
    );
    check!(
        "forex_previous_close",
        forex::aggregates::forex_previous_close("C:EURUSD", Some(true))
    );
    check!(
        "forex_grouped_daily",
        forex::aggregates::forex_grouped_daily(&to, Some(true))
    );
    check_plan!(
        "forex_last_quote",
        forex::quotes::forex_last_quote("EUR", "USD")
    );
    check_plan!(
        "fetch_forex_quote_response",
        forex::quotes::fetch_forex_quote_response("EUR", "USD")
    );
    check_plan!(
        "forex_snapshots_all",
        forex::snapshots::forex_snapshots_all(Some("C:EURUSD"))
    );
    check_plan!(
        "forex_snapshot",
        forex::snapshots::forex_snapshot("C:EURUSD")
    );
    check_plan!(
        "forex_top_movers",
        forex::snapshots::forex_top_movers("gainers")
    );

    check!(
        "stock_financials",
        fundamentals::stock_financials("AAPL", &[("limit", "2")])
    );
    for statement in [
        StatementType::Income,
        StatementType::Balance,
        StatementType::CashFlow,
    ] {
        check!(
            "fetch_financials_response",
            fundamentals::fetch_financials_response("AAPL", statement, Frequency::Annual)
        );
    }
    check!(
        "stock_short_interest",
        fundamentals::stock_short_interest("AAPL", &[("limit", "2")])
    );
    check!(
        "stock_short_volume",
        fundamentals::stock_short_volume("AAPL", &[("limit", "2")])
    );
    check!("stock_float", fundamentals::stock_float("AAPL"));
    check!(
        "fetch_short_interest_response",
        fundamentals::fetch_short_interest_response("AAPL")
    );
    check!(
        "fetch_short_volume_response",
        fundamentals::fetch_short_volume_response("AAPL")
    );
    check!(
        "fetch_share_float_response",
        fundamentals::fetch_share_float_response("AAPL")
    );

    let contracts = futures::reference::futures_contracts(&[
        ("product_code", "ES"),
        ("active", "true"),
        ("limit", "1"),
    ])
    .await;
    total += 1;
    let futures_ticker = match contracts {
        Ok(response) => {
            passed += 1;
            println!("ok: futures_contracts");
            response
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.ticker)
        }
        Err(error) => {
            failures.push(format!("futures_contracts: {error}"));
            None
        }
    };
    check!(
        "futures_products",
        futures::reference::futures_products(&[("product_code", "ES"), ("limit", "2")])
    );
    check!(
        "futures_schedules",
        futures::reference::futures_schedules(&[("product_code", "ES"), ("limit", "2")])
    );
    check!(
        "futures_market_status",
        futures::reference::futures_market_status(&[("product_code", "ES"), ("limit", "2")])
    );
    check!(
        "futures_exchanges",
        futures::reference::futures_exchanges(&[("limit", "2")])
    );
    if let Some(ticker) = futures_ticker.as_deref() {
        check!(
            "futures_aggregates",
            futures::aggregates::futures_aggregates(ticker, "1session", &[("limit", "2")])
        );
        check_plan!(
            "futures_trades",
            futures::trades::futures_trades(ticker, &[("limit", "2")])
        );
        check_plan!(
            "futures_quotes",
            futures::trades::futures_quotes(ticker, &[("limit", "2")])
        );
        check_plan!(
            "futures_snapshot",
            futures::snapshots::futures_snapshot(ticker)
        );
        check_plan!(
            "fetch_futures_quote_response",
            futures::snapshots::fetch_futures_quote_response(ticker)
        );
    } else {
        failures.push("futures market data: no active ES contract returned".to_string());
    }

    check_plan!(
        "index_aggregates",
        indices::aggregates::index_aggregates(
            "I:SPX",
            1,
            super::models::Timespan::Day,
            &from,
            &to,
            None,
        )
    );
    check_plan!(
        "index_previous_close",
        indices::aggregates::index_previous_close("I:SPX")
    );
    check_plan!(
        "index_daily_open_close",
        indices::aggregates::index_daily_open_close("I:SPX", &to)
    );
    check_plan!(
        "index_snapshot",
        indices::snapshots::index_snapshot("I:SPX")
    );
    check_plan!(
        "fetch_indices_quote_response",
        indices::snapshots::fetch_indices_quote_response("I:SPX")
    );

    check_plan!(
        "etf_analytics",
        market::etf_analytics(&[("composite_ticker", "SPY"), ("limit", "1")])
    );
    check_plan!(
        "etf_constituents",
        market::etf_constituents("SPY", &[("limit", "1")])
    );
    check_plan!(
        "etf_fund_flows",
        market::etf_fund_flows(&[("composite_ticker", "SPY"), ("limit", "1")])
    );
    check_plan!(
        "etf_profiles",
        market::etf_profiles(&[("composite_ticker", "SPY"), ("limit", "1")])
    );
    check_plan!(
        "etf_taxonomies",
        market::etf_taxonomies(&[("composite_ticker", "SPY"), ("limit", "1")])
    );

    let option_contracts = options::reference::options_contracts(&[
        ("underlying_ticker", "AAPL"),
        ("expired", "false"),
        ("limit", "1"),
    ])
    .await;
    total += 1;
    let option_ticker = match option_contracts {
        Ok(response) => {
            passed += 1;
            println!("ok: options_contracts");
            response
                .results
                .unwrap_or_default()
                .into_iter()
                .find_map(|item| item.ticker)
        }
        Err(error) => {
            failures.push(format!("options_contracts: {error}"));
            None
        }
    };
    check_plan!(
        "options_chain_snapshot",
        options::snapshots::options_chain_snapshot("AAPL", &[("limit", "1")])
    );
    check_plan!(
        "fetch_options_response",
        options::snapshots::fetch_options_response("AAPL", None)
    );
    if let Some(ticker) = option_ticker.as_deref() {
        check_plan!(
            "options_aggregates",
            options::aggregates::options_aggregates(
                ticker,
                1,
                super::models::Timespan::Day,
                &from,
                &to,
                None,
            )
        );
        check_plan!(
            "options_previous_close",
            options::aggregates::options_previous_close(ticker, Some(true))
        );
        check_plan!(
            "options_daily_open_close",
            options::aggregates::options_daily_open_close(ticker, &to, Some(true))
        );
        check_plan!(
            "options_last_trade",
            options::trades::options_last_trade(ticker)
        );
        check_plan!(
            "options_trades",
            options::trades::options_trades(ticker, &[("limit", "2")])
        );
        check_plan!(
            "options_quotes",
            options::trades::options_quotes(ticker, &[("limit", "2")])
        );
        check_plan!(
            "options_contract_snapshot",
            options::snapshots::options_contract_snapshot("AAPL", ticker)
        );
    } else {
        failures.push("options market data: no active AAPL contract returned".to_string());
    }

    check_plan!("stock_snapshot", quote::stock_snapshot("AAPL"));
    check_plan!("fetch_quote_response", quote::fetch_quote_response("AAPL"));
    check_plan!(
        "fetch_quotes_batch_response",
        quote::fetch_quotes_batch_response(&["AAPL", "MSFT"])
    );
    check_plan!(
        "stock_snapshots_all",
        quote::stock_snapshots_all(Some("AAPL,MSFT"))
    );
    check_plan!("stock_last_trade", quote::trades::stock_last_trade("AAPL"));
    check_plan!(
        "stock_trades",
        quote::trades::stock_trades("AAPL", &[("limit", "2")])
    );
    check_plan!("stock_last_quote", quote::trades::stock_last_quote("AAPL"));
    check_plan!(
        "stock_quotes",
        quote::trades::stock_quotes("AAPL", &[("limit", "2")])
    );
    check_plan!(
        "unified_snapshot",
        quote::unified::unified_snapshot(&["AAPL", "X:BTCUSD"])
    );
    check_plan!(
        "fetch_unified_snapshot_response",
        quote::unified::fetch_unified_snapshot_response(&["AAPL", "X:BTCUSD"])
    );

    for indicator in [
        technical::TechnicalIndicator::Sma,
        technical::TechnicalIndicator::Ema,
        technical::TechnicalIndicator::Macd,
        technical::TechnicalIndicator::Rsi,
    ] {
        check!(
            "technical_indicator",
            technical::technical_indicator(indicator, "AAPL", &[("limit", "2")])
        );
    }
    check!("stock_sma", technical::stock_sma("AAPL", &[("limit", "2")]));
    check!("stock_ema", technical::stock_ema("AAPL", &[("limit", "2")]));
    check!(
        "stock_macd",
        technical::stock_macd("AAPL", &[("limit", "2")])
    );
    check!("stock_rsi", technical::stock_rsi("AAPL", &[("limit", "2")]));

    total += 1;
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
                        plan_limited += 1;
                        println!("plan-limited: websocket_stocks");
                    }
                    Ok(Some(_)) => {
                        passed += 1;
                        println!("ok: websocket_stocks");
                    }
                    Ok(None) => failures.push("websocket_stocks: stream closed".to_string()),
                    Err(_) => failures.push("websocket_stocks: subscription timed out".to_string()),
                }
            }
            Err(FinanceError::AuthenticationFailed { .. }) => {
                plan_limited += 1;
                println!("plan-limited: websocket_stocks");
            }
            Err(error) => failures.push(format!("websocket_stocks: {error}")),
        },
        Err(error) => failures.push(format!("websocket_stocks: {error}")),
    }

    println!(
        "Polygon live matrix: {passed} data checks passed, {plan_limited} routes verified but plan-limited, {total} total"
    );
    assert!(
        failures.is_empty(),
        "{} Polygon live checks failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
