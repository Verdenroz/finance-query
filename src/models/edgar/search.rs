//! EDGAR Full-Text Search (EFTS) models.
//!
//! Models for results from the SEC EDGAR full-text search API at
//! `https://efts.sec.gov/LATEST/search-index`.

use serde::{Deserialize, Serialize};

/// Full-text search results from SEC EDGAR.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarSearchResults {
    /// The search query that was executed (Elasticsearch query DSL, stored as raw JSON)
    #[serde(default)]
    pub query: Option<serde_json::Value>,

    /// Nested hits container
    #[serde(default)]
    pub hits: Option<EdgarSearchHitsContainer>,
}

/// Container for search hits with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarSearchHitsContainer {
    /// Total number of matching results
    #[serde(default)]
    pub total: Option<EdgarSearchTotal>,

    /// Maximum score
    #[serde(default)]
    pub max_score: Option<f64>,

    /// Search result hits
    #[serde(default)]
    pub hits: Vec<EdgarSearchHit>,
}

/// Total count information for search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarSearchTotal {
    /// Total number of matching documents
    #[serde(default)]
    pub value: Option<u64>,

    /// Relation to the actual total (e.g., "eq" for exact, "gte" for 10000+)
    #[serde(default)]
    pub relation: Option<String>,
}

/// A single search result hit from EDGAR full-text search (Elasticsearch format).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarSearchHit {
    /// Elasticsearch index name
    #[serde(default)]
    pub _index: Option<String>,

    /// Hit ID
    #[serde(default)]
    pub _id: Option<String>,

    /// Relevance score
    #[serde(default)]
    pub _score: Option<f64>,

    /// The actual filing data
    #[serde(default)]
    pub _source: Option<EdgarSearchSource>,
}

/// Source data for a search hit containing the actual filing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EdgarSearchSource {
    /// CIK numbers (as strings)
    #[serde(default)]
    pub ciks: Vec<String>,

    /// Filing date (YYYY-MM-DD)
    #[serde(default)]
    pub file_date: Option<String>,

    /// Form type (e.g., "10-K", "10-Q", "8-K")
    #[serde(default)]
    pub form: Option<String>,

    /// Accession number (EDGAR document ID)
    #[serde(default)]
    pub adsh: Option<String>,

    /// Display names (company name with ticker)
    #[serde(default)]
    pub display_names: Vec<String>,

    /// Period ending date
    #[serde(default)]
    pub period_ending: Option<String>,

    /// Root form types
    #[serde(default)]
    pub root_forms: Vec<String>,

    /// Standard Industrial Classification codes
    #[serde(default)]
    pub sics: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_search_results() {
        let json = r#"{
            "query": {"query": {"match": {"doc_text": "test"}}},
            "hits": {
                "total": {
                    "value": 10000,
                    "relation": "gte"
                },
                "max_score": 1.5,
                "hits": [
                    {
                        "_index": "edgar-filings",
                        "_id": "1",
                        "_score": 1.5,
                        "_source": {
                            "ciks": ["320193"],
                            "file_date": "2024-11-01",
                            "form": "10-K",
                            "adsh": "0000320193-24-000123",
                            "display_names": ["Apple Inc. (AAPL)"],
                            "period_ending": "2024-09-28",
                            "root_forms": ["10-K"],
                            "sics": ["3571"]
                        }
                    }
                ]
            }
        }"#;

        let results: EdgarSearchResults = serde_json::from_str(json).unwrap();
        assert!(results.query.is_some());
        let hits_container = results.hits.as_ref().unwrap();
        assert_eq!(hits_container.total.as_ref().unwrap().value, Some(10000));
        assert_eq!(hits_container.hits.len(), 1);

        let first_hit = &hits_container.hits[0];
        let source = first_hit._source.as_ref().unwrap();
        assert_eq!(source.ciks, vec!["320193"]);
        assert_eq!(source.form.as_deref(), Some("10-K"));
        assert!(!source.display_names.is_empty());
    }
}
