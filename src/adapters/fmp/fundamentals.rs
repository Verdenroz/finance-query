//! FMP financial statement endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::models::Period;

// ============================================================================
// Response types
// ============================================================================

/// Income statement from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IncomeStatement {
    /// Filing date.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Reporting period (annual/quarter).
    #[serde(rename = "reportedCurrency")]
    pub reported_currency: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Filing date.
    #[serde(rename = "fillingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period (e.g., "Q1", "FY").
    pub period: Option<String>,
    /// Total revenue.
    pub revenue: Option<f64>,
    /// Cost of revenue.
    #[serde(rename = "costOfRevenue")]
    pub cost_of_revenue: Option<f64>,
    /// Gross profit.
    #[serde(rename = "grossProfit")]
    pub gross_profit: Option<f64>,
    /// Gross profit ratio.
    #[serde(rename = "grossProfitRatio")]
    pub gross_profit_ratio: Option<f64>,
    /// Research and development expenses.
    #[serde(rename = "researchAndDevelopmentExpenses")]
    pub research_and_development_expenses: Option<f64>,
    /// General and administrative expenses.
    #[serde(rename = "generalAndAdministrativeExpenses")]
    pub general_and_administrative_expenses: Option<f64>,
    /// Selling and marketing expenses.
    #[serde(rename = "sellingAndMarketingExpenses")]
    pub selling_and_marketing_expenses: Option<f64>,
    /// Selling, general and administrative expenses.
    #[serde(rename = "sellingGeneralAndAdministrativeExpenses")]
    pub selling_general_and_administrative_expenses: Option<f64>,
    /// Other expenses.
    #[serde(rename = "otherExpenses")]
    pub other_expenses: Option<f64>,
    /// Operating expenses.
    #[serde(rename = "operatingExpenses")]
    pub operating_expenses: Option<f64>,
    /// Cost and expenses.
    #[serde(rename = "costAndExpenses")]
    pub cost_and_expenses: Option<f64>,
    /// Interest income.
    #[serde(rename = "interestIncome")]
    pub interest_income: Option<f64>,
    /// Interest expense.
    #[serde(rename = "interestExpense")]
    pub interest_expense: Option<f64>,
    /// Depreciation and amortization.
    #[serde(rename = "depreciationAndAmortization")]
    pub depreciation_and_amortization: Option<f64>,
    /// EBITDA.
    pub ebitda: Option<f64>,
    /// EBITDA ratio.
    #[serde(rename = "ebitdaratio")]
    pub ebitda_ratio: Option<f64>,
    /// Operating income.
    #[serde(rename = "operatingIncome")]
    pub operating_income: Option<f64>,
    /// Operating income ratio.
    #[serde(rename = "operatingIncomeRatio")]
    pub operating_income_ratio: Option<f64>,
    /// Total other income/expenses net.
    #[serde(rename = "totalOtherIncomeExpensesNet")]
    pub total_other_income_expenses_net: Option<f64>,
    /// Income before tax.
    #[serde(rename = "incomeBeforeTax")]
    pub income_before_tax: Option<f64>,
    /// Income before tax ratio.
    #[serde(rename = "incomeBeforeTaxRatio")]
    pub income_before_tax_ratio: Option<f64>,
    /// Income tax expense.
    #[serde(rename = "incomeTaxExpense")]
    pub income_tax_expense: Option<f64>,
    /// Net income.
    #[serde(rename = "netIncome")]
    pub net_income: Option<f64>,
    /// Net income ratio.
    #[serde(rename = "netIncomeRatio")]
    pub net_income_ratio: Option<f64>,
    /// Earnings per share (basic).
    pub eps: Option<f64>,
    /// Earnings per share (diluted).
    #[serde(rename = "epsdiluted")]
    pub eps_diluted: Option<f64>,
    /// Weighted average shares outstanding.
    #[serde(rename = "weightedAverageShsOut")]
    pub weighted_average_shs_out: Option<f64>,
    /// Weighted average shares outstanding (diluted).
    #[serde(rename = "weightedAverageShsOutDil")]
    pub weighted_average_shs_out_dil: Option<f64>,
    /// Link to SEC filing.
    pub link: Option<String>,
    /// Final link to filing.
    #[serde(rename = "finalLink")]
    pub final_link: Option<String>,
}

