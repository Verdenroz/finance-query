//! One snake_case metric vocabulary for financial statements, whichever provider served them.
//!
//! Canonical names are Yahoo's field names in snake_case. Each other provider maps its own
//! names onto them through a per-statement table. A metric with no canonical equivalent keeps
//! its own name in snake_case, so no data is dropped; if that name is already taken, by a
//! canonical name or another unmapped key, the mapped value keeps it and the unmapped metric
//! keeps its own spelling. Values follow Yahoo's sign convention (cash outflows negative); a
//! provider that reports them positive marks those entries with `negate`.

#[cfg(feature = "alphavantage")]
mod alphavantage;
#[cfg(feature = "fmp")]
mod fmp;
#[cfg(feature = "polygon")]
mod polygon;

use std::collections::BTreeMap;

use crate::constants::StatementType;
use finance_query_types::Provider;

type Metrics = BTreeMap<String, BTreeMap<String, f64>>;
type Table = &'static [Entry];

struct Entry {
    from: &'static str,
    to: &'static str,
    negate: bool,
}

const fn rename(from: &'static str, to: &'static str) -> Entry {
    Entry {
        from,
        to,
        negate: false,
    }
}

#[cfg(feature = "alphavantage")]
const fn negate(from: &'static str, to: &'static str) -> Entry {
    Entry {
        from,
        to,
        negate: true,
    }
}

const YAHOO_CASHFLOW: Table = &[rename("ChangesinCash", "changes_in_cash")];

/// Rewrites a statement's metric names to the canonical vocabulary.
pub(crate) fn canonicalize(
    provider: Provider,
    statement_type: StatementType,
    metrics: Metrics,
) -> Metrics {
    let table = table(provider, statement_type);
    let derived = derived(provider, statement_type, &metrics);

    let mut out = Metrics::new();
    let mut unmapped = Vec::new();
    for (key, values) in metrics {
        match table.iter().find(|entry| entry.from == key) {
            Some(entry) => {
                let values = if entry.negate {
                    values.into_iter().map(|(date, v)| (date, -v)).collect()
                } else {
                    values
                };
                out.insert(entry.to.to_string(), values);
            }
            None => unmapped.push((key, values)),
        }
    }
    out.extend(derived);
    for (key, values) in unmapped {
        let snake = snake_case(&key);
        let name = if out.contains_key(&snake) { key } else { snake };
        out.entry(name).or_insert(values);
    }
    out
}

fn table(provider: Provider, statement_type: StatementType) -> Table {
    match (provider, statement_type) {
        (Provider::Yahoo, StatementType::CashFlow) => YAHOO_CASHFLOW,
        #[cfg(feature = "fmp")]
        (Provider::Fmp, StatementType::Income) => fmp::INCOME,
        #[cfg(feature = "fmp")]
        (Provider::Fmp, StatementType::Balance) => fmp::BALANCE,
        #[cfg(feature = "fmp")]
        (Provider::Fmp, StatementType::CashFlow) => fmp::CASHFLOW,
        #[cfg(feature = "polygon")]
        (Provider::Polygon, StatementType::Income) => polygon::INCOME,
        #[cfg(feature = "polygon")]
        (Provider::Polygon, StatementType::Balance) => polygon::BALANCE,
        #[cfg(feature = "polygon")]
        (Provider::Polygon, StatementType::CashFlow) => polygon::CASHFLOW,
        #[cfg(feature = "alphavantage")]
        (Provider::AlphaVantage, StatementType::Income) => alphavantage::INCOME,
        #[cfg(feature = "alphavantage")]
        (Provider::AlphaVantage, StatementType::Balance) => alphavantage::BALANCE,
        #[cfg(feature = "alphavantage")]
        (Provider::AlphaVantage, StatementType::CashFlow) => alphavantage::CASHFLOW,
        _ => &[],
    }
}

#[cfg(feature = "polygon")]
fn derived(provider: Provider, statement_type: StatementType, metrics: &Metrics) -> Metrics {
    match (provider, statement_type) {
        (Provider::Polygon, StatementType::Income) => polygon::derive_net_income(metrics),
        _ => Metrics::new(),
    }
}

