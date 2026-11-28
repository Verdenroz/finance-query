use std::collections::BTreeMap;

use crate::constants::StatementType;

pub(super) fn table(statement: StatementType) -> &'static [(&'static str, &'static str)] {
    match statement {
        StatementType::Income => INCOME,
        StatementType::Balance => BALANCE,
        StatementType::CashFlow => CASHFLOW,
    }
}

/// Polygon reports no plain net income. Yahoo's `net_income` is net income attributable to
/// the parent before preferred dividends: consolidated net income minus the non-controlling
/// interest, which Polygon leaves null for companies without one.
pub(super) fn derive_net_income(
    metrics: &BTreeMap<String, BTreeMap<String, f64>>,
) -> Option<(String, BTreeMap<String, f64>)> {
    let consolidated = metrics.get("consolidated_net_income_loss")?;
    let noncontrolling = metrics.get("noncontrolling_interest");
    let net_income = consolidated
        .iter()
        .map(|(period, total)| {
            let minority = noncontrolling.and_then(|nci| nci.get(period)).copied();
            (period.clone(), total - minority.unwrap_or(0.0))
        })
        .collect();
    Some(("net_income".to_string(), net_income))
}

const INCOME: &[(&str, &str)] = &[
    ("revenue", "total_revenue"),
    ("cost_of_revenue", "cost_of_revenue"),
    ("gross_profit", "gross_profit"),
    ("research_development", "research_and_development"),
    (
        "selling_general_administrative",
        "selling_general_and_administration",
    ),
    (
        "depreciation_depletion_amortization",
        "depreciation_and_amortization",
    ),
    ("total_operating_expenses", "operating_expense"),
    ("operating_income", "operating_income"),
    ("interest_expense", "interest_expense"),
    ("interest_income", "interest_income"),
    ("other_income_expense", "other_income_expense"),
    ("income_before_income_taxes", "pretax_income"),
    ("income_taxes", "tax_provision"),
    (
        "net_income_loss_attributable_common_shareholders",
        "net_income_common_stockholders",
    ),
    ("basic_earnings_per_share", "basic_eps"),
    ("diluted_earnings_per_share", "diluted_eps"),
    ("basic_shares_outstanding", "basic_average_shares"),
    ("diluted_shares_outstanding", "diluted_average_shares"),
    ("ebitda", "ebitda"),
];

const BALANCE: &[(&str, &str)] = &[
    ("cash_and_equivalents", "cash_and_cash_equivalents"),
    ("receivables", "receivables"),
    ("inventories", "inventory"),
    ("other_current_assets", "other_current_assets"),
    ("total_current_assets", "current_assets"),
    ("property_plant_equipment_net", "net_ppe"),
    ("goodwill", "goodwill"),
    ("intangible_assets_net", "other_intangible_assets"),
    ("total_assets", "total_assets"),
    ("accounts_payable", "accounts_payable"),
    ("debt_current", "current_debt"),
    ("deferred_revenue_current", "current_deferred_revenue"),
    ("total_current_liabilities", "current_liabilities"),
    (
        "long_term_debt_and_capital_lease_obligations",
        "long_term_debt_and_capital_lease_obligation",
    ),
    (
        "other_noncurrent_liabilities",
        "other_non_current_liabilities",
    ),
    (
        "total_liabilities",
        "total_liabilities_net_minority_interest",
    ),
    ("common_stock", "common_stock"),
    ("additional_paid_in_capital", "additional_paid_in_capital"),
    ("retained_earnings_deficit", "retained_earnings"),
    ("treasury_stock", "treasury_stock"),
    ("total_equity_attributable_to_parent", "stockholders_equity"),
    ("total_equity", "total_equity_gross_minority_interest"),
];

const CASHFLOW: &[(&str, &str)] = &[
    ("net_income", "net_income_from_continuing_operations"),
    (
        "depreciation_depletion_and_amortization",
        "depreciation_and_amortization",
    ),
    (
        "cash_from_operating_activities_continuing_operations",
        "cash_flow_from_continuing_operating_activities",
    ),
    ("net_cash_from_operating_activities", "operating_cash_flow"),
    (
        "purchase_of_property_plant_and_equipment",
        "purchase_of_ppe",
    ),
    ("sale_of_property_plant_and_equipment", "sale_of_ppe"),
    ("other_investing_activities", "net_other_investing_changes"),
    (
        "net_cash_from_investing_activities_continuing_operations",
        "cash_flow_from_continuing_investing_activities",
    ),
    ("net_cash_from_investing_activities", "investing_cash_flow"),
    ("dividends", "cash_dividends_paid"),
    (
        "short_term_debt_issuances_repayments",
        "net_short_term_debt_issuance",
    ),
    (
        "long_term_debt_issuances_repayments",
        "net_long_term_debt_issuance",
    ),
    ("other_financing_activities", "net_other_financing_charges"),
    (
        "net_cash_from_financing_activities_continuing_operations",
        "cash_flow_from_continuing_financing_activities",
    ),
    ("net_cash_from_financing_activities", "financing_cash_flow"),
    (
        "effect_of_currency_exchange_rate",
        "effect_of_exchange_rate_changes",
    ),
    ("change_in_cash_and_equivalents", "changes_in_cash"),
];
