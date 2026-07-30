use crate::constants::fundamental_types;
use serde::{Deserialize, Serialize};

/// Statement types for financial data
///
/// The `alias`es mirror the spellings [`FromStr`](std::str::FromStr) accepts, so
/// deserializing (axum path/query extraction, JSON) takes the same spellings parsing does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatementType {
    /// Income statement
    #[serde(alias = "income-statement")]
    Income,
    /// Balance sheet
    #[serde(alias = "balance-sheet")]
    Balance,
    /// Cash flow statement
    #[serde(alias = "cash", alias = "cash-flow")]
    CashFlow,
}

impl StatementType {
    /// Convert statement type to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            StatementType::Income => "income",
            StatementType::Balance => "balance",
            StatementType::CashFlow => "cashflow",
        }
    }

    /// Get the list of fields for this statement type
    ///
    /// Returns field names without frequency prefix (e.g., "TotalRevenue" not "annualTotalRevenue")
    pub fn get_fields(&self) -> &'static [&'static str] {
        match self {
            StatementType::Income => &INCOME_STATEMENT_FIELDS,
            StatementType::Balance => &BALANCE_SHEET_FIELDS,
            StatementType::CashFlow => &CASH_FLOW_FIELDS,
        }
    }
}

impl std::str::FromStr for StatementType {
    type Err = ();

    /// Case-insensitive; accepts `as_str()`'s canonical form plus the hyphenated
    /// `-statement`/`-sheet` variants and the `"cash"` shorthand.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "income" | "income-statement" => Ok(StatementType::Income),
            "balance" | "balance-sheet" => Ok(StatementType::Balance),
            "cash" | "cashflow" | "cash-flow" => Ok(StatementType::CashFlow),
            _ => Err(()),
        }
    }
}