/// Balance sheet statement from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BalanceSheet {
    /// Filing date.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Reported currency.
    #[serde(rename = "reportedCurrency")]
    pub reported_currency: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Filing date.
    #[serde(rename = "fillingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period.
    pub period: Option<String>,
    /// Cash and cash equivalents.
    #[serde(rename = "cashAndCashEquivalents")]
    pub cash_and_cash_equivalents: Option<f64>,
    /// Short-term investments.
    #[serde(rename = "shortTermInvestments")]
    pub short_term_investments: Option<f64>,
    /// Cash and short-term investments.
    #[serde(rename = "cashAndShortTermInvestments")]
    pub cash_and_short_term_investments: Option<f64>,
    /// Net receivables.
    #[serde(rename = "netReceivables")]
    pub net_receivables: Option<f64>,
    /// Inventory.
    pub inventory: Option<f64>,
    /// Other current assets.
    #[serde(rename = "otherCurrentAssets")]
    pub other_current_assets: Option<f64>,
    /// Total current assets.
    #[serde(rename = "totalCurrentAssets")]
    pub total_current_assets: Option<f64>,
    /// Property, plant and equipment net.
    #[serde(rename = "propertyPlantEquipmentNet")]
    pub property_plant_equipment_net: Option<f64>,
    /// Goodwill.
    pub goodwill: Option<f64>,
    /// Intangible assets.
    #[serde(rename = "intangibleAssets")]
    pub intangible_assets: Option<f64>,
    /// Goodwill and intangible assets.
    #[serde(rename = "goodwillAndIntangibleAssets")]
    pub goodwill_and_intangible_assets: Option<f64>,
    /// Long-term investments.
    #[serde(rename = "longTermInvestments")]
    pub long_term_investments: Option<f64>,
    /// Tax assets.
    #[serde(rename = "taxAssets")]
    pub tax_assets: Option<f64>,
    /// Other non-current assets.
    #[serde(rename = "otherNonCurrentAssets")]
    pub other_non_current_assets: Option<f64>,
    /// Total non-current assets.
    #[serde(rename = "totalNonCurrentAssets")]
    pub total_non_current_assets: Option<f64>,
    /// Other assets.
    #[serde(rename = "otherAssets")]
    pub other_assets: Option<f64>,
    /// Total assets.
    #[serde(rename = "totalAssets")]
    pub total_assets: Option<f64>,
    /// Account payables.
    #[serde(rename = "accountPayables")]
    pub account_payables: Option<f64>,
    /// Short-term debt.
    #[serde(rename = "shortTermDebt")]
    pub short_term_debt: Option<f64>,
    /// Tax payables.
    #[serde(rename = "taxPayables")]
    pub tax_payables: Option<f64>,
    /// Deferred revenue.
    #[serde(rename = "deferredRevenue")]
    pub deferred_revenue: Option<f64>,
    /// Other current liabilities.
    #[serde(rename = "otherCurrentLiabilities")]
    pub other_current_liabilities: Option<f64>,
    /// Total current liabilities.
    #[serde(rename = "totalCurrentLiabilities")]
    pub total_current_liabilities: Option<f64>,
    /// Long-term debt.
    #[serde(rename = "longTermDebt")]
    pub long_term_debt: Option<f64>,
    /// Deferred revenue non-current.
    #[serde(rename = "deferredRevenueNonCurrent")]
    pub deferred_revenue_non_current: Option<f64>,
    /// Deferred tax liabilities non-current.
    #[serde(rename = "deferredTaxLiabilitiesNonCurrent")]
    pub deferred_tax_liabilities_non_current: Option<f64>,
    /// Other non-current liabilities.
    #[serde(rename = "otherNonCurrentLiabilities")]
    pub other_non_current_liabilities: Option<f64>,
    /// Total non-current liabilities.
    #[serde(rename = "totalNonCurrentLiabilities")]
    pub total_non_current_liabilities: Option<f64>,
    /// Other liabilities.
    #[serde(rename = "otherLiabilities")]
    pub other_liabilities: Option<f64>,
    /// Capital lease obligations.
    #[serde(rename = "capitalLeaseObligations")]
    pub capital_lease_obligations: Option<f64>,
    /// Total liabilities.
    #[serde(rename = "totalLiabilities")]
    pub total_liabilities: Option<f64>,
    /// Preferred stock.
    #[serde(rename = "preferredStock")]
    pub preferred_stock: Option<f64>,
    /// Common stock.
    #[serde(rename = "commonStock")]
    pub common_stock: Option<f64>,
    /// Retained earnings.
    #[serde(rename = "retainedEarnings")]
    pub retained_earnings: Option<f64>,
    /// Accumulated other comprehensive income/loss.
    #[serde(rename = "accumulatedOtherComprehensiveIncomeLoss")]
    pub accumulated_other_comprehensive_income_loss: Option<f64>,
    /// Other total stockholders equity.
    #[serde(rename = "othertotalStockholdersEquity")]
    pub other_total_stockholders_equity: Option<f64>,
    /// Total stockholders equity.
    #[serde(rename = "totalStockholdersEquity")]
    pub total_stockholders_equity: Option<f64>,
    /// Total equity.
    #[serde(rename = "totalEquity")]
    pub total_equity: Option<f64>,
    /// Total liabilities and stockholders equity.
    #[serde(rename = "totalLiabilitiesAndStockholdersEquity")]
    pub total_liabilities_and_stockholders_equity: Option<f64>,
    /// Minority interest.
    #[serde(rename = "minorityInterest")]
    pub minority_interest: Option<f64>,
    /// Total liabilities and total equity.
    #[serde(rename = "totalLiabilitiesAndTotalEquity")]
    pub total_liabilities_and_total_equity: Option<f64>,
    /// Total investments.
    #[serde(rename = "totalInvestments")]
    pub total_investments: Option<f64>,
    /// Total debt.
    #[serde(rename = "totalDebt")]
    pub total_debt: Option<f64>,
    /// Net debt.
    #[serde(rename = "netDebt")]
    pub net_debt: Option<f64>,
    /// Link to SEC filing.
    pub link: Option<String>,
    /// Final link to filing.
    #[serde(rename = "finalLink")]
    pub final_link: Option<String>,
}

