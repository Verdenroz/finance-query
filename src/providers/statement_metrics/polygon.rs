use super::{Metrics, Table, rename};

/// Polygon reports no plain net income. Yahoo's `net_income` is net income attributable to
/// the parent before preferred dividends: consolidated net income minus the non-controlling
/// interest, which Polygon leaves null for companies without one. Reads the snake_case
/// passthrough keys `consolidated_net_income_loss` and `noncontrolling_interest`.
pub(super) fn derive_net_income(metrics: &Metrics) -> Metrics {
    let Some(consolidated) = metrics.get("consolidated_net_income_loss") else {
        return Metrics::new();
    };
    let noncontrolling = metrics.get("noncontrolling_interest");
    let net_income = consolidated
        .iter()
        .map(|(period, total)| {
            let minority = noncontrolling
                .and_then(|n| n.get(period))
                .copied()
                .unwrap_or(0.0);
            (period.clone(), total - minority)
        })
        .collect();
    Metrics::from([("net_income".to_string(), net_income)])
}

pub(super) const INCOME: Table = &[
    rename("revenue", "total_revenue"),
    rename("cost_of_revenue", "cost_of_revenue"),
    rename("gross_profit", "gross_profit"),
    rename("research_development", "research_and_development"),
    rename(
        "selling_general_administrative",
        "selling_general_and_administration",
    ),
    rename(
        "depreciation_depletion_amortization",
        "depreciation_and_amortization",
    ),
    rename("total_operating_expenses", "operating_expense"),
    rename("operating_income", "operating_income"),
    rename("interest_expense", "interest_expense"),
    rename("interest_income", "interest_income"),
    rename("other_income_expense", "other_income_expense"),
    rename("income_before_income_taxes", "pretax_income"),
    rename("income_taxes", "tax_provision"),
    rename(
        "net_income_loss_attributable_common_shareholders",
        "net_income_common_stockholders",
    ),
    rename("basic_earnings_per_share", "basic_eps"),
    rename("diluted_earnings_per_share", "diluted_eps"),
    rename("basic_shares_outstanding", "basic_average_shares"),
    rename("diluted_shares_outstanding", "diluted_average_shares"),
    rename("ebitda", "ebitda"),
];

pub(super) const BALANCE: Table = &[
    rename("cash_and_equivalents", "cash_and_cash_equivalents"),
    rename("receivables", "receivables"),
    rename("inventories", "inventory"),
    rename("other_current_assets", "other_current_assets"),
    rename("total_current_assets", "current_assets"),
    rename("property_plant_equipment_net", "net_ppe"),
    rename("goodwill", "goodwill"),
    rename("intangible_assets_net", "other_intangible_assets"),
    rename("total_assets", "total_assets"),
    rename("accounts_payable", "accounts_payable"),
    rename("debt_current", "current_debt"),
    rename("deferred_revenue_current", "current_deferred_revenue"),
    rename("total_current_liabilities", "current_liabilities"),
    rename(
        "long_term_debt_and_capital_lease_obligations",
        "long_term_debt_and_capital_lease_obligation",
    ),
    rename(
        "other_noncurrent_liabilities",
        "other_non_current_liabilities",
    ),
    rename(
        "total_liabilities",
        "total_liabilities_net_minority_interest",
    ),
    rename("common_stock", "common_stock"),
    rename("additional_paid_in_capital", "additional_paid_in_capital"),
    rename("retained_earnings_deficit", "retained_earnings"),
    rename("treasury_stock", "treasury_stock"),
    rename("total_equity_attributable_to_parent", "stockholders_equity"),
    rename("total_equity", "total_equity_gross_minority_interest"),
];

pub(super) const CASHFLOW: Table = &[
    rename("net_income", "net_income_from_continuing_operations"),
    rename(
        "depreciation_depletion_and_amortization",
        "depreciation_and_amortization",
    ),
    rename(
        "cash_from_operating_activities_continuing_operations",
        "cash_flow_from_continuing_operating_activities",
    ),
    rename("net_cash_from_operating_activities", "operating_cash_flow"),
    rename(
        "purchase_of_property_plant_and_equipment",
        "purchase_of_ppe",
    ),
    rename("sale_of_property_plant_and_equipment", "sale_of_ppe"),
    rename("other_investing_activities", "net_other_investing_changes"),
    rename(
        "net_cash_from_investing_activities_continuing_operations",
        "cash_flow_from_continuing_investing_activities",
    ),
    rename("net_cash_from_investing_activities", "investing_cash_flow"),
    rename("dividends", "cash_dividends_paid"),
    rename(
        "short_term_debt_issuances_repayments",
        "net_short_term_debt_issuance",
    ),
    rename(
        "long_term_debt_issuances_repayments",
        "net_long_term_debt_issuance",
    ),
    rename("other_financing_activities", "net_other_financing_charges"),
    rename(
        "net_cash_from_financing_activities_continuing_operations",
        "cash_flow_from_continuing_financing_activities",
    ),
    rename("net_cash_from_financing_activities", "financing_cash_flow"),
    rename(
        "effect_of_currency_exchange_rate",
        "effect_of_exchange_rate_changes",
    ),
    rename("change_in_cash_and_equivalents", "changes_in_cash"),
];
