//! GraphQL types for EDGAR company facts (XBRL), submissions, and search.

use async_graphql::SimpleObject;
use serde::Deserialize;

// ── Company Facts (XBRL) ───────────────────────────────────────────────────

/// A single XBRL fact concept with its data points.
#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFactConcept {
    pub concept: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub taxonomy: String,
    pub unit: String,
    pub data_points: Vec<GqlFactDataPoint>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlFactDataPoint {
    pub start: Option<String>,
    pub end: Option<String>,
    pub val: Option<f64>,
    pub accn: Option<String>,
    pub fy: Option<i32>,
    pub fp: Option<String>,
    pub form: Option<String>,
    pub filed: Option<String>,
    pub frame: Option<String>,
}

// ── Submissions ────────────────────────────────────────────────────────────

#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEdgarSubmissions {
    pub cik: Option<String>,
    pub name: Option<String>,
    pub tickers: Vec<String>,
    pub exchanges: Vec<String>,
    pub sic: Option<String>,
    pub sic_description: Option<String>,
    pub fiscal_year_end: Option<String>,
    pub category: Option<String>,
    pub filings: Vec<GqlEdgarFiling>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEdgarFiling {
    pub accession_number: String,
    pub filing_date: String,
    pub report_date: String,
    pub form: String,
    pub size: i64,
    pub primary_document: String,
    pub primary_doc_description: String,
}

// ── Search ─────────────────────────────────────────────────────────────────

#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEdgarSearchResults {
    pub total_hits: Option<i64>,
    pub hits: Vec<GqlEdgarSearchHit>,
}

#[derive(SimpleObject, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEdgarSearchHit {
    pub file_date: Option<String>,
    pub form: Option<String>,
    pub adsh: Option<String>,
    pub display_names: Vec<String>,
    pub ciks: Vec<String>,
}

// ── CIK resolution ──────────────────────────────────────────────────────────

/// Result of resolving a ticker symbol to its SEC CIK number.
#[derive(SimpleObject, Deserialize, Debug, Clone)]
#[graphql(rename_fields = "camelCase")]
pub struct GqlEdgarCik {
    pub symbol: String,
    pub cik: u64,
}
