//! FMP financial statement endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::models::Period;

// ============================================================================
// Response types
// ============================================================================

/// Income statement from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IncomeStatementDTO {
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
    #[serde(rename = "filingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "fiscalYear")]
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
    /// Net interest income.
    #[serde(rename = "netInterestIncome")]
    pub net_interest_income: Option<f64>,
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
    /// EBIT.
    pub ebit: Option<f64>,
    /// Non-operating income excluding interest.
    #[serde(rename = "nonOperatingIncomeExcludingInterest")]
    pub non_operating_income_excluding_interest: Option<f64>,
    /// Operating income.
    #[serde(rename = "operatingIncome")]
    pub operating_income: Option<f64>,
    /// Total other income/expenses net.
    #[serde(rename = "totalOtherIncomeExpensesNet")]
    pub total_other_income_expenses_net: Option<f64>,
    /// Income before tax.
    #[serde(rename = "incomeBeforeTax")]
    pub income_before_tax: Option<f64>,
    /// Income tax expense.
    #[serde(rename = "incomeTaxExpense")]
    pub income_tax_expense: Option<f64>,
    /// Net income from continuing operations.
    #[serde(rename = "netIncomeFromContinuingOperations")]
    pub net_income_from_continuing_operations: Option<f64>,
    /// Net income from discontinued operations.
    #[serde(rename = "netIncomeFromDiscontinuedOperations")]
    pub net_income_from_discontinued_operations: Option<f64>,
    /// Other adjustments to net income.
    #[serde(rename = "otherAdjustmentsToNetIncome")]
    pub other_adjustments_to_net_income: Option<f64>,
    /// Net income.
    #[serde(rename = "netIncome")]
    pub net_income: Option<f64>,
    /// Net income deductions.
    #[serde(rename = "netIncomeDeductions")]
    pub net_income_deductions: Option<f64>,
    /// Bottom-line net income.
    #[serde(rename = "bottomLineNetIncome")]
    pub bottom_line_net_income: Option<f64>,
    /// Earnings per share (basic).
    pub eps: Option<f64>,
    /// Earnings per share (diluted).
    #[serde(rename = "epsDiluted")]
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
pub struct BalanceSheetDTO {
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
    #[serde(rename = "filingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "fiscalYear")]
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
    /// Accounts receivable.
    #[serde(rename = "accountsReceivables")]
    pub accounts_receivables: Option<f64>,
    /// Other receivables.
    #[serde(rename = "otherReceivables")]
    pub other_receivables: Option<f64>,
    /// Inventory.
    pub inventory: Option<f64>,
    /// Prepaid expenses.
    pub prepaids: Option<f64>,
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
    /// Total payables.
    #[serde(rename = "totalPayables")]
    pub total_payables: Option<f64>,
    /// Account payables.
    #[serde(rename = "accountPayables")]
    pub account_payables: Option<f64>,
    /// Other payables.
    #[serde(rename = "otherPayables")]
    pub other_payables: Option<f64>,
    /// Accrued expenses.
    #[serde(rename = "accruedExpenses")]
    pub accrued_expenses: Option<f64>,
    /// Short-term debt.
    #[serde(rename = "shortTermDebt")]
    pub short_term_debt: Option<f64>,
    /// Current capital lease obligations.
    #[serde(rename = "capitalLeaseObligationsCurrent")]
    pub capital_lease_obligations_current: Option<f64>,
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
    /// Non-current capital lease obligations.
    #[serde(rename = "capitalLeaseObligationsNonCurrent")]
    pub capital_lease_obligations_non_current: Option<f64>,
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
    /// Treasury stock.
    #[serde(rename = "treasuryStock")]
    pub treasury_stock: Option<f64>,
    /// Preferred stock.
    #[serde(rename = "preferredStock")]
    pub preferred_stock: Option<f64>,
    /// Common stock.
    #[serde(rename = "commonStock")]
    pub common_stock: Option<f64>,
    /// Retained earnings.
    #[serde(rename = "retainedEarnings")]
    pub retained_earnings: Option<f64>,
    /// Additional paid-in capital.
    #[serde(rename = "additionalPaidInCapital")]
    pub additional_paid_in_capital: Option<f64>,
    /// Accumulated other comprehensive income/loss.
    #[serde(rename = "accumulatedOtherComprehensiveIncomeLoss")]
    pub accumulated_other_comprehensive_income_loss: Option<f64>,
    /// Other total stockholders equity.
    #[serde(rename = "otherTotalStockholdersEquity")]
    pub other_total_stockholders_equity: Option<f64>,
    /// Total stockholders equity.
    #[serde(rename = "totalStockholdersEquity")]
    pub total_stockholders_equity: Option<f64>,
    /// Total equity.
    #[serde(rename = "totalEquity")]
    pub total_equity: Option<f64>,
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
pub struct CashFlowDTO {
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
    #[serde(rename = "filingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// Calendar year.
    #[serde(rename = "fiscalYear")]
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
    #[serde(rename = "otherInvestingActivities")]
    pub other_investing_activities: Option<f64>,
    /// Net cash used for investing activities.
    #[serde(rename = "netCashProvidedByInvestingActivities")]
    pub net_cash_used_for_investing_activities: Option<f64>,
    /// Net debt issued less repaid.
    #[serde(rename = "netDebtIssuance")]
    pub debt_repayment: Option<f64>,
    /// Net long-term debt issued less repaid.
    #[serde(rename = "longTermNetDebtIssuance")]
    pub long_term_net_debt_issuance: Option<f64>,
    /// Net short-term debt issued less repaid.
    #[serde(rename = "shortTermNetDebtIssuance")]
    pub short_term_net_debt_issuance: Option<f64>,
    /// Net stock issued less repurchased.
    #[serde(rename = "netStockIssuance")]
    pub net_stock_issuance: Option<f64>,
    /// Net common stock issued less repurchased.
    #[serde(rename = "netCommonStockIssuance")]
    pub net_common_stock_issuance: Option<f64>,
    /// Common stock issued.
    #[serde(rename = "commonStockIssuance")]
    pub common_stock_issued: Option<f64>,
    /// Common stock repurchased.
    #[serde(rename = "commonStockRepurchased")]
    pub common_stock_repurchased: Option<f64>,
    /// Net preferred stock issued less repurchased.
    #[serde(rename = "netPreferredStockIssuance")]
    pub net_preferred_stock_issuance: Option<f64>,
    /// Dividends paid, net.
    #[serde(rename = "netDividendsPaid")]
    pub dividends_paid: Option<f64>,
    /// Common dividends paid.
    #[serde(rename = "commonDividendsPaid")]
    pub common_dividends_paid: Option<f64>,
    /// Preferred dividends paid.
    #[serde(rename = "preferredDividendsPaid")]
    pub preferred_dividends_paid: Option<f64>,
    /// Other financing activities.
    #[serde(rename = "otherFinancingActivities")]
    pub other_financing_activities: Option<f64>,
    /// Net cash used/provided by financing activities.
    #[serde(rename = "netCashProvidedByFinancingActivities")]
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
    /// Income taxes paid in cash.
    #[serde(rename = "incomeTaxesPaid")]
    pub income_taxes_paid: Option<f64>,
    /// Interest paid in cash.
    #[serde(rename = "interestPaid")]
    pub interest_paid: Option<f64>,
    /// Link to SEC filing.
    pub link: Option<String>,
    /// Final link to filing.
    #[serde(rename = "finalLink")]
    pub final_link: Option<String>,
}

// ============================================================================
// Canonical conversion functions
// ============================================================================

/// Pivot a Vec of serde-serializable financial statements into the canonical
/// `HashMap<String, HashMap<String, serde_json::Value>>` format used by
/// `FinancialStatementData`.
///
/// Each statement should have a `date` field and arbitrary numeric metric fields.
pub fn pivot_financials<T: serde::Serialize>(
    stmts: Vec<T>,
) -> std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>> {
    let mut data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, serde_json::Value>,
    > = std::collections::HashMap::new();

    for stmt in stmts {
        let val = match serde_json::to_value(&stmt) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let obj = match val.as_object() {
            Some(o) => o,
            None => continue,
        };

        let date = match obj.get("date").and_then(|v| v.as_str()) {
            Some(d) if !d.is_empty() => d.to_string(),
            _ => continue,
        };

        for (key, value) in obj {
            if key == "date" {
                continue;
            }
            // Only include numeric values
            match &value {
                serde_json::Value::Number(_) => {}
                _ => continue,
            }
            data.entry(key.clone())
                .or_default()
                .insert(date.clone(), value.clone());
        }
    }

    data
}

/// Fetch canonical financial statement data for a symbol.
pub async fn fetch_financials_data(
    symbol: &str,
    stmt_type: crate::StatementType,
    period: Period,
    limit: Option<u32>,
) -> Result<std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>>
{
    let data = match stmt_type {
        crate::StatementType::Income => {
            let stmts = income_statement(symbol, period, limit).await?;
            pivot_financials(stmts)
        }
        crate::StatementType::Balance => {
            let stmts = balance_sheet(symbol, period, limit).await?;
            pivot_financials(stmts)
        }
        crate::StatementType::CashFlow => {
            let stmts = cash_flow(symbol, period, limit).await?;
            pivot_financials(stmts)
        }
    };
    Ok(data)
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
) -> Result<Vec<IncomeStatementDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            "/stable/income-statement",
            &[
                ("symbol", symbol),
                ("period", period.as_str()),
                ("limit", &limit_str),
            ],
        )
        .await
}