#[cfg(not(feature = "polygon"))]
fn derived(_: Provider, _: StatementType, _: &Metrics) -> Metrics {
    Metrics::new()
}

/// `TotalRevenue` -> `total_revenue`, `DilutedEPS` -> `diluted_eps`, `totalRevenue` ->
/// `total_revenue`; already snake_case names pass through unchanged.
fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if !c.is_ascii_uppercase() {
            out.push(c);
            continue;
        }
        let prev = i.checked_sub(1).map(|j| chars[j]);
        let next_is_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
        let starts_word = match prev {
            Some(p) if p.is_ascii_lowercase() || p.is_ascii_digit() => true,
            Some(p) if p.is_ascii_uppercase() => next_is_lower,
            _ => false,
        };
        if starts_word && !out.ends_with('_') {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const STATEMENTS: [StatementType; 3] = [
        StatementType::Income,
        StatementType::Balance,
        StatementType::CashFlow,
    ];

    fn metrics(rows: &[(&str, &[(&str, f64)])]) -> Metrics {
        rows.iter()
            .map(|(key, values)| {
                (
                    key.to_string(),
                    values.iter().map(|(d, v)| (d.to_string(), *v)).collect(),
                )
            })
            .collect()
    }

    fn vocabulary(statement_type: StatementType) -> BTreeSet<String> {
        statement_type
            .get_fields()
            .iter()
            .map(|field| match *field {
                "ChangesinCash" => "changes_in_cash".to_string(),
                field => snake_case(field),
            })
            .collect()
    }

    #[test]
    fn snake_case_handles_provider_spellings() {
        for (raw, want) in [
            ("TotalRevenue", "total_revenue"),
            ("totalRevenue", "total_revenue"),
            ("total_revenue", "total_revenue"),
            ("DilutedEPS", "diluted_eps"),
            ("EBITDA", "ebitda"),
            ("NetPPE", "net_ppe"),
            ("EPSBasic", "eps_basic"),
            ("revenue", "revenue"),
        ] {
            assert_eq!(snake_case(raw), want, "{raw}");
        }
    }

    #[test]
    fn vocabulary_is_the_snake_cased_yahoo_field_lists() {
        let all: BTreeSet<String> = STATEMENTS.into_iter().flat_map(vocabulary).collect();
        assert_eq!(all.len(), 122);
        assert!(all.contains("total_revenue") && all.contains("changes_in_cash"));
    }

    #[test]
    fn yahoo_keys_become_snake_case_with_changes_in_cash_override() {
        let out = canonicalize(
            Provider::Yahoo,
            StatementType::CashFlow,
            metrics(&[
                ("OperatingCashFlow", &[("2024-09-30", 1.0)]),
                ("ChangesinCash", &[("2024-09-30", 2.0)]),
                ("SomethingNew", &[("2024-09-30", 3.0)]),
            ]),
        );
        let keys: Vec<_> = out.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["changes_in_cash", "operating_cash_flow", "something_new"]
        );
    }

    #[test]
    fn every_yahoo_field_is_already_canonical() {
        for statement_type in STATEMENTS {
            let raw = statement_type
                .get_fields()
                .iter()
                .map(|f| (f.to_string(), BTreeMap::from([("d".to_string(), 1.0)])))
                .collect();
            let out = canonicalize(Provider::Yahoo, statement_type, raw);
            let keys: BTreeSet<String> = out.into_keys().collect();
            assert_eq!(keys, vocabulary(statement_type), "{statement_type:?}");
        }
    }

    #[test]
    fn unmapped_key_colliding_with_a_canonical_name_keeps_its_spelling() {
        let out = canonicalize(
            Provider::Yahoo,
            StatementType::CashFlow,
            metrics(&[
                ("ChangesInCash", &[("2024-09-30", 9.0)]),
                ("ChangesinCash", &[("2024-09-30", 2.0)]),
            ]),
        );
        assert_eq!(out["changes_in_cash"]["2024-09-30"], 2.0);
        assert_eq!(out["ChangesInCash"]["2024-09-30"], 9.0);
    }

    #[test]
    fn unmapped_keys_colliding_with_each_other_both_survive() {
        let out = canonicalize(
            Provider::Yahoo,
            StatementType::Income,
            metrics(&[
                ("fooBar", &[("2024-09-30", 1.0)]),
                ("FooBar", &[("2024-09-30", 2.0)]),
            ]),
        );
        assert_eq!(out["foo_bar"]["2024-09-30"], 2.0);
        assert_eq!(out["fooBar"]["2024-09-30"], 1.0);
    }

    #[cfg(any(feature = "fmp", feature = "polygon", feature = "alphavantage"))]
    fn assert_table_is_canonical(name: &str, statement_type: StatementType, table: Table) {
        let vocabulary = vocabulary(statement_type);
        let mut sources = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for Entry { from, to, .. } in table {
            assert!(vocabulary.contains(*to), "{name} {statement_type:?}: {to}");
            assert!(sources.insert(*from), "{name} {statement_type:?}: {from}");
            assert!(targets.insert(*to), "{name} {statement_type:?}: {to} twice");
        }
    }

    #[cfg(any(feature = "fmp", feature = "polygon"))]
    fn dto_keys<T: serde::de::DeserializeOwned + serde::Serialize>() -> BTreeSet<String> {
        let blank: T = serde_json::from_value(serde_json::json!({})).unwrap();
        serde_json::to_value(blank)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    #[cfg(any(feature = "fmp", feature = "polygon"))]
    fn assert_dtos_cover_table(provider: Provider, name: &str, dtos: [BTreeSet<String>; 3]) {
        for (statement_type, keys) in STATEMENTS.into_iter().zip(dtos) {
            let table = table(provider, statement_type);
            assert_table_is_canonical(name, statement_type, table);
            for Entry { from, .. } in table {
                assert!(keys.contains(*from), "{statement_type:?}: {from}");
            }
        }
    }

    #[cfg(feature = "fmp")]
    mod fmp {
        use super::*;
        use crate::adapters::fmp::fundamentals::core::{
            BalanceSheetDTO, CashFlowDTO, IncomeStatementDTO,
        };

        #[test]
        fn tables_are_canonical_and_name_real_fields() {
            let dtos = [
                dto_keys::<IncomeStatementDTO>(),
                dto_keys::<BalanceSheetDTO>(),
                dto_keys::<CashFlowDTO>(),
            ];
            assert_dtos_cover_table(Provider::Fmp, "fmp", dtos);
        }

        #[test]
        fn same_key_maps_per_statement() {
            let rows = metrics(&[("accountsReceivables", &[("2024-09-30", 5.0)])]);
            let balance = canonicalize(Provider::Fmp, StatementType::Balance, rows.clone());
            let cashflow = canonicalize(Provider::Fmp, StatementType::CashFlow, rows);
            assert!(balance.contains_key("accounts_receivable"));
            assert!(cashflow.contains_key("changes_in_account_receivables"));
        }

        #[test]
        fn unmapped_twin_passes_through_in_snake_case() {
            let out = canonicalize(
                Provider::Fmp,
                StatementType::CashFlow,
                metrics(&[
                    ("operatingCashFlow", &[("2024-09-30", 7.0)]),
                    (
                        "netCashProvidedByOperatingActivities",
                        &[("2024-09-30", 7.0)],
                    ),
                ]),
            );
            let keys: Vec<_> = out.keys().map(String::as_str).collect();
            assert_eq!(
                keys,
                [
                    "net_cash_provided_by_operating_activities",
                    "operating_cash_flow"
                ]
            );
        }

        #[test]
        fn income_fixture_maps_to_canonical_names() {
            let out = canonicalize(
                Provider::Fmp,
                StatementType::Income,
                metrics(&[
                    ("revenue", &[("2024-09-30", 391.0)]),
                    ("epsDiluted", &[("2024-09-30", 6.1)]),
                    ("bottomLineNetIncome", &[("2024-09-30", 99.0)]),
                ]),
            );
            let keys: Vec<_> = out.keys().map(String::as_str).collect();
            assert_eq!(
                keys,
                ["bottom_line_net_income", "diluted_eps", "total_revenue"]
            );
        }
    }

    #[cfg(feature = "polygon")]
    mod polygon {
        use super::*;
        use crate::adapters::polygon::{BalanceSheetDTO, CashFlowStatementDTO, IncomeStatementDTO};

        #[test]
        fn tables_are_canonical_and_name_real_fields() {
            let dtos = [
                dto_keys::<IncomeStatementDTO>(),
                dto_keys::<BalanceSheetDTO>(),
                dto_keys::<CashFlowStatementDTO>(),
            ];
            assert_dtos_cover_table(Provider::Polygon, "polygon", dtos);
        }

        #[test]
        fn net_income_is_consolidated_less_noncontrolling_interest() {
            let out = canonicalize(
                Provider::Polygon,
                StatementType::Income,
                metrics(&[
                    (
                        "consolidated_net_income_loss",
                        &[
                            ("2024-09-30", 110.0),
                            ("2023-09-30", 90.0),
                            ("2022-09-30", 80.0),
                        ],
                    ),
                    (
                        "noncontrolling_interest",
                        &[("2024-09-30", 10.0), ("2023-09-30", 4.0)],
                    ),
                    (
                        "net_income_loss_attributable_common_shareholders",
                        &[("2024-09-30", 98.0)],
                    ),
                ]),
            );
            assert_eq!(
                out["net_income"],
                BTreeMap::from([
                    ("2024-09-30".to_string(), 100.0),
                    ("2023-09-30".to_string(), 86.0),
                    ("2022-09-30".to_string(), 80.0),
                ])
            );
            assert_eq!(out["net_income_common_stockholders"]["2024-09-30"], 98.0);
            assert!(out.contains_key("consolidated_net_income_loss"));
            assert!(out.contains_key("noncontrolling_interest"));
        }

        #[test]
        fn net_income_without_noncontrolling_interest_is_consolidated() {
            let out = canonicalize(
                Provider::Polygon,
                StatementType::Income,
                metrics(&[("consolidated_net_income_loss", &[("2024-09-30", 93.7)])]),
            );
            assert_eq!(out["net_income"]["2024-09-30"], 93.7);
        }

        #[test]
        fn net_income_is_omitted_without_consolidated() {
            let out = canonicalize(
                Provider::Polygon,
                StatementType::Income,
                metrics(&[("noncontrolling_interest", &[("2024-09-30", 10.0)])]),
            );
            assert!(!out.contains_key("net_income"));
        }
    }

    #[cfg(feature = "alphavantage")]
    mod alphavantage {
        use super::*;

        #[test]
        fn tables_are_canonical() {
            for statement_type in STATEMENTS {
                let table = table(Provider::AlphaVantage, statement_type);
                assert_table_is_canonical("alphavantage", statement_type, table);
            }
        }

        #[test]
        fn positive_outflows_become_negative() {
            let out = canonicalize(
                Provider::AlphaVantage,
                StatementType::CashFlow,
                metrics(&[
                    ("capitalExpenditures", &[("2024-09-30", 9447.0)]),
                    ("dividendPayout", &[("2024-09-30", 15234.0)]),
                    ("operatingCashflow", &[("2024-09-30", 118.0)]),
                ]),
            );
            assert_eq!(out["capital_expenditure"]["2024-09-30"], -9447.0);
            assert_eq!(out["cash_dividends_paid"]["2024-09-30"], -15234.0);
            assert_eq!(out["operating_cash_flow"]["2024-09-30"], 118.0);
        }

        #[test]
        fn cashflow_fixture_maps_to_canonical_names() {
            let out = canonicalize(
                Provider::AlphaVantage,
                StatementType::CashFlow,
                metrics(&[
                    ("operatingCashflow", &[("2024-09-30", 118.0)]),
                    ("netIncome", &[("2024-09-30", 93.0)]),
                    ("profitLoss", &[("2024-09-30", 94.0)]),
                ]),
            );
            let keys: Vec<_> = out.keys().map(String::as_str).collect();
            assert_eq!(
                keys,
                [
                    "net_income_from_continuing_operations",
                    "operating_cash_flow",
                    "profit_loss"
                ]
            );
        }
    }
}
