//! Employee headcount extracted from EDGAR's XBRL company facts.
//!
//! `dei:EntityNumberOfEmployees` is a voluntary cover-page tag that most
//! filers, including large caps like Apple, don't use — absence here is a
//! real "not reported" result, not a parsing failure.

use crate::adapters::edgar::company_facts_for_symbol;
use crate::error::{FinanceError, Result};
use crate::models::corporate::governance::EmployeeCount;
use crate::models::filings::{CompanyFacts, FactUnit};

fn to_employee_count(
    unit: &FactUnit,
    symbol: &str,
    cik: Option<u64>,
    company_name: &Option<String>,
) -> EmployeeCount {
    EmployeeCount {
        symbol: Some(symbol.to_string()),
        cik: cik.map(|c| c.to_string()),
        company_name: company_name.clone(),
        employee_count: unit.val.map(|v| v as i64),
        period_of_report: unit.end.clone(),
        form_type: unit.form.clone(),
        filing_date: unit.filed.clone(),
        source: match (cik, &unit.accn) {
            (Some(cik), Some(accn)) => Some(format!(
                "https://www.sec.gov/Archives/edgar/data/{cik}/{}/",
                accn.replace('-', "")
            )),
            _ => None,
        },
    }
}

fn dei_employee_counts(facts: &CompanyFacts, symbol: &str) -> Result<Vec<EmployeeCount>> {
    let units = facts
        .dei()
        .and_then(|dei| dei.0.get("EntityNumberOfEmployees"))
        .and_then(|concept| concept.units.get("pure"))
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "facts.dei.EntityNumberOfEmployees".to_string(),
            context: format!("{symbol} does not report this tag"),
        })?;

    let mut out: Vec<EmployeeCount> = units
        .iter()
        .map(|unit| to_employee_count(unit, symbol, facts.cik, &facts.entity_name))
        .collect();
    out.sort_by(|a, b| b.period_of_report.cmp(&a.period_of_report));
    Ok(out)
}

/// Fetch canonical employee headcount history, most recent period first.
pub async fn fetch_employee_count_response(symbol: &str) -> Result<Vec<EmployeeCount>> {
    let facts = company_facts_for_symbol(symbol).await?;
    dei_employee_counts(&facts, symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts_with_employees(entries: serde_json::Value) -> CompanyFacts {
        serde_json::from_value(serde_json::json!({
            "cik": 320193,
            "entityName": "Apple Inc.",
            "facts": {
                "dei": {
                    "EntityNumberOfEmployees": {
                        "label": "Entity Number of Employees",
                        "units": { "pure": entries }
                    }
                }
            }
        }))
        .unwrap()
    }

    #[test]
    fn maps_filing_metadata_and_builds_source_url() {
        let facts = facts_with_employees(serde_json::json!([{
            "end": "2023-09-30",
            "val": 161_000,
            "accn": "0000320193-23-000106",
            "form": "10-K",
            "filed": "2023-11-03"
        }]));

        let out = dei_employee_counts(&facts, "AAPL").unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].employee_count, Some(161_000));
        assert_eq!(out[0].period_of_report.as_deref(), Some("2023-09-30"));
        assert_eq!(out[0].form_type.as_deref(), Some("10-K"));
        assert_eq!(out[0].cik.as_deref(), Some("320193"));
        assert_eq!(
            out[0].source.as_deref(),
            Some("https://www.sec.gov/Archives/edgar/data/320193/000032019323000106/")
        );
    }

    #[test]
    fn sorts_most_recent_period_first() {
        let facts = facts_with_employees(serde_json::json!([
            { "end": "2021-09-30", "val": 100_000, "form": "10-K" },
            { "end": "2023-09-30", "val": 161_000, "form": "10-K" },
            { "end": "2022-09-30", "val": 150_000, "form": "10-K" }
        ]));

        let out = dei_employee_counts(&facts, "AAPL").unwrap();
        assert_eq!(
            out.iter()
                .map(|e| e.period_of_report.clone())
                .collect::<Vec<_>>(),
            vec![
                Some("2023-09-30".to_string()),
                Some("2022-09-30".to_string()),
                Some("2021-09-30".to_string())
            ]
        );
    }

    #[test]
    fn missing_tag_errors_instead_of_returning_empty() {
        let facts: CompanyFacts = serde_json::from_value(serde_json::json!({
            "cik": 320193,
            "entityName": "Apple Inc.",
            "facts": { "us-gaap": {} }
        }))
        .unwrap();

        assert!(dei_employee_counts(&facts, "AAPL").is_err());
    }

    /// HII tags `dei:EntityNumberOfEmployees`; most filers (including AAPL)
    /// don't, so this is one of the few reliable live fixtures.
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn fetches_hii_employee_count_from_live_edgar() {
        let _ = crate::adapters::edgar::init("test@example.com");
        let out = fetch_employee_count_response("HII").await.unwrap();
        assert!(!out.is_empty());
        assert!(out[0].employee_count.unwrap() > 0);
    }
}
