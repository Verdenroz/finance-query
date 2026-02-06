//! EDGAR Submissions API models.
//!
//! Models for the filing history and company metadata from
//! `https://data.sec.gov/submissions/CIK{padded}.json`.

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize empty strings as None
fn deserialize_empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.filter(|s| !s.is_empty()))
}

/// Deserialize Vec<String>, filtering out empty strings
fn deserialize_vec_string_filter_empty<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<String>::deserialize(deserializer)?;
    Ok(vec.into_iter().filter(|s| !s.is_empty()).collect())
}

/// Full submissions response for a company from SEC EDGAR.
///
/// Contains company metadata and filing history. The `filings` field holds
/// the most recent ~1000 filings inline, with links to older history files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EdgarSubmissions {
    /// CIK number (as string)
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub cik: Option<String>,

    /// Company name
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub name: Option<String>,

    /// Entity type (e.g., "operating")
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub entity_type: Option<String>,

    /// Standard Industrial Classification code
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub sic: Option<String>,

    /// SIC description
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub sic_description: Option<String>,

    /// Ticker symbols associated with this entity
    #[serde(default)]
    pub tickers: Vec<String>,

    /// Stock exchanges
    #[serde(default)]
    pub exchanges: Vec<String>,

    /// State of incorporation
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub state_of_incorporation: Option<String>,

    /// Fiscal year end (MMDD format, e.g., "0930" for September 30)
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub fiscal_year_end: Option<String>,

    /// Employer Identification Number
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub ein: Option<String>,

    /// Company website (often empty in SEC data)
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub website: Option<String>,

    /// Filer category (e.g., "Large accelerated filer")
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub category: Option<String>,

    /// Whether insider transaction data exists for this entity as owner (0 or 1)
    #[serde(default)]
    pub insider_transaction_for_owner_exists: Option<u8>,

    /// Whether insider transaction data exists for this entity as issuer (0 or 1)
    #[serde(default)]
    pub insider_transaction_for_issuer_exists: Option<u8>,

    /// Filing history
    #[serde(default)]
    pub filings: Option<EdgarFilings>,
}

/// Container for recent filings and links to older filing history files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EdgarFilings {
    /// Recent filings (up to ~1000, inline in the response)
    #[serde(default)]
    pub recent: Option<EdgarFilingRecent>,

    /// Links to additional filing history JSON files
    #[serde(default)]
    pub files: Vec<EdgarFilingFile>,
}

/// Reference to an additional filing history file for older filings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EdgarFilingFile {
    /// Filename of the additional filings JSON (relative to submissions URL)
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub name: Option<String>,

    /// Number of filings in this file
    #[serde(default)]
    pub filing_count: Option<u32>,

    /// Earliest filing date in this file
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub filing_from: Option<String>,

    /// Latest filing date in this file
    #[serde(default, deserialize_with = "deserialize_empty_string_as_none")]
    pub filing_to: Option<String>,
}

/// Recent filings data stored as parallel arrays.
///
/// EDGAR returns filing data as parallel arrays (each field is a `Vec` of the same length).
/// Use [`to_filings()`](EdgarFilingRecent::to_filings) to convert to a `Vec<EdgarFiling>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EdgarFilingRecent {
    /// Accession numbers (unique filing identifiers)
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub accession_number: Vec<String>,

    /// Filing dates (YYYY-MM-DD)
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub filing_date: Vec<String>,

    /// Report dates (YYYY-MM-DD, may be empty for some form types)
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub report_date: Vec<String>,

    /// Acceptance date-times
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub acceptance_date_time: Vec<String>,

    /// Form types (10-K, 10-Q, 8-K, etc.)
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub form: Vec<String>,

    /// Filing sizes in bytes
    #[serde(default)]
    pub size: Vec<u64>,

    /// Whether the filing is XBRL
    #[serde(default, rename = "isXBRL")]
    pub is_xbrl: Vec<u8>,

    /// Whether the filing is Inline XBRL
    #[serde(default, rename = "isInlineXBRL")]
    pub is_inline_xbrl: Vec<u8>,

    /// Primary document filenames
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub primary_document: Vec<String>,

    /// Primary document descriptions
    #[serde(default, deserialize_with = "deserialize_vec_string_filter_empty")]
    pub primary_doc_description: Vec<String>,
}