/// Income statement fields (without frequency prefix)
const INCOME_STATEMENT_FIELDS: [&str; 30] = [
    fundamental_types::TOTAL_REVENUE,
    fundamental_types::OPERATING_REVENUE,
    fundamental_types::COST_OF_REVENUE,
    fundamental_types::GROSS_PROFIT,
    fundamental_types::OPERATING_EXPENSE,
    fundamental_types::SELLING_GENERAL_AND_ADMIN,
    fundamental_types::RESEARCH_AND_DEVELOPMENT,
    fundamental_types::OPERATING_INCOME,
    fundamental_types::NET_INTEREST_INCOME,
    fundamental_types::INTEREST_EXPENSE,
    fundamental_types::INTEREST_INCOME,
    fundamental_types::NET_NON_OPERATING_INTEREST_INCOME_EXPENSE,
    fundamental_types::OTHER_INCOME_EXPENSE,
    fundamental_types::PRETAX_INCOME,
    fundamental_types::TAX_PROVISION,
    fundamental_types::NET_INCOME_COMMON_STOCKHOLDERS,
    fundamental_types::NET_INCOME,
    fundamental_types::DILUTED_EPS,
    fundamental_types::BASIC_EPS,
    fundamental_types::DILUTED_AVERAGE_SHARES,
    fundamental_types::BASIC_AVERAGE_SHARES,
    fundamental_types::EBIT,
    fundamental_types::EBITDA,
    fundamental_types::RECONCILED_COST_OF_REVENUE,
    fundamental_types::RECONCILED_DEPRECIATION,
    fundamental_types::NET_INCOME_FROM_CONTINUING_OPERATION_NET_MINORITY_INTEREST,
    fundamental_types::NORMALIZED_EBITDA,
    fundamental_types::TOTAL_EXPENSES,
    fundamental_types::TOTAL_OPERATING_INCOME_AS_REPORTED,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

/// Balance sheet fields (without frequency prefix)
const BALANCE_SHEET_FIELDS: [&str; 48] = [
    fundamental_types::TOTAL_ASSETS,
    fundamental_types::CURRENT_ASSETS,
    fundamental_types::CASH_CASH_EQUIVALENTS_AND_SHORT_TERM_INVESTMENTS,
    fundamental_types::CASH_AND_CASH_EQUIVALENTS,
    fundamental_types::CASH_FINANCIAL,
    fundamental_types::RECEIVABLES,
    fundamental_types::ACCOUNTS_RECEIVABLE,
    fundamental_types::INVENTORY,
    fundamental_types::PREPAID_ASSETS,
    fundamental_types::OTHER_CURRENT_ASSETS,
    fundamental_types::TOTAL_NON_CURRENT_ASSETS,
    fundamental_types::NET_PPE,
    fundamental_types::GROSS_PPE,
    fundamental_types::ACCUMULATED_DEPRECIATION,
    fundamental_types::GOODWILL,
    fundamental_types::GOODWILL_AND_OTHER_INTANGIBLE_ASSETS,
    fundamental_types::OTHER_INTANGIBLE_ASSETS,
    fundamental_types::INVESTMENTS_AND_ADVANCES,
    fundamental_types::LONG_TERM_EQUITY_INVESTMENT,
    fundamental_types::OTHER_NON_CURRENT_ASSETS,
    fundamental_types::TOTAL_LIABILITIES_NET_MINORITY_INTEREST,
    fundamental_types::CURRENT_LIABILITIES,
    fundamental_types::PAYABLES_AND_ACCRUED_EXPENSES,
    fundamental_types::ACCOUNTS_PAYABLE,
    fundamental_types::CURRENT_DEBT,
    fundamental_types::CURRENT_DEFERRED_REVENUE,
    fundamental_types::OTHER_CURRENT_LIABILITIES,
    fundamental_types::TOTAL_NON_CURRENT_LIABILITIES_NET_MINORITY_INTEREST,
    fundamental_types::LONG_TERM_DEBT,
    fundamental_types::LONG_TERM_DEBT_AND_CAPITAL_LEASE_OBLIGATION,
    fundamental_types::NON_CURRENT_DEFERRED_REVENUE,
    fundamental_types::NON_CURRENT_DEFERRED_TAXES_LIABILITIES,
    fundamental_types::OTHER_NON_CURRENT_LIABILITIES,
    fundamental_types::STOCKHOLDERS_EQUITY,
    fundamental_types::COMMON_STOCK_EQUITY,
    fundamental_types::COMMON_STOCK,
    fundamental_types::RETAINED_EARNINGS,
    fundamental_types::ADDITIONAL_PAID_IN_CAPITAL,
    fundamental_types::TREASURY_STOCK,
    fundamental_types::TOTAL_EQUITY_GROSS_MINORITY_INTEREST,
    fundamental_types::WORKING_CAPITAL,
    fundamental_types::INVESTED_CAPITAL,
    fundamental_types::TANGIBLE_BOOK_VALUE,
    fundamental_types::TOTAL_DEBT,
    fundamental_types::NET_DEBT,
    fundamental_types::SHARE_ISSUED,
    fundamental_types::ORDINARY_SHARES_NUMBER,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

/// Cash flow statement fields (without frequency prefix)
const CASH_FLOW_FIELDS: [&str; 47] = [
    fundamental_types::OPERATING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_OPERATING_ACTIVITIES,
    fundamental_types::NET_INCOME_FROM_CONTINUING_OPERATIONS,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
    fundamental_types::DEFERRED_INCOME_TAX,
    fundamental_types::CHANGE_IN_WORKING_CAPITAL,
    fundamental_types::CHANGE_IN_RECEIVABLES,
    fundamental_types::CHANGES_IN_ACCOUNT_RECEIVABLES,
    fundamental_types::CHANGE_IN_INVENTORY,
    fundamental_types::CHANGE_IN_ACCOUNT_PAYABLE,
    fundamental_types::CHANGE_IN_OTHER_WORKING_CAPITAL,
    fundamental_types::STOCK_BASED_COMPENSATION,
    fundamental_types::OTHER_NON_CASH_ITEMS,
    fundamental_types::INVESTING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_INVESTING_ACTIVITIES,
    fundamental_types::NET_PPE_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_PPE,
    fundamental_types::SALE_OF_PPE,
    fundamental_types::CAPITAL_EXPENDITURE,
    fundamental_types::NET_BUSINESS_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_BUSINESS,
    fundamental_types::SALE_OF_BUSINESS,
    fundamental_types::NET_INVESTMENT_PURCHASE_AND_SALE,
    fundamental_types::PURCHASE_OF_INVESTMENT,
    fundamental_types::SALE_OF_INVESTMENT,
    fundamental_types::NET_OTHER_INVESTING_CHANGES,
    fundamental_types::FINANCING_CASH_FLOW,
    fundamental_types::CASH_FLOW_FROM_CONTINUING_FINANCING_ACTIVITIES,
    fundamental_types::NET_ISSUANCE_PAYMENTS_OF_DEBT,
    fundamental_types::NET_LONG_TERM_DEBT_ISSUANCE,
    fundamental_types::LONG_TERM_DEBT_ISSUANCE,
    fundamental_types::LONG_TERM_DEBT_PAYMENTS,
    fundamental_types::NET_SHORT_TERM_DEBT_ISSUANCE,
    fundamental_types::NET_COMMON_STOCK_ISSUANCE,
    fundamental_types::COMMON_STOCK_ISSUANCE,
    fundamental_types::COMMON_STOCK_PAYMENTS,
    fundamental_types::REPURCHASE_OF_CAPITAL_STOCK,
    fundamental_types::CASH_DIVIDENDS_PAID,
    fundamental_types::COMMON_STOCK_DIVIDEND_PAID,
    fundamental_types::NET_OTHER_FINANCING_CHARGES,
    fundamental_types::END_CASH_POSITION,
    fundamental_types::BEGINNING_CASH_POSITION,
    fundamental_types::CHANGESIN_CASH,
    fundamental_types::EFFECT_OF_EXCHANGE_RATE_CHANGES,
    fundamental_types::FREE_CASH_FLOW,
    fundamental_types::CAPITAL_EXPENDITURE_REPORTED,
    fundamental_types::DEPRECIATION_AND_AMORTIZATION,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_type_from_str_round_trips_as_str() {
        for statement in [
            StatementType::Income,
            StatementType::Balance,
            StatementType::CashFlow,
        ] {
            assert_eq!(statement.as_str().parse(), Ok(statement));
        }
        assert_eq!("INCOME-STATEMENT".parse(), Ok(StatementType::Income));
        assert_eq!("balance-sheet".parse(), Ok(StatementType::Balance));
        assert_eq!("cash".parse(), Ok(StatementType::CashFlow));
        assert_eq!("bogus".parse::<StatementType>(), Err(()));
    }
}
