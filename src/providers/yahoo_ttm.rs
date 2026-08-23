//! Locally computed trailing-twelve-month metrics and ratios for
//! [`YahooProvider`](super::yahoo::YahooProvider), reusing data already
//! fetched via `fetch_financials`/`fetch_quote` rather than a new source.
//!
//! FMP's `KeyMetricsTtm`/`FinancialRatiosTtm` carry far more fields than can
//! honestly be derived from a standard three-statement filing — fields
//! needing FMP's own proprietary definitions (`graham_net_net`,
//! `price_to_fair_value`), a tax/EBIT allocation this can't cleanly separate
//! (`tax_burden`, `interest_burden`), or a forward-estimate dependency this
//! shouldn't silently pull in (`forward_peg_ratio`) stay `None` rather than
//! guess. Named subset only.

use std::collections::HashMap;

use crate::adapters::yahoo::client::YahooClient;
use crate::constants::fundamental_types::*;
use crate::constants::{Frequency, StatementType};
use crate::error::Result;
use crate::models::fundamentals::{FinancialRatiosTtm, KeyMetricsTtm};
use crate::models::quote::price::Price;

#[derive(Default)]
struct Facts {
    price: Option<f64>,
    market_cap: Option<f64>,
    shares_outstanding: Option<f64>,
    dividend_yield: Option<f64>,
    revenue: Option<f64>,
    cost_of_revenue: Option<f64>,
    gross_profit: Option<f64>,
    operating_income: Option<f64>,
    ebit: Option<f64>,
    ebitda: Option<f64>,
    net_income: Option<f64>,
    interest_expense: Option<f64>,
    diluted_eps: Option<f64>,
    capex: Option<f64>,
    total_assets: Option<f64>,
    prior_total_assets: Option<f64>,
    current_assets: Option<f64>,
    current_liabilities: Option<f64>,
    total_debt: Option<f64>,
    cash: Option<f64>,
    stockholders_equity: Option<f64>,
    inventory: Option<f64>,
    prior_inventory: Option<f64>,
    receivables: Option<f64>,
    prior_receivables: Option<f64>,
    payables: Option<f64>,
    prior_payables: Option<f64>,
}

