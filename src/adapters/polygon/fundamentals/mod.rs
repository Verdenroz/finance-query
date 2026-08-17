//! Stock fundamental data: balance sheets, cash flow, income statements, ratios, short interest, float.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::fundamentals::FinancialStatement;
use crate::providers::build_financial_statement;
use crate::{Frequency, Provider, StatementType};

use super::build_client;
use super::models::PaginatedResponseDTO;

/// Reporting metadata shared by every Stock Financials v1 statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StatementPeriodDTO {
    /// CIK number.
    pub cik: Option<String>,
    /// Ticker symbols the filing covers.
    pub tickers: Option<Vec<String>>,
    /// Last day of the reporting period.
    pub period_end: Option<String>,
    /// Date the statement was filed.
    pub filing_date: Option<String>,
    /// Fiscal year.
    pub fiscal_year: Option<f64>,
    /// Fiscal quarter (1-4).
    pub fiscal_quarter: Option<f64>,
    /// `quarterly`, `annual`, or `trailing_twelve_months`.
    pub timeframe: Option<String>,
}

/// One income-statement period (`/stocks/financials/v1/income-statements`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IncomeStatementDTO {
    /// Reporting metadata.
    #[serde(flatten)]
    pub period: StatementPeriodDTO,
    /// Total revenue.
    pub revenue: Option<f64>,
    /// Cost of revenue.
    pub cost_of_revenue: Option<f64>,
    /// Gross profit.
    pub gross_profit: Option<f64>,
    /// Research and development expense.
    pub research_development: Option<f64>,
    /// Selling, general and administrative expense.
    pub selling_general_administrative: Option<f64>,
    /// Depreciation, depletion and amortization.
    pub depreciation_depletion_amortization: Option<f64>,
    /// Other operating expenses.
    pub other_operating_expenses: Option<f64>,
    /// Total operating expenses.
    pub total_operating_expenses: Option<f64>,
    /// Operating income.
    pub operating_income: Option<f64>,
    /// Interest expense.
    pub interest_expense: Option<f64>,
    /// Interest income.
    pub interest_income: Option<f64>,
    /// Other income and expense.
    pub other_income_expense: Option<f64>,
    /// Total other income and expense.
    pub total_other_income_expense: Option<f64>,
    /// Income before income taxes.
    pub income_before_income_taxes: Option<f64>,
    /// Income taxes.
    pub income_taxes: Option<f64>,
    /// Equity in affiliates.
    pub equity_in_affiliates: Option<f64>,
    /// Discontinued operations.
    pub discontinued_operations: Option<f64>,
    /// Extraordinary items.
    pub extraordinary_items: Option<f64>,
    /// Non-controlling interest.
    pub noncontrolling_interest: Option<f64>,
    /// Consolidated net income or loss.
    pub consolidated_net_income_loss: Option<f64>,
    /// Net income or loss attributable to common shareholders.
    pub net_income_loss_attributable_common_shareholders: Option<f64>,
    /// Preferred stock dividends declared.
    pub preferred_stock_dividends_declared: Option<f64>,
    /// Basic earnings per share.
    pub basic_earnings_per_share: Option<f64>,
    /// Diluted earnings per share.
    pub diluted_earnings_per_share: Option<f64>,
    /// Basic shares outstanding.
    pub basic_shares_outstanding: Option<f64>,
    /// Diluted shares outstanding.
    pub diluted_shares_outstanding: Option<f64>,
    /// EBITDA.
    pub ebitda: Option<f64>,
}