/// Cash flow statement from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CashFlow {
    /// Filing date.
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Reported currency.
    #[serde(rename = "reportedCurrency")]
    pub reported_currency: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Filing date.
    #[serde(rename = "fillingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "calendarYear")]
    pub calendar_year: Option<String>,
    /// Fiscal period.
    pub period: Option<String>,
    /// Net income.
    #[serde(rename = "netIncome")]
    pub net_income: Option<f64>,
    /// Depreciation and amortization.
    #[serde(rename = "depreciationAndAmortization")]
    pub depreciation_and_amortization: Option<f64>,
    /// Deferred income tax.
    #[serde(rename = "deferredIncomeTax")]
    pub deferred_income_tax: Option<f64>,
    /// Stock-based compensation.
    #[serde(rename = "stockBasedCompensation")]
    pub stock_based_compensation: Option<f64>,
    /// Change in working capital.
    #[serde(rename = "changeInWorkingCapital")]
    pub change_in_working_capital: Option<f64>,
    /// Accounts receivables.
    #[serde(rename = "accountsReceivables")]
    pub accounts_receivables: Option<f64>,
    /// Inventory.
    pub inventory: Option<f64>,
    /// Accounts payables.
    #[serde(rename = "accountsPayables")]
    pub accounts_payables: Option<f64>,
    /// Other working capital.
    #[serde(rename = "otherWorkingCapital")]
    pub other_working_capital: Option<f64>,
    /// Other non-cash items.
    #[serde(rename = "otherNonCashItems")]
    pub other_non_cash_items: Option<f64>,
    /// Net cash provided by operating activities.
    #[serde(rename = "netCashProvidedByOperatingActivities")]
    pub net_cash_provided_by_operating_activities: Option<f64>,
    /// Investments in property, plant and equipment.
    #[serde(rename = "investmentsInPropertyPlantAndEquipment")]
    pub investments_in_property_plant_and_equipment: Option<f64>,
    /// Acquisitions net.
    #[serde(rename = "acquisitionsNet")]
    pub acquisitions_net: Option<f64>,
    /// Purchases of investments.
    #[serde(rename = "purchasesOfInvestments")]
    pub purchases_of_investments: Option<f64>,
    /// Sales/maturities of investments.
    #[serde(rename = "salesMaturitiesOfInvestments")]
    pub sales_maturities_of_investments: Option<f64>,
    /// Other investing activities.
    #[serde(rename = "otherInvestingActivites")]
    pub other_investing_activities: Option<f64>,
    /// Net cash used for investing activities.
    #[serde(rename = "netCashUsedForInvestingActivites")]
    pub net_cash_used_for_investing_activities: Option<f64>,
    /// Debt repayment.
    #[serde(rename = "debtRepayment")]
    pub debt_repayment: Option<f64>,
    /// Common stock issued.
    #[serde(rename = "commonStockIssued")]
    pub common_stock_issued: Option<f64>,
    /// Common stock repurchased.
    #[serde(rename = "commonStockRepurchased")]
    pub common_stock_repurchased: Option<f64>,
    /// Dividends paid.
    #[serde(rename = "dividendsPaid")]
    pub dividends_paid: Option<f64>,
    /// Other financing activities.
    #[serde(rename = "otherFinancingActivites")]
    pub other_financing_activities: Option<f64>,
    /// Net cash used/provided by financing activities.
    #[serde(rename = "netCashUsedProvidedByFinancingActivities")]
    pub net_cash_used_provided_by_financing_activities: Option<f64>,
    /// Effect of forex changes on cash.
    #[serde(rename = "effectOfForexChangesOnCash")]
    pub effect_of_forex_changes_on_cash: Option<f64>,
    /// Net change in cash.
    #[serde(rename = "netChangeInCash")]
    pub net_change_in_cash: Option<f64>,
    /// Cash at end of period.
    #[serde(rename = "cashAtEndOfPeriod")]
    pub cash_at_end_of_period: Option<f64>,
    /// Cash at beginning of period.
    #[serde(rename = "cashAtBeginningOfPeriod")]
    pub cash_at_beginning_of_period: Option<f64>,
    /// Operating cash flow.
    #[serde(rename = "operatingCashFlow")]
    pub operating_cash_flow: Option<f64>,
    /// Capital expenditure.
    #[serde(rename = "capitalExpenditure")]
    pub capital_expenditure: Option<f64>,
    /// Free cash flow.
    #[serde(rename = "freeCashFlow")]
    pub free_cash_flow: Option<f64>,
    /// Link to SEC filing.
    pub link: Option<String>,
    /// Final link to filing.
    #[serde(rename = "finalLink")]
    pub final_link: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch income statements for a symbol.