impl EdgarFilingRecent {
    /// Convert parallel arrays into a vector of individual filings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use finance_query::EdgarSubmissions;
    /// # fn example(submissions: EdgarSubmissions) {
    /// if let Some(filings) = &submissions.filings {
    ///     if let Some(recent) = &filings.recent {
    ///         for filing in recent.to_filings() {
    ///             println!("{}: {} ({})", filing.filing_date, filing.form, filing.primary_doc_description);
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn to_filings(&self) -> Vec<EdgarFiling> {
        let len = self.accession_number.len();
        (0..len)
            .map(|i| EdgarFiling {
                accession_number: self.accession_number.get(i).cloned().unwrap_or_default(),
                filing_date: self.filing_date.get(i).cloned().unwrap_or_default(),
                report_date: self.report_date.get(i).cloned().unwrap_or_default(),
                acceptance_date_time: self
                    .acceptance_date_time
                    .get(i)
                    .cloned()
                    .unwrap_or_default(),
                form: self.form.get(i).cloned().unwrap_or_default(),
                size: self.size.get(i).copied().unwrap_or(0),
                is_xbrl: self.is_xbrl.get(i).copied().unwrap_or(0) != 0,
                is_inline_xbrl: self.is_inline_xbrl.get(i).copied().unwrap_or(0) != 0,
                primary_document: self.primary_document.get(i).cloned().unwrap_or_default(),
                primary_doc_description: self
                    .primary_doc_description
                    .get(i)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect()
    }
}

/// A single SEC filing with metadata.
///
/// Derived from the parallel arrays in [`EdgarFilingRecent`] via
/// [`to_filings()`](EdgarFilingRecent::to_filings).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarFiling {
    /// Accession number (unique filing identifier, e.g., "0000320193-24-000123")
    pub accession_number: String,
    /// Filing date (YYYY-MM-DD)
    pub filing_date: String,
    /// Report date (YYYY-MM-DD)
    pub report_date: String,
    /// Acceptance date-time
    pub acceptance_date_time: String,
    /// Form type (e.g., "10-K", "10-Q", "8-K")
    pub form: String,
    /// Filing size in bytes
    pub size: u64,
    /// Whether the filing contains XBRL data
    pub is_xbrl: bool,
    /// Whether the filing contains Inline XBRL data
    pub is_inline_xbrl: bool,
    /// Primary document filename
    pub primary_document: String,
    /// Primary document description
    pub primary_doc_description: String,
}

impl EdgarFiling {
    /// Get the URL to view this filing on SEC EDGAR.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use finance_query::{EdgarClientBuilder, Ticker};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let edgar = EdgarClientBuilder::new("user@example.com").build()?;
    /// let ticker = Ticker::builder("AAPL").edgar(std::sync::Arc::new(edgar)).build().await?;
    /// let submissions = ticker.edgar_submissions().await?;
    ///
    /// if let Some(filings) = &submissions.filings {
    ///     if let Some(recent) = &filings.recent {
    ///         for filing in recent.to_filings() {
    ///             let url = filing.edgar_url();
    ///             println!("Filing URL: {}", url);
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn edgar_url(&self) -> String {
        let accession_no_dashes = self.accession_number.replace('-', "");
        format!(
            "https://www.sec.gov/Archives/edgar/data/{}/{}",
            accession_no_dashes, self.primary_document
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_submissions() {
        let json = r#"{
            "cik": "0000320193",
            "entityType": "operating",
            "sic": "3571",
            "sicDescription": "Electronic Computers",
            "name": "Apple Inc.",
            "tickers": ["AAPL"],
            "exchanges": ["Nasdaq"],
            "stateOfIncorporation": "CA",
            "fiscalYearEnd": "0930",
            "website": "https://www.apple.com",
            "category": "Large accelerated filer",
            "filings": {
                "recent": {
                    "accessionNumber": ["0000320193-24-000123", "0000320193-24-000100"],
                    "filingDate": ["2024-11-01", "2024-08-02"],
                    "reportDate": ["2024-09-28", "2024-06-29"],
                    "acceptanceDateTime": ["2024-11-01T16:30:00.000Z", "2024-08-02T16:15:00.000Z"],
                    "form": ["10-K", "10-Q"],
                    "size": [15000000, 8000000],
                    "isXBRL": [1, 1],
                    "isInlineXBRL": [1, 1],
                    "primaryDocument": ["aapl-20240928.htm", "aapl-20240629.htm"],
                    "primaryDocDescription": ["10-K", "10-Q"]
                },
                "files": []
            }
        }"#;

        let submissions: EdgarSubmissions = serde_json::from_str(json).unwrap();
        assert_eq!(submissions.name.as_deref(), Some("Apple Inc."));
        assert_eq!(submissions.tickers, vec!["AAPL"]);
        assert_eq!(submissions.sic.as_deref(), Some("3571"));

        let filings = submissions.filings.unwrap();
        let recent = filings.recent.unwrap();
        assert_eq!(recent.accession_number.len(), 2);

        let individual = recent.to_filings();
        assert_eq!(individual.len(), 2);
        assert_eq!(individual[0].form, "10-K");
        assert_eq!(individual[1].form, "10-Q");
        assert!(individual[0].is_xbrl);
    }

    #[test]
    fn test_empty_string_deserialization() {
        let json = r#"{
            "cik": "0000320193",
            "name": "Test Company",
            "website": "",
            "ein": "",
            "tickers": [],
            "exchanges": [],
            "filings": {
                "recent": {
                    "accessionNumber": ["123"],
                    "filingDate": ["2024-01-01"],
                    "reportDate": [""],
                    "acceptanceDateTime": [""],
                    "form": ["4"],
                    "size": [100],
                    "isXBRL": [0],
                    "isInlineXBRL": [0],
                    "primaryDocument": ["doc.xml"],
                    "primaryDocDescription": [""]
                }
            }
        }"#;

        let submissions: EdgarSubmissions = serde_json::from_str(json).unwrap();
        assert_eq!(submissions.name.as_deref(), Some("Test Company"));
        // Empty strings should be None
        assert_eq!(submissions.website, None);
        assert_eq!(submissions.ein, None);

        let filings = submissions.filings.as_ref().unwrap();
        let recent = filings.recent.as_ref().unwrap();
        // Empty strings should be filtered out from Vec<String>
        assert_eq!(recent.accession_number, vec!["123"]);
        assert_eq!(recent.report_date, Vec::<String>::new()); // Empty string filtered out
        assert_eq!(recent.acceptance_date_time, Vec::<String>::new()); // Empty string filtered out
        assert_eq!(recent.primary_doc_description, Vec::<String>::new()); // Empty string filtered out

        // Test round-trip: serialize back to JSON and verify None becomes null
        let serialized = serde_json::to_value(&submissions).unwrap();
        assert_eq!(serialized["website"], serde_json::Value::Null);
        assert_eq!(serialized["ein"], serde_json::Value::Null);
    }

    #[test]
    fn test_edgar_filing_url() {
        let filing = EdgarFiling {
            accession_number: "0000320193-24-000123".to_string(),
            filing_date: "2024-11-01".to_string(),
            report_date: "2024-09-28".to_string(),
            acceptance_date_time: String::new(),
            form: "10-K".to_string(),
            size: 15000000,
            is_xbrl: true,
            is_inline_xbrl: true,
            primary_document: "aapl-20240928.htm".to_string(),
            primary_doc_description: "10-K".to_string(),
        };

        assert_eq!(
            filing.edgar_url(),
            "https://www.sec.gov/Archives/edgar/data/000032019324000123/aapl-20240928.htm"
        );
    }
}
