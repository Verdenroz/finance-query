"""
Constants for Yahoo Finance fundamentals-timeseries API.

These field names correspond to the 'type' parameter in the Yahoo Finance API.
"""

# Income Statement Fields
INCOME_STATEMENT_FIELDS = [
    "TotalRevenue",
    "OperatingRevenue",
    "CostOfRevenue",
    "GrossProfit",
    "OperatingExpense",
    "SellingGeneralAndAdministration",
    "ResearchAndDevelopment",
    "OperatingIncome",
    "NetInterestIncome",
    "InterestExpense",
    "InterestIncome",
    "NetNonOperatingInterestIncomeExpense",
    "OtherIncomeExpense",
    "PretaxIncome",
    "TaxProvision",
    "NetIncomeCommonStockholders",
    "NetIncome",
    "DilutedEPS",
    "BasicEPS",
    "DilutedAverageShares",
    "BasicAverageShares",
    "EBIT",
    "EBITDA",
    "ReconciledCostOfRevenue",
    "ReconciledDepreciation",
    "NetIncomeFromContinuingOperationNetMinorityInterest",
    "NormalizedEBITDA",
    "TotalExpenses",
    "TotalOperatingIncomeAsReported",
]

# Balance Sheet Fields
BALANCE_SHEET_FIELDS = [
    "TotalAssets",
    "CurrentAssets",
    "CashCashEquivalentsAndShortTermInvestments",
    "CashAndCashEquivalents",
    "CashFinancial",
    "Receivables",
    "AccountsReceivable",
    "Inventory",
    "PrepaidAssets",
    "OtherCurrentAssets",
    "TotalNonCurrentAssets",
    "NetPPE",
    "GrossPPE",
    "AccumulatedDepreciation",
    "Goodwill",
    "GoodwillAndOtherIntangibleAssets",
    "OtherIntangibleAssets",
    "InvestmentsAndAdvances",
    "LongTermEquityInvestment",
    "OtherNonCurrentAssets",
    "TotalLiabilitiesNetMinorityInterest",
    "CurrentLiabilities",
    "PayablesAndAccruedExpenses",
    "AccountsPayable",
    "CurrentDebt",
    "CurrentDeferredRevenue",
    "OtherCurrentLiabilities",
    "TotalNonCurrentLiabilitiesNetMinorityInterest",
    "LongTermDebt",
    "LongTermDebtAndCapitalLeaseObligation",
    "NonCurrentDeferredRevenue",
    "NonCurrentDeferredTaxesLiabilities",
    "OtherNonCurrentLiabilities",
    "StockholdersEquity",
    "CommonStockEquity",
    "CommonStock",
    "RetainedEarnings",
    "AdditionalPaidInCapital",
    "TreasuryStock",
    "TotalEquityGrossMinorityInterest",
    "WorkingCapital",
    "InvestedCapital",
    "TangibleBookValue",
    "TotalDebt",
    "NetDebt",
    "ShareIssued",
    "OrdinarySharesNumber",
]

# Cash Flow Fields
CASH_FLOW_FIELDS = [
    "OperatingCashFlow",
    "CashFlowFromContinuingOperatingActivities",
    "NetIncomeFromContinuingOperations",
    "DepreciationAndAmortization",
    "DeferredIncomeTax",
    "ChangeInWorkingCapital",
    "ChangeInReceivables",
    "ChangesInAccountReceivables",
    "ChangeInInventory",
    "ChangeInAccountPayable",
    "ChangeInOtherWorkingCapital",
    "StockBasedCompensation",
    "OtherNonCashItems",
    "InvestingCashFlow",
    "CashFlowFromContinuingInvestingActivities",
    "NetPPEPurchaseAndSale",
    "PurchaseOfPPE",
    "SaleOfPPE",
    "CapitalExpenditure",
    "NetBusinessPurchaseAndSale",
    "PurchaseOfBusiness",
    "SaleOfBusiness",
    "NetInvestmentPurchaseAndSale",
    "PurchaseOfInvestment",
    "SaleOfInvestment",
    "NetOtherInvestingChanges",
    "FinancingCashFlow",
    "CashFlowFromContinuingFinancingActivities",
    "NetIssuancePaymentsOfDebt",
    "NetLongTermDebtIssuance",
    "LongTermDebtIssuance",
    "LongTermDebtPayments",
    "NetShortTermDebtIssuance",
    "NetCommonStockIssuance",
    "CommonStockIssuance",
    "CommonStockPayments",
    "RepurchaseOfCapitalStock",
    "CashDividendsPaid",
    "CommonStockDividendPaid",
    "NetOtherFinancingCharges",
    "EndCashPosition",
    "BeginningCashPosition",
    "ChangesinCash",
    "EffectOfExchangeRateChanges",
    "FreeCashFlow",
    "CapitalExpenditureReported",
]


# Mapping of statement types to their field lists
STATEMENT_TYPE_FIELDS = {
    "income": INCOME_STATEMENT_FIELDS,
    "balance": BALANCE_SHEET_FIELDS,
    "cashflow": CASH_FLOW_FIELDS,
}


def get_statement_fields(statement_type: str, frequency: str) -> list[str]:
    """
    Get the list of fields for a given statement type and frequency.

    Args:
        statement_type: One of 'income', 'balance', 'cashflow'
        frequency: One of 'annual', 'quarterly'

    Returns:
        List of field names with frequency prefix (e.g., 'annualTotalRevenue')
    """
    fields = STATEMENT_TYPE_FIELDS.get(statement_type)
    if fields is None:
        raise ValueError(f"Invalid statement type: {statement_type}. Must be one of {list(STATEMENT_TYPE_FIELDS.keys())}")

    return [f"{frequency}{field}" for field in fields]
