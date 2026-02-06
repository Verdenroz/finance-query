//! EDGAR Company Facts (XBRL) models.
//!
//! Models for structured XBRL financial data from
//! `https://data.sec.gov/api/xbrl/companyfacts/CIK{padded}.json`.
//!
//! This data includes historical financial statement values (revenue, assets,
//! liabilities, etc.) extracted from 10-K and 10-Q filings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete company facts response containing all XBRL financial data.
///
/// Facts are organized by taxonomy (e.g., `us-gaap`, `ifrs-full`, `dei`).
/// Use the convenience methods to access common taxonomies.
///
/// # Example
///
/// ```no_run
/// # use finance_query::CompanyFacts;
/// # fn example(facts: CompanyFacts) {
/// // Get US-GAAP revenue data
/// if let Some(revenue) = facts.get_us_gaap_fact("Revenue") {
///     for (unit, values) in &revenue.units {
///         println!("Unit: {}, data points: {}", unit, values.len());
///     }
/// }
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyFacts {
    /// CIK number
    #[serde(default)]
    pub cik: Option<u64>,

    /// Company name
    #[serde(default, rename = "entityName")]
    pub entity_name: Option<String>,

    /// Facts organized by taxonomy (e.g., "us-gaap", "ifrs-full", "dei")
    #[serde(default)]
    pub facts: HashMap<String, FactsByTaxonomy>,
}

impl CompanyFacts {
    /// Get US-GAAP facts (most common for US-listed companies).
    pub fn us_gaap(&self) -> Option<&FactsByTaxonomy> {
        self.facts.get("us-gaap")
    }

    /// Get a specific fact concept from the US-GAAP taxonomy.
    ///
    /// Common concepts: `"Revenue"`, `"Assets"`, `"Liabilities"`,
    /// `"NetIncomeLoss"`, `"EarningsPerShareBasic"`, `"StockholdersEquity"`.
    pub fn get_us_gaap_fact(&self, concept: &str) -> Option<&FactConcept> {
        self.us_gaap().and_then(|gaap| gaap.0.get(concept))
    }

    /// Get IFRS facts (for companies reporting under International Financial Reporting Standards).
    pub fn ifrs(&self) -> Option<&FactsByTaxonomy> {
        self.facts.get("ifrs-full")
    }

    /// Get DEI (Document and Entity Information) facts.
    pub fn dei(&self) -> Option<&FactsByTaxonomy> {
        self.facts.get("dei")
    }
}

/// Facts within a single taxonomy (e.g., "us-gaap").
///
/// Maps concept names (e.g., "Revenue", "Assets") to their [`FactConcept`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FactsByTaxonomy(pub HashMap<String, FactConcept>);

/// A single XBRL concept (e.g., "Revenue") with all reported values.
///
/// Values are organized by unit of measure (e.g., "USD", "shares", "pure").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FactConcept {
    /// Human-readable label
    #[serde(default)]
    pub label: Option<String>,

    /// Description of this concept
    #[serde(default)]
    pub description: Option<String>,

    /// Values organized by unit type (e.g., "USD" -> vec of data points)
    #[serde(default)]
    pub units: HashMap<String, Vec<FactUnit>>,
}

/// A single data point for an XBRL fact.
///
/// Represents one reported value from a specific filing and period.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FactUnit {
    /// Start date of the reporting period (for duration facts, e.g., revenue)
    #[serde(default)]
    pub start: Option<String>,

    /// End date of the period (for duration facts) or instant date (for point-in-time facts)
    #[serde(default)]
    pub end: Option<String>,

    /// The reported value
    #[serde(default)]
    pub val: Option<f64>,

    /// Accession number of the filing that reported this value
    #[serde(default)]
    pub accn: Option<String>,

    /// Fiscal year
    #[serde(default)]
    pub fy: Option<i32>,

    /// Fiscal period (FY, Q1, Q2, Q3, Q4)
    #[serde(default)]
    pub fp: Option<String>,

    /// Form type (10-K, 10-Q, etc.)
    #[serde(default)]
    pub form: Option<String>,

    /// Date the filing was filed
    #[serde(default)]
    pub filed: Option<String>,

    /// Frame identifier (e.g., "CY2023Q4I")
    #[serde(default)]
    pub frame: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_company_facts() {
        let json = r#"{
            "cik": 320193,
            "entityName": "Apple Inc.",
            "facts": {
                "us-gaap": {
                    "Revenue": {
                        "label": "Revenue",
                        "description": "Amount of revenue recognized.",
                        "units": {
                            "USD": [
                                {
                                    "start": "2023-10-01",
                                    "end": "2024-09-28",
                                    "val": 391035000000.0,
                                    "accn": "0000320193-24-000123",
                                    "fy": 2024,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2024-11-01",
                                    "frame": "CY2024"
                                },
                                {
                                    "start": "2022-09-25",
                                    "end": "2023-09-30",
                                    "val": 383285000000.0,
                                    "accn": "0000320193-23-000106",
                                    "fy": 2023,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2023-11-03"
                                }
                            ]
                        }
                    },
                    "Assets": {
                        "label": "Assets",
                        "description": "Sum of the carrying amounts.",
                        "units": {
                            "USD": [
                                {
                                    "end": "2024-09-28",
                                    "val": 364980000000.0,
                                    "accn": "0000320193-24-000123",
                                    "fy": 2024,
                                    "fp": "FY",
                                    "form": "10-K",
                                    "filed": "2024-11-01"
                                }
                            ]
                        }
                    }
                }
            }
        }"#;

        let facts: CompanyFacts = serde_json::from_str(json).unwrap();
        assert_eq!(facts.cik, Some(320193));
        assert_eq!(facts.entity_name.as_deref(), Some("Apple Inc."));

        // US-GAAP access
        let gaap = facts.us_gaap().unwrap();
        assert!(gaap.0.contains_key("Revenue"));
        assert!(gaap.0.contains_key("Assets"));

        // Convenience method
        let revenue = facts.get_us_gaap_fact("Revenue").unwrap();
        assert_eq!(revenue.label.as_deref(), Some("Revenue"));
        let usd_values = revenue.units.get("USD").unwrap();
        assert_eq!(usd_values.len(), 2);
        assert_eq!(usd_values[0].val, Some(391035000000.0));
        assert_eq!(usd_values[0].fy, Some(2024));
        assert_eq!(usd_values[0].fp.as_deref(), Some("FY"));
    }
}