async fn gather_facts(client: &YahooClient, symbol: &str) -> Result<Facts> {
    let (quote, income, balance, cashflow) = tokio::try_join!(
        crate::adapters::yahoo::quote::summary::fetch_summary(client, symbol),
        crate::adapters::yahoo::fundamentals::fetch(
            client,
            symbol,
            StatementType::Income,
            Frequency::Annual
        ),
        crate::adapters::yahoo::fundamentals::fetch(
            client,
            symbol,
            StatementType::Balance,
            Frequency::Annual
        ),
        crate::adapters::yahoo::fundamentals::fetch(
            client,
            symbol,
            StatementType::CashFlow,
            Frequency::Annual
        ),
    )?;

    let shares_outstanding = quote
        .default_key_statistics
        .as_ref()
        .and_then(|s| s.shares_outstanding.as_ref())
        .and_then(|v| v.raw)
        .map(|r| r as f64);
    let price_data: Option<&Price> = quote.price.as_ref();

    let balance_dates = sorted_dates(&balance.statement, TOTAL_ASSETS);
    let current_balance_date = balance_dates.first().map(String::as_str);
    let prior_balance_date = balance_dates.get(1).map(String::as_str);

    Ok(Facts {
        price: price_data.and_then(Price::current_price),
        market_cap: price_data
            .and_then(|p| p.market_cap.as_ref())
            .and_then(|v| v.raw)
            .map(|r| r as f64),
        shares_outstanding,
        dividend_yield: quote
            .summary_detail
            .as_ref()
            .and_then(|s| s.dividend_yield.as_ref())
            .and_then(|v| v.raw),
        revenue: latest(&income.statement, TOTAL_REVENUE),
        cost_of_revenue: latest(&income.statement, COST_OF_REVENUE),
        gross_profit: latest(&income.statement, GROSS_PROFIT),
        operating_income: latest(&income.statement, OPERATING_INCOME),
        ebit: latest(&income.statement, EBIT),
        ebitda: latest(&income.statement, EBITDA),
        net_income: latest(&income.statement, NET_INCOME),
        interest_expense: latest(&income.statement, INTEREST_EXPENSE),
        diluted_eps: latest(&income.statement, DILUTED_EPS),
        capex: latest(&cashflow.statement, CAPITAL_EXPENDITURE).map(f64::abs),
        total_assets: current_balance_date
            .and_then(|d| value_at(&balance.statement, TOTAL_ASSETS, d)),
        prior_total_assets: prior_balance_date
            .and_then(|d| value_at(&balance.statement, TOTAL_ASSETS, d)),
        current_assets: current_balance_date
            .and_then(|d| value_at(&balance.statement, CURRENT_ASSETS, d)),
        current_liabilities: current_balance_date
            .and_then(|d| value_at(&balance.statement, CURRENT_LIABILITIES, d)),
        total_debt: current_balance_date.and_then(|d| value_at(&balance.statement, TOTAL_DEBT, d)),
        cash: current_balance_date
            .and_then(|d| value_at(&balance.statement, CASH_AND_CASH_EQUIVALENTS, d)),
        stockholders_equity: current_balance_date
            .and_then(|d| value_at(&balance.statement, STOCKHOLDERS_EQUITY, d)),
        inventory: current_balance_date.and_then(|d| value_at(&balance.statement, INVENTORY, d)),
        prior_inventory: prior_balance_date
            .and_then(|d| value_at(&balance.statement, INVENTORY, d)),
        receivables: current_balance_date
            .and_then(|d| value_at(&balance.statement, ACCOUNTS_RECEIVABLE, d)),
        prior_receivables: prior_balance_date
            .and_then(|d| value_at(&balance.statement, ACCOUNTS_RECEIVABLE, d)),
        payables: current_balance_date
            .and_then(|d| value_at(&balance.statement, ACCOUNTS_PAYABLE, d)),
        prior_payables: prior_balance_date
            .and_then(|d| value_at(&balance.statement, ACCOUNTS_PAYABLE, d)),
    })
}

pub(super) async fn fetch_key_metrics_ttm(
    client: &YahooClient,
    symbol: &str,
) -> Result<KeyMetricsTtm> {
    let f = gather_facts(client, symbol).await?;
    Ok(key_metrics_from_facts(symbol, &f))
}

fn key_metrics_from_facts(symbol: &str, f: &Facts) -> KeyMetricsTtm {
    let enterprise_value = add(f.market_cap, f.total_debt).and_then(|v| sub(Some(v), f.cash));
    let book_value_per_share = div(f.stockholders_equity, f.shares_outstanding);
    let average_receivables = average(f.receivables, f.prior_receivables);
    let average_payables = average(f.payables, f.prior_payables);
    let average_inventory = average(f.inventory, f.prior_inventory);
    let days_of_sales_outstanding = days(average_receivables, f.revenue);
    let days_of_inventory_outstanding = days(average_inventory, f.cost_of_revenue);
    let days_of_payables_outstanding = days(average_payables, f.cost_of_revenue);
    let operating_cycle = add(days_of_sales_outstanding, days_of_inventory_outstanding);
    let cash_conversion_cycle =
        operating_cycle.and_then(|c| sub(Some(c), days_of_payables_outstanding));

    KeyMetricsTtm {
        symbol: Some(symbol.to_string()),
        market_cap: f.market_cap,
        enterprise_value,
        ev_to_sales: div(enterprise_value, f.revenue),
        ev_to_ebitda: div(enterprise_value, f.ebitda),
        current_ratio: div(f.current_assets, f.current_liabilities),
        working_capital: sub(f.current_assets, f.current_liabilities),
        graham_number: f
            .diluted_eps
            .zip(book_value_per_share)
            .map(|(eps, bvps)| (22.5 * eps * bvps).max(0.0).sqrt()),
        return_on_assets: div(f.net_income, f.total_assets),
        return_on_equity: div(f.net_income, f.stockholders_equity),
        earnings_yield: div(f.diluted_eps, f.price),
        capex_to_revenue: div(f.capex, f.revenue),
        average_receivables,
        average_payables,
        average_inventory,
        days_of_sales_outstanding,
        days_of_inventory_outstanding,
        days_of_payables_outstanding,
        operating_cycle,
        cash_conversion_cycle,
        ..Default::default()
    }
}