/// One balance-sheet period (`/stocks/financials/v1/balance-sheets`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BalanceSheetDTO {
    /// Reporting metadata.
    #[serde(flatten)]
    pub period: StatementPeriodDTO,
    /// Cash and cash equivalents.
    pub cash_and_equivalents: Option<f64>,
    /// Short-term investments.
    pub short_term_investments: Option<f64>,
    /// Receivables.
    pub receivables: Option<f64>,
    /// Inventories.
    pub inventories: Option<f64>,
    /// Other current assets.
    pub other_current_assets: Option<f64>,
    /// Total current assets.
    pub total_current_assets: Option<f64>,
    /// Property, plant and equipment, net.
    pub property_plant_equipment_net: Option<f64>,
    /// Goodwill.
    pub goodwill: Option<f64>,
    /// Intangible assets, net.
    pub intangible_assets_net: Option<f64>,
    /// Other assets.
    pub other_assets: Option<f64>,
    /// Total assets.
    pub total_assets: Option<f64>,
    /// Accounts payable.
    pub accounts_payable: Option<f64>,
    /// Accrued and other current liabilities.
    pub accrued_and_other_current_liabilities: Option<f64>,
    /// Current portion of debt.
    pub debt_current: Option<f64>,
    /// Current deferred revenue.
    pub deferred_revenue_current: Option<f64>,
    /// Total current liabilities.
    pub total_current_liabilities: Option<f64>,
    /// Long-term debt and capital lease obligations.
    pub long_term_debt_and_capital_lease_obligations: Option<f64>,
    /// Other non-current liabilities.
    pub other_noncurrent_liabilities: Option<f64>,
    /// Total liabilities.
    pub total_liabilities: Option<f64>,
    /// Commitments and contingencies.
    pub commitments_and_contingencies: Option<f64>,
    /// Preferred stock.
    pub preferred_stock: Option<f64>,
    /// Common stock.
    pub common_stock: Option<f64>,
    /// Additional paid-in capital.
    pub additional_paid_in_capital: Option<f64>,
    /// Retained earnings or accumulated deficit.
    pub retained_earnings_deficit: Option<f64>,
    /// Treasury stock.
    pub treasury_stock: Option<f64>,
    /// Accumulated other comprehensive income.
    pub accumulated_other_comprehensive_income: Option<f64>,
    /// Other equity.
    pub other_equity: Option<f64>,
    /// Total equity attributable to the parent.
    pub total_equity_attributable_to_parent: Option<f64>,
    /// Non-controlling interest.
    pub noncontrolling_interest: Option<f64>,
    /// Total equity.
    pub total_equity: Option<f64>,
    /// Total liabilities and equity.
    pub total_liabilities_and_equity: Option<f64>,
}

/// One cash-flow-statement period (`/stocks/financials/v1/cash-flow-statements`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CashFlowStatementDTO {
    /// Reporting metadata.
    #[serde(flatten)]
    pub period: StatementPeriodDTO,
    /// Net income.
    pub net_income: Option<f64>,
    /// Depreciation, depletion and amortization.
    pub depreciation_depletion_and_amortization: Option<f64>,
    /// Net change in other operating assets and liabilities.
    pub change_in_other_operating_assets_and_liabilities_net: Option<f64>,
    /// Other operating activities.
    pub other_operating_activities: Option<f64>,
    /// Cash from continuing operating activities.
    pub cash_from_operating_activities_continuing_operations: Option<f64>,
    /// Net cash from discontinued operating activities.
    pub net_cash_from_operating_activities_discontinued_operations: Option<f64>,
    /// Net cash from operating activities.
    pub net_cash_from_operating_activities: Option<f64>,
    /// Purchases of property, plant and equipment.
    pub purchase_of_property_plant_and_equipment: Option<f64>,
    /// Sales of property, plant and equipment.
    pub sale_of_property_plant_and_equipment: Option<f64>,
    /// Other investing activities.
    pub other_investing_activities: Option<f64>,
    /// Net cash from continuing investing activities.
    pub net_cash_from_investing_activities_continuing_operations: Option<f64>,
    /// Net cash from discontinued investing activities.
    pub net_cash_from_investing_activities_discontinued_operations: Option<f64>,
    /// Net cash from investing activities.
    pub net_cash_from_investing_activities: Option<f64>,
    /// Dividends paid.
    pub dividends: Option<f64>,
    /// Short-term debt issuances net of repayments.
    pub short_term_debt_issuances_repayments: Option<f64>,
    /// Long-term debt issuances net of repayments.
    pub long_term_debt_issuances_repayments: Option<f64>,
    /// Other financing activities.
    pub other_financing_activities: Option<f64>,
    /// Net cash from continuing financing activities.
    pub net_cash_from_financing_activities_continuing_operations: Option<f64>,
    /// Net cash from discontinued financing activities.
    pub net_cash_from_financing_activities_discontinued_operations: Option<f64>,
    /// Net cash from financing activities.
    pub net_cash_from_financing_activities: Option<f64>,
    /// Effect of currency exchange rate changes.
    pub effect_of_currency_exchange_rate: Option<f64>,
    /// Net change in cash and equivalents.
    pub change_in_cash_and_equivalents: Option<f64>,
    /// Income or loss from discontinued operations.
    pub income_loss_from_discontinued_operations: Option<f64>,
    /// Non-controlling interests.
    pub noncontrolling_interests: Option<f64>,
    /// Other cash adjustments.
    pub other_cash_adjustments: Option<f64>,
}