///
/// Returns quarterly or annual income statements. FMP returns an array.
pub async fn income_statement(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<IncomeStatement>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/income-statement/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch balance sheet statements for a symbol.
pub async fn balance_sheet(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<BalanceSheet>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/balance-sheet-statement/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch cash flow statements for a symbol.
pub async fn cash_flow(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<CashFlow>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/cash-flow-statement/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch income statements as reported for a symbol.
pub async fn income_statement_as_reported(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/income-statement-as-reported/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch balance sheet statements as reported for a symbol.
pub async fn balance_sheet_as_reported(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/balance-sheet-statement-as-reported/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch cash flow statements as reported for a symbol.
pub async fn cash_flow_as_reported(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<serde_json::Value>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            &format!("/api/v3/cash-flow-statement-as-reported/{symbol}"),
            &[("period", period.as_str()), ("limit", &limit_str)],
        )
        .await
}

/// Fetch full financial statement as reported for a symbol.
pub async fn full_financial_statement(
    symbol: &str,
    period: Period,
) -> Result<Vec<serde_json::Value>> {
    let client = super::build_client()?;
    client
        .get(
            &format!("/api/v3/financial-statement-full-as-reported/{symbol}"),
            &[("period", period.as_str())],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_income_statement_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/income-statement/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "quarter".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "2".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "date": "2024-03-30",
                    "symbol": "AAPL",
                    "reportedCurrency": "USD",
                    "calendarYear": "2024",
                    "period": "Q2",
                    "revenue": 90753000000.0,
                    "costOfRevenue": 49141000000.0,
                    "grossProfit": 41612000000.0,
                    "netIncome": 23636000000.0,
                    "eps": 1.53,
                    "epsdiluted": 1.52
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<IncomeStatement> = client
            .get(
                "/api/v3/income-statement/AAPL",
                &[("period", "quarter"), ("limit", "2")],
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].revenue, Some(90753000000.0));
        assert_eq!(result[0].eps, Some(1.53));
    }

    #[tokio::test]
    async fn test_balance_sheet_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/balance-sheet-statement/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "annual".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "1".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "date": "2024-09-28",
                    "symbol": "AAPL",
                    "totalAssets": 364980000000.0,
                    "totalLiabilities": 308030000000.0,
                    "totalStockholdersEquity": 56950000000.0,
                    "cashAndCashEquivalents": 29943000000.0
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<BalanceSheet> = client
            .get(
                "/api/v3/balance-sheet-statement/AAPL",
                &[("period", "annual"), ("limit", "1")],
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total_assets, Some(364980000000.0));
        assert_eq!(result[0].total_stockholders_equity, Some(56950000000.0));
    }
}