pub(super) async fn fetch_ratios_ttm(
    client: &YahooClient,
    symbol: &str,
) -> Result<FinancialRatiosTtm> {
    let f = gather_facts(client, symbol).await?;
    Ok(ratios_from_facts(symbol, &f))
}

fn ratios_from_facts(symbol: &str, f: &Facts) -> FinancialRatiosTtm {
    let book_value_per_share = div(f.stockholders_equity, f.shares_outstanding);
    let average_receivables = average(f.receivables, f.prior_receivables);
    let average_payables = average(f.payables, f.prior_payables);
    let average_inventory = average(f.inventory, f.prior_inventory);
    let average_total_assets = average(f.total_assets, f.prior_total_assets);

    FinancialRatiosTtm {
        symbol: Some(symbol.to_string()),
        gross_profit_margin: div(f.gross_profit, f.revenue),
        operating_profit_margin: div(f.operating_income, f.revenue),
        net_profit_margin: div(f.net_income, f.revenue),
        receivables_turnover: div(f.revenue, average_receivables),
        payables_turnover: div(f.cost_of_revenue, average_payables),
        inventory_turnover: div(f.cost_of_revenue, average_inventory),
        asset_turnover: div(f.revenue, average_total_assets),
        current_ratio: div(f.current_assets, f.current_liabilities),
        quick_ratio: sub(f.current_assets, f.inventory)
            .and_then(|qa| div(Some(qa), f.current_liabilities)),
        price_earnings_ratio: div(f.price, f.diluted_eps),
        price_to_book_ratio: div(f.price, book_value_per_share),
        price_to_sales_ratio: div(f.market_cap, f.revenue),
        debt_equity_ratio: div(f.total_debt, f.stockholders_equity),
        interest_coverage: f
            .interest_expense
            .filter(|v| *v != 0.0)
            .and_then(|ie| div(f.ebit, Some(ie.abs()))),
        dividend_yield: f.dividend_yield,
        book_value_per_share,
        revenue_per_share: div(f.revenue, f.shares_outstanding),
        net_income_per_share: div(f.net_income, f.shares_outstanding),
        ..Default::default()
    }
}

fn sorted_dates(stmt: &HashMap<String, HashMap<String, f64>>, metric: &str) -> Vec<String> {
    let mut dates: Vec<String> = stmt
        .get(metric)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    dates.sort_by(|a, b| b.cmp(a));
    dates
}

fn value_at(stmt: &HashMap<String, HashMap<String, f64>>, metric: &str, date: &str) -> Option<f64> {
    stmt.get(metric).and_then(|m| m.get(date)).copied()
}

fn latest(stmt: &HashMap<String, HashMap<String, f64>>, metric: &str) -> Option<f64> {
    let dates = sorted_dates(stmt, metric);
    let date = dates.first()?;
    value_at(stmt, metric, date)
}

fn add(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    Some(a? + b?)
}

fn sub(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    Some(a? - b?)
}

fn div(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    let a = a?;
    let b = b?;
    (b != 0.0).then_some(a / b)
}

fn average(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) => Some((a + b) / 2.0),
        (Some(a), None) => Some(a),
        _ => None,
    }
}