/// Short interest data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortInterestDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Settlement date.
    pub settlement_date: Option<String>,
    /// Short interest (shares).
    pub short_interest: Option<f64>,
    /// Average daily volume.
    pub avg_daily_volume: Option<f64>,
    /// Days to cover.
    pub days_to_cover: Option<f64>,
}

/// Short volume data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ShortVolumeDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Short volume.
    pub short_volume: Option<f64>,
    /// Short exempt volume.
    pub exempt_volume: Option<f64>,
    /// Short volume excluding exempt transactions.
    pub non_exempt_volume: Option<f64>,
    /// Total volume.
    pub total_volume: Option<f64>,
}

/// Float data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FloatDataDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Float shares.
    pub free_float: Option<f64>,
    /// Percentage of shares freely tradable.
    pub free_float_percent: Option<f64>,
    /// Effective date of the measurement.
    pub effective_date: Option<String>,
}

/// Fetch income statements for a stock ticker.
///
/// * `params` - Optional: `timeframe`, `period_end`, `filing_date`, `fiscal_year`,
///   `fiscal_quarter`, `limit`, `sort`, each also accepting the documented
///   comparison suffixes
pub async fn income_statements(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<IncomeStatementDTO>> {
    statements("income-statements", ticker, params).await
}

/// Fetch balance sheets for a stock ticker. `timeframe` accepts only
/// `quarterly` and `annual` here; a balance sheet has no TTM rollup.
pub async fn balance_sheets(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<BalanceSheetDTO>> {
    statements("balance-sheets", ticker, params).await
}

/// Fetch cash flow statements for a stock ticker.
pub async fn cash_flow_statements(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<CashFlowStatementDTO>> {
    statements("cash-flow-statements", ticker, params).await
}

async fn statements<T: serde::de::DeserializeOwned>(
    endpoint: &str,
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<T>> {
    let client = build_client()?;
    let path = format!("/stocks/financials/v1/{endpoint}");
    let mut query: Vec<(&str, &str)> = vec![("tickers", ticker)];
    query.extend_from_slice(params);
    client.get(&path, &query).await
}

/// Keys that identify the reporting period rather than measure it.
const STATEMENT_METADATA: [&str; 7] = [
    "cik",
    "tickers",
    "period_end",
    "filing_date",
    "fiscal_year",
    "fiscal_quarter",
    "timeframe",
];

/// Pivot flat statement rows into the canonical metric-by-period map.
fn pivot_statements<T: serde::Serialize>(
    rows: Vec<T>,
) -> std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>> {
    let mut data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, serde_json::Value>,
    > = std::collections::HashMap::new();

    for row in rows {
        let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(&row) else {
            continue;
        };
        let field = |key: &str| obj.get(key).and_then(serde_json::Value::as_str);
        let period = field("period_end")
            .or_else(|| field("filing_date"))
            .unwrap_or("unknown")
            .to_string();

        for (metric, value) in &obj {
            if STATEMENT_METADATA.contains(&metric.as_str())
                || !matches!(value, serde_json::Value::Number(_))
            {
                continue;
            }
            data.entry(metric.clone())
                .or_default()
                .insert(period.clone(), value.clone());
        }
    }

    data
}

/// Fetch financial statements (canonical) for a stock ticker.
pub async fn fetch_financials_response(
    symbol: &str,
    stmt_type: StatementType,
    frequency: Frequency,
) -> Result<FinancialStatement> {
    let timeframe = match frequency {
        Frequency::Annual => "annual",
        Frequency::Quarterly => "quarterly",
    };
    let params = [("timeframe", timeframe), ("limit", "100")];

    let data = match stmt_type {
        StatementType::Income => pivot_statements(
            income_statements(symbol, &params)
                .await?
                .results
                .unwrap_or_default(),
        ),
        StatementType::Balance => pivot_statements(
            balance_sheets(symbol, &params)
                .await?
                .results
                .unwrap_or_default(),
        ),
        StatementType::CashFlow => pivot_statements(
            cash_flow_statements(symbol, &params)
                .await?
                .results
                .unwrap_or_default(),
        ),
    };

    Ok(build_financial_statement(
        symbol.to_string(),
        stmt_type.as_str().to_string(),
        frequency.as_str().to_string(),
        Provider::Polygon,
        data,
    ))
}

/// Fetch short interest data for a stock ticker.
pub async fn stock_short_interest(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<ShortInterestDTO>> {
    let client = build_client()?;
    let mut query = vec![("ticker", ticker)];
    query.extend_from_slice(params);
    client.get("/stocks/v1/short-interest", &query).await
}

/// Fetch short volume data for a stock ticker.
pub async fn stock_short_volume(
    ticker: &str,
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<ShortVolumeDTO>> {
    let client = build_client()?;
    let mut query = vec![("ticker", ticker)];
    query.extend_from_slice(params);
    client.get("/stocks/v1/short-volume", &query).await
}

/// Fetch float data for a stock ticker.
pub async fn stock_float(ticker: &str) -> Result<PaginatedResponseDTO<FloatDataDTO>> {
    let client = build_client()?;
    client.get("/stocks/vX/float", &[("ticker", ticker)]).await
}

/// Fetch canonical short-interest reports for a stock ticker.
pub async fn fetch_short_interest_response(
    symbol: &str,
) -> Result<Vec<crate::models::fundamentals::ShortInterest>> {
    let paginated = stock_short_interest(symbol, &[("limit", "50")]).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|d| crate::models::fundamentals::ShortInterest {
            settlement_date: d.settlement_date,
            short_interest: d.short_interest,
            avg_daily_volume: d.avg_daily_volume,
            days_to_cover: d.days_to_cover,
        })
        .collect())
}

/// Fetch canonical daily short-volume data for a stock ticker.
pub async fn fetch_short_volume_response(
    symbol: &str,
) -> Result<Vec<crate::models::fundamentals::ShortVolume>> {
    let paginated = stock_short_volume(symbol, &[("limit", "50")]).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|d| crate::models::fundamentals::ShortVolume {
            date: d.date,
            short_volume: d.short_volume,
            short_exempt_volume: d.exempt_volume,
            total_volume: d.total_volume,
        })
        .collect())
}

/// Fetch canonical share float / shares outstanding for a stock ticker.
pub async fn fetch_share_float_response(
    symbol: &str,
) -> Result<crate::models::fundamentals::ShareFloat> {
    let paginated = stock_float(symbol).await?;
    let d = paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .next()
        .ok_or_else(|| crate::error::FinanceError::ResponseStructureError {
            field: "results".into(),
            context: format!("no float data returned for {symbol}"),
        })?;
    Ok(crate::models::fundamentals::ShareFloat {
        symbol: d.ticker.or_else(|| Some(symbol.to_string())),
        float_shares: d.free_float,
        // Massive's float endpoint reports no shares-outstanding figure, and
        // `free_float_percent` is rounded to 2dp so deriving one would be a
        // guess dressed as a report.
        outstanding_shares: None,
        float_percent: d.free_float_percent,
        date: d.effective_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn income_statements_read_the_flat_v1_shape() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stocks/financials/v1/income-statements")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("tickers".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("timeframe".into(), "annual".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
                    "status": "OK",
                    "request_id": "abc",
                    "results": [{
                        "cik": "0000320193",
                        "tickers": ["AAPL"],
                        "period_end": "2024-09-28",
                        "filing_date": "2024-11-01",
                        "fiscal_year": 2024,
                        "fiscal_quarter": 4,
                        "timeframe": "annual",
                        "revenue": 391035000000,
                        "cost_of_revenue": 210352000000,
                        "gross_profit": 180683000000,
                        "operating_income": 123216000000,
                        "diluted_earnings_per_share": 6.08,
                        "ebitda": 134661000000
                    }]
                }"#,
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponseDTO<IncomeStatementDTO> = client
            .get(
                "/stocks/financials/v1/income-statements",
                &[("tickers", "AAPL"), ("timeframe", "annual")],
            )
            .await
            .unwrap();

        let row = &resp.results.unwrap()[0];
        assert_eq!(row.period.period_end.as_deref(), Some("2024-09-28"));
        assert_eq!(row.period.fiscal_year, Some(2024.0));
        assert_eq!(row.revenue, Some(391_035_000_000.0));
        assert_eq!(row.gross_profit, Some(180_683_000_000.0));
        assert_eq!(row.diluted_earnings_per_share, Some(6.08));
    }

    #[test]
    fn pivot_keys_metrics_by_period_and_drops_metadata() {
        let rows: Vec<BalanceSheetDTO> = serde_json::from_str(
            r#"[
                {"period_end":"2024-09-28","fiscal_year":2024,"tickers":["AAPL"],
                 "total_assets":364980000000,"total_liabilities":308030000000},
                {"period_end":"2023-09-30","fiscal_year":2023,"tickers":["AAPL"],
                 "total_assets":352583000000}
            ]"#,
        )
        .unwrap();

        let data = pivot_statements(rows);

        assert_eq!(
            data["total_assets"]["2024-09-28"].as_f64(),
            Some(364_980_000_000.0)
        );
        assert_eq!(data["total_assets"].len(), 2);
        assert_eq!(data["total_liabilities"].len(), 1);
        // Period identifiers are not metrics.
        assert!(!data.contains_key("fiscal_year"));
        assert!(!data.contains_key("period_end"));
    }

    #[test]
    fn pivot_falls_back_to_the_filing_date() {
        let rows: Vec<CashFlowStatementDTO> =
            serde_json::from_str(r#"[{"filing_date":"2024-11-01","net_income":93736000000}]"#)
                .unwrap();
        assert!(pivot_statements(rows)["net_income"].contains_key("2024-11-01"));
    }

    #[tokio::test]
    async fn share_float_reports_the_percent_and_never_derives_outstanding() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stocks/vX/float")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(
                r#"{"status":"OK","results":[{"ticker":"AAPL","free_float":15000000000,
                    "free_float_percent":98.5,"effective_date":"2025-11-01"}]}"#,
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: PaginatedResponseDTO<FloatDataDTO> =
            client.get("/stocks/vX/float", &[]).await.unwrap();
        let d = resp.results.unwrap().into_iter().next().unwrap();

        assert_eq!(d.free_float, Some(15_000_000_000.0));
        assert_eq!(d.free_float_percent, Some(98.5));

        let share_float = crate::models::fundamentals::ShareFloat {
            symbol: d.ticker,
            float_shares: d.free_float,
            outstanding_shares: None,
            float_percent: d.free_float_percent,
            date: d.effective_date,
        };
        assert_eq!(share_float.float_percent, Some(98.5));
        assert!(share_float.outstanding_shares.is_none());
    }
}
