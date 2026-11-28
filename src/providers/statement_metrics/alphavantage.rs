use super::{Table, negate, rename};

pub(super) const INCOME: Table = &[
    rename("totalRevenue", "total_revenue"),
    rename("costOfRevenue", "cost_of_revenue"),
    rename("grossProfit", "gross_profit"),
    rename("operatingIncome", "operating_income"),
    rename(
        "sellingGeneralAndAdministrative",
        "selling_general_and_administration",
    ),
    rename("researchAndDevelopment", "research_and_development"),
    rename("operatingExpenses", "operating_expense"),
    rename("netInterestIncome", "net_interest_income"),
    rename("interestIncome", "interest_income"),
    rename("interestExpense", "interest_expense"),
    rename(
        "depreciationAndAmortization",
        "depreciation_and_amortization",
    ),
    rename("incomeBeforeTax", "pretax_income"),
    rename("incomeTaxExpense", "tax_provision"),
    rename("ebit", "ebit"),
    rename("ebitda", "ebitda"),
    rename("netIncome", "net_income"),
];

pub(super) const BALANCE: Table = &[
    rename("totalAssets", "total_assets"),
    rename("totalCurrentAssets", "current_assets"),
    rename(
        "cashAndCashEquivalentsAtCarryingValue",
        "cash_and_cash_equivalents",
    ),
    rename(
        "cashAndShortTermInvestments",
        "cash_cash_equivalents_and_short_term_investments",
    ),
    rename("inventory", "inventory"),
    rename("currentNetReceivables", "receivables"),
    rename("totalNonCurrentAssets", "total_non_current_assets"),
    rename("propertyPlantEquipment", "net_ppe"),
    rename(
        "accumulatedDepreciationAmortizationPPE",
        "accumulated_depreciation",
    ),
    rename("intangibleAssets", "goodwill_and_other_intangible_assets"),
    rename(
        "intangibleAssetsExcludingGoodwill",
        "other_intangible_assets",
    ),
    rename("goodwill", "goodwill"),
    rename("longTermInvestments", "investments_and_advances"),
    rename("otherCurrentAssets", "other_current_assets"),
    rename("otherNonCurrentAssets", "other_non_current_assets"),
    rename(
        "totalLiabilities",
        "total_liabilities_net_minority_interest",
    ),
    rename("totalCurrentLiabilities", "current_liabilities"),
    rename("currentAccountsPayable", "accounts_payable"),
    rename("deferredRevenue", "current_deferred_revenue"),
    rename("currentDebt", "current_debt"),
    rename(
        "totalNonCurrentLiabilities",
        "total_non_current_liabilities_net_minority_interest",
    ),
    rename("longTermDebtNoncurrent", "long_term_debt"),
    rename("otherCurrentLiabilities", "other_current_liabilities"),
    rename(
        "otherNonCurrentLiabilities",
        "other_non_current_liabilities",
    ),
    rename("totalShareholderEquity", "stockholders_equity"),
    rename("treasuryStock", "treasury_stock"),
    rename("retainedEarnings", "retained_earnings"),
    rename("commonStock", "common_stock"),
    rename("commonStockSharesOutstanding", "ordinary_shares_number"),
];

pub(super) const CASHFLOW: Table = &[
    rename("netIncome", "net_income_from_continuing_operations"),
    rename("operatingCashflow", "operating_cash_flow"),
    rename(
        "depreciationDepletionAndAmortization",
        "depreciation_and_amortization",
    ),
    negate("capitalExpenditures", "capital_expenditure"),
    rename("changeInReceivables", "change_in_receivables"),
    rename("changeInInventory", "change_in_inventory"),
    rename("cashflowFromInvestment", "investing_cash_flow"),
    rename("cashflowFromFinancing", "financing_cash_flow"),
    rename(
        "proceedsFromRepaymentsOfShortTermDebt",
        "net_short_term_debt_issuance",
    ),
    negate(
        "paymentsForRepurchaseOfCommonStock",
        "common_stock_payments",
    ),
    negate(
        "paymentsForRepurchaseOfEquity",
        "repurchase_of_capital_stock",
    ),
    negate("dividendPayout", "cash_dividends_paid"),
    negate("dividendPayoutCommonStock", "common_stock_dividend_paid"),
    rename("proceedsFromIssuanceOfCommonStock", "common_stock_issuance"),
    rename(
        "proceedsFromIssuanceOfLongTermDebtAndCapitalSecuritiesNet",
        "long_term_debt_issuance",
    ),
    rename("changeInCashAndCashEquivalents", "changes_in_cash"),
    rename("changeInExchangeRate", "effect_of_exchange_rate_changes"),
];
