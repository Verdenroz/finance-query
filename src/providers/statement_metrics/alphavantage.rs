use crate::constants::StatementType;

pub(super) fn table(statement: StatementType) -> &'static [(&'static str, &'static str)] {
    match statement {
        StatementType::Income => INCOME,
        StatementType::Balance => BALANCE,
        StatementType::CashFlow => CASHFLOW,
    }
}

/// Alpha Vantage reports these cash outflows as positive numbers; Yahoo reports them negative.
pub(super) fn negated(statement: StatementType) -> &'static [&'static str] {
    match statement {
        StatementType::CashFlow => CASHFLOW_OUTFLOWS,
        _ => &[],
    }
}

const CASHFLOW_OUTFLOWS: &[&str] = &[
    "capitalExpenditures",
    "dividendPayout",
    "dividendPayoutCommonStock",
    "paymentsForRepurchaseOfCommonStock",
    "paymentsForRepurchaseOfEquity",
];

const INCOME: &[(&str, &str)] = &[
    ("totalRevenue", "total_revenue"),
    ("costOfRevenue", "cost_of_revenue"),
    ("grossProfit", "gross_profit"),
    ("operatingIncome", "operating_income"),
    (
        "sellingGeneralAndAdministrative",
        "selling_general_and_administration",
    ),
    ("researchAndDevelopment", "research_and_development"),
    ("operatingExpenses", "operating_expense"),
    ("netInterestIncome", "net_interest_income"),
    ("interestIncome", "interest_income"),
    ("interestExpense", "interest_expense"),
    (
        "depreciationAndAmortization",
        "depreciation_and_amortization",
    ),
    ("incomeBeforeTax", "pretax_income"),
    ("incomeTaxExpense", "tax_provision"),
    ("ebit", "ebit"),
    ("ebitda", "ebitda"),
    ("netIncome", "net_income"),
];

const BALANCE: &[(&str, &str)] = &[
    ("totalAssets", "total_assets"),
    ("totalCurrentAssets", "current_assets"),
    (
        "cashAndCashEquivalentsAtCarryingValue",
        "cash_and_cash_equivalents",
    ),
    (
        "cashAndShortTermInvestments",
        "cash_cash_equivalents_and_short_term_investments",
    ),
    ("inventory", "inventory"),
    ("currentNetReceivables", "receivables"),
    ("totalNonCurrentAssets", "total_non_current_assets"),
    ("propertyPlantEquipment", "net_ppe"),
    (
        "accumulatedDepreciationAmortizationPPE",
        "accumulated_depreciation",
    ),
    ("intangibleAssets", "goodwill_and_other_intangible_assets"),
    (
        "intangibleAssetsExcludingGoodwill",
        "other_intangible_assets",
    ),
    ("goodwill", "goodwill"),
    ("longTermInvestments", "investments_and_advances"),
    ("otherCurrentAssets", "other_current_assets"),
    ("otherNonCurrentAssets", "other_non_current_assets"),
    (
        "totalLiabilities",
        "total_liabilities_net_minority_interest",
    ),
    ("totalCurrentLiabilities", "current_liabilities"),
    ("currentAccountsPayable", "accounts_payable"),
    ("deferredRevenue", "current_deferred_revenue"),
    ("currentDebt", "current_debt"),
    (
        "totalNonCurrentLiabilities",
        "total_non_current_liabilities_net_minority_interest",
    ),
    ("longTermDebtNoncurrent", "long_term_debt"),
    ("otherCurrentLiabilities", "other_current_liabilities"),
    (
        "otherNonCurrentLiabilities",
        "other_non_current_liabilities",
    ),
    ("totalShareholderEquity", "stockholders_equity"),
    ("treasuryStock", "treasury_stock"),
    ("retainedEarnings", "retained_earnings"),
    ("commonStock", "common_stock"),
    ("commonStockSharesOutstanding", "ordinary_shares_number"),
];

const CASHFLOW: &[(&str, &str)] = &[
    ("netIncome", "net_income_from_continuing_operations"),
    ("operatingCashflow", "operating_cash_flow"),
    (
        "depreciationDepletionAndAmortization",
        "depreciation_and_amortization",
    ),
    ("capitalExpenditures", "capital_expenditure"),
    ("changeInReceivables", "change_in_receivables"),
    ("changeInInventory", "change_in_inventory"),
    ("cashflowFromInvestment", "investing_cash_flow"),
    ("cashflowFromFinancing", "financing_cash_flow"),
    (
        "proceedsFromRepaymentsOfShortTermDebt",
        "net_short_term_debt_issuance",
    ),
    (
        "paymentsForRepurchaseOfCommonStock",
        "common_stock_payments",
    ),
    (
        "paymentsForRepurchaseOfEquity",
        "repurchase_of_capital_stock",
    ),
    ("dividendPayout", "cash_dividends_paid"),
    ("dividendPayoutCommonStock", "common_stock_dividend_paid"),
    ("proceedsFromIssuanceOfCommonStock", "common_stock_issuance"),
    (
        "proceedsFromIssuanceOfLongTermDebtAndCapitalSecuritiesNet",
        "long_term_debt_issuance",
    ),
    ("changeInCashAndCashEquivalents", "changes_in_cash"),
    ("changeInExchangeRate", "effect_of_exchange_rate_changes"),
];