/// Fetch balance sheet statements for a symbol.
pub async fn balance_sheet(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<BalanceSheetDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            "/stable/balance-sheet-statement",
            &[
                ("symbol", symbol),
                ("period", period.as_str()),
                ("limit", &limit_str),
            ],
        )
        .await
}

/// Fetch cash flow statements for a symbol.
pub async fn cash_flow(
    symbol: &str,
    period: Period,
    limit: Option<u32>,
) -> Result<Vec<CashFlowDTO>> {
    let client = crate::adapters::fmp::build_client()?;
    let limit_str = limit.unwrap_or(4).to_string();
    client
        .get(
            "/stable/cash-flow-statement",
            &[
                ("symbol", symbol),
                ("period", period.as_str()),
                ("limit", &limit_str),
            ],
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
            .mock("GET", "/stable/income-statement")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("period".into(), "quarter".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "2".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"[{
                    "date": "2024-03-30",
                    "symbol": "AAPL",
                    "reportedCurrency": "USD",
                    "filingDate": "2024-05-03",
                    "fiscalYear": "2024",
                    "period": "Q2",
                    "revenue": 90753000000.0,
                    "costOfRevenue": 49141000000.0,
                    "grossProfit": 41612000000.0,
                    "ebit": 27900000000.0,
                    "netIncome": 23636000000.0,
                    "bottomLineNetIncome": 23636000000.0,
                    "eps": 1.53,
                    "epsDiluted": 1.52
                }]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<IncomeStatementDTO> = client
            .get(
                "/stable/income-statement",
                &[("period", "quarter"), ("limit", "2")],
            )
            .await
            .unwrap();

        let stmt = &result[0];
        assert_eq!(stmt.symbol.as_deref(), Some("AAPL"));
        assert_eq!(stmt.revenue, Some(90_753_000_000.0));
        assert_eq!(stmt.eps, Some(1.53));
        assert_eq!(stmt.eps_diluted, Some(1.52));
        assert_eq!(stmt.filling_date.as_deref(), Some("2024-05-03"));
        assert_eq!(stmt.calendar_year.as_deref(), Some("2024"));
        assert_eq!(stmt.ebit, Some(27_900_000_000.0));
        assert_eq!(stmt.bottom_line_net_income, Some(23_636_000_000.0));
    }

    #[tokio::test]
    async fn cash_flow_reads_the_renamed_financing_keys() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/cash-flow-statement")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body(
                r#"[{
                    "date": "2024-09-28",
                    "symbol": "AAPL",
                    "filingDate": "2024-11-01",
                    "fiscalYear": "2024",
                    "period": "FY",
                    "netIncome": 93736000000.0,
                    "otherInvestingActivities": -1176000000.0,
                    "netCashProvidedByInvestingActivities": 2935000000.0,
                    "netDebtIssuance": -5998000000.0,
                    "commonStockIssuance": 0.0,
                    "netDividendsPaid": -15234000000.0,
                    "otherFinancingActivities": -5802000000.0,
                    "netCashProvidedByFinancingActivities": -121983000000.0,
                    "freeCashFlow": 108807000000.0
                }]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<CashFlowDTO> = client
            .get("/stable/cash-flow-statement", &[])
            .await
            .unwrap();

        let stmt = &result[0];
        assert_eq!(stmt.filling_date.as_deref(), Some("2024-11-01"));
        assert_eq!(stmt.calendar_year.as_deref(), Some("2024"));
        assert_eq!(stmt.other_investing_activities, Some(-1_176_000_000.0));
        assert_eq!(
            stmt.net_cash_used_for_investing_activities,
            Some(2_935_000_000.0)
        );
        assert_eq!(stmt.debt_repayment, Some(-5_998_000_000.0));
        assert_eq!(stmt.common_stock_issued, Some(0.0));
        assert_eq!(stmt.dividends_paid, Some(-15_234_000_000.0));
        assert_eq!(stmt.other_financing_activities, Some(-5_802_000_000.0));
        assert_eq!(
            stmt.net_cash_used_provided_by_financing_activities,
            Some(-121_983_000_000.0)
        );
    }

    #[test]
    fn pivot_keeps_numeric_metrics_only() {
        let stmts: Vec<IncomeStatementDTO> = serde_json::from_str(
            r#"[{"date":"2024-09-28","symbol":"AAPL","fiscalYear":"2024","revenue":391035000000.0}]"#,
        )
        .unwrap();

        let data = pivot_financials(stmts);
        assert_eq!(
            data["revenue"]["2024-09-28"],
            serde_json::json!(391_035_000_000.0)
        );
        assert!(!data.contains_key("symbol"));
        assert!(!data.contains_key("fiscalYear"));
        // Absent keys are omitted rather than emitted as explicit nulls.
        assert!(!data.contains_key("ebit"));
    }

    #[tokio::test]
    async fn test_balance_sheet_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/balance-sheet-statement")
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<BalanceSheetDTO> = client
            .get(
                "/stable/balance-sheet-statement",
                &[("period", "annual"), ("limit", "1")],
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total_assets, Some(364980000000.0));
        assert_eq!(result[0].total_stockholders_equity, Some(56950000000.0));
    }
}