/// `average / per_period * 365`, the standard days-outstanding formula.
fn days(average: Option<f64>, per_period: Option<f64>) -> Option<f64> {
    div(average, per_period).map(|r| r * 365.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_facts() -> Facts {
        Facts {
            price: Some(200.0),
            market_cap: Some(200e9),
            shares_outstanding: Some(1e9),
            dividend_yield: Some(0.005),
            revenue: Some(100e9),
            cost_of_revenue: Some(60e9),
            gross_profit: Some(40e9),
            operating_income: Some(30e9),
            ebit: Some(30e9),
            ebitda: Some(35e9),
            net_income: Some(25e9),
            interest_expense: Some(1e9),
            diluted_eps: Some(25.0),
            capex: Some(5e9),
            total_assets: Some(300e9),
            prior_total_assets: Some(280e9),
            current_assets: Some(120e9),
            current_liabilities: Some(60e9),
            total_debt: Some(80e9),
            cash: Some(50e9),
            stockholders_equity: Some(150e9),
            inventory: Some(10e9),
            prior_inventory: Some(8e9),
            receivables: Some(20e9),
            prior_receivables: Some(18e9),
            payables: Some(15e9),
            prior_payables: Some(13e9),
        }
    }

    fn approx(a: Option<f64>, b: f64) {
        assert!(
            a.is_some_and(|v| (v - b).abs() < 1e-6),
            "expected {b}, got {a:?}"
        );
    }

    #[test]
    fn key_metrics_match_hand_computed_values() {
        let metrics = key_metrics_from_facts("AAPL", &fixture_facts());
        approx(metrics.enterprise_value, 230e9);
        approx(metrics.ev_to_sales, 2.3);
        approx(metrics.current_ratio, 2.0);
        approx(metrics.working_capital, 60e9);
        approx(metrics.return_on_assets, 25.0 / 300.0);
        approx(metrics.return_on_equity, 25.0 / 150.0);
        approx(metrics.earnings_yield, 0.125);
        approx(metrics.capex_to_revenue, 0.05);
        // graham_number = sqrt(22.5 * eps * book_value_per_share) = sqrt(22.5 * 25 * 150)
        approx(metrics.graham_number, 84375f64.sqrt());
        approx(metrics.average_receivables, 19e9);
        approx(metrics.days_of_sales_outstanding, 19.0 / 100.0 * 365.0);
        approx(metrics.cash_conversion_cycle, 38.933_333_333_333_33);
    }

    #[test]
    fn ratios_match_hand_computed_values() {
        let ratios = ratios_from_facts("AAPL", &fixture_facts());
        approx(ratios.gross_profit_margin, 0.4);
        approx(ratios.net_profit_margin, 0.25);
        approx(ratios.current_ratio, 2.0);
        approx(ratios.quick_ratio, 110.0 / 60.0);
        approx(ratios.price_earnings_ratio, 8.0);
        approx(ratios.price_to_book_ratio, 200.0 / 150.0);
        approx(ratios.price_to_sales_ratio, 2.0);
        approx(ratios.debt_equity_ratio, 80.0 / 150.0);
        approx(ratios.interest_coverage, 30.0);
        approx(ratios.dividend_yield, 0.005);
        approx(ratios.book_value_per_share, 150.0);
        approx(ratios.revenue_per_share, 100.0);
        approx(ratios.net_income_per_share, 25.0);
        approx(ratios.receivables_turnover, 100.0 / 19.0);
        approx(ratios.inventory_turnover, 60.0 / 9.0);
    }

    #[test]
    fn missing_inputs_stay_none_rather_than_panic() {
        let empty = Facts::default();
        let metrics = key_metrics_from_facts("AAPL", &empty);
        assert_eq!(metrics.enterprise_value, None);
        assert_eq!(metrics.current_ratio, None);
        assert_eq!(metrics.graham_number, None);
        let ratios = ratios_from_facts("AAPL", &empty);
        assert_eq!(ratios.gross_profit_margin, None);
        assert_eq!(ratios.price_earnings_ratio, None);
    }

    #[test]
    fn division_by_zero_is_none_not_infinity() {
        assert_eq!(div(Some(1.0), Some(0.0)), None);
        assert_eq!(div(None, Some(1.0)), None);
        assert_eq!(div(Some(1.0), None), None);
    }

    #[test]
    fn average_falls_back_to_the_single_available_period() {
        assert_eq!(average(Some(10.0), Some(20.0)), Some(15.0));
        assert_eq!(average(Some(10.0), None), Some(10.0));
        assert_eq!(average(None, None), None);
    }
}
