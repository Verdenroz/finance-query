//! EDGAR full-text search (EFTS) mapped onto the routed FILINGS capability.
//!
//! EFTS indexes filing *text* across every filer, so it answers a different
//! question from the per-CIK submissions endpoint: "which filings mention X"
//! rather than "what has this company filed".

use crate::adapters::edgar::build_client;
use crate::error::Result;
use crate::models::filings::{
    EdgarSearchHit, EdgarSearchResults, FilingSearchFilters, FilingSearchHit,
};

/// EFTS refuses page sizes above 100.
const MAX_SIZE: u32 = 100;

/// EFTS hit ids are `"{accession}:{document}"`; the document half is what makes
/// a direct archive URL possible, so a hit without it gets no URL rather than a
/// guessed one.
fn hit_url(id: Option<&str>, ciks: &[String]) -> Option<String> {
    let (accession, document) = id?.split_once(':')?;
    if document.is_empty() {
        return None;
    }
    let cik = ciks.first()?.trim_start_matches('0');
    if cik.is_empty() {
        return None;
    }
    Some(format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
        cik,
        accession.replace('-', ""),
        document
    ))
}

pub(crate) fn to_search_hit(hit: EdgarSearchHit) -> FilingSearchHit {
    // Destructure once so each field moves out; EFTS returns up to 100 hits per
    // search and cloning every field off a borrowed source adds up.
    let (accession_number, form, filed_date, period_ending, company_names, ciks) = match hit._source
    {
        Some(s) => (
            s.adsh,
            s.form,
            s.file_date,
            s.period_ending,
            s.display_names,
            s.ciks,
        ),
        None => (None, None, None, None, Vec::new(), Vec::new()),
    };
    FilingSearchHit {
        accession_number,
        form,
        filed_date,
        period_ending,
        company_names,
        url: hit_url(hit._id.as_deref(), &ciks),
        ciks,
        score: hit._score,
    }
}

pub(crate) fn to_search_hits(results: EdgarSearchResults) -> Vec<FilingSearchHit> {
    results
        .hits
        .map(|h| h.hits)
        .unwrap_or_default()
        .into_iter()
        .map(to_search_hit)
        .collect()
}

/// Run a full-text search over EDGAR filings and return canonical hits.
///
/// `symbol` scopes the search to one filer. EFTS is keyed by CIK, so resolving
/// it is EDGAR's own business — the routed handle never sees the identifier.
pub async fn fetch_filing_search_response(
    symbol: Option<&str>,
    query: &str,
    filters: &FilingSearchFilters,
) -> Result<Vec<FilingSearchHit>> {
    let forms: Option<Vec<&str>> = filters
        .forms
        .as_ref()
        .map(|f| f.iter().map(String::as_str).collect());
    let size = filters.limit.map(|l| l.min(MAX_SIZE) as usize);

    let client = build_client()?;
    let resolved_cik = match (&filters.cik, symbol) {
        (None, Some(sym)) => Some(format!("{:010}", client.resolve_cik(sym).await?)),
        _ => None,
    };

    let results = client
        .search_filtered(
            query,
            forms.as_deref(),
            filters.start_date.as_deref(),
            filters.end_date.as_deref(),
            None,
            size,
            resolved_cik.as_deref().or(filters.cik.as_deref()),
        )
        .await?;

    Ok(to_search_hits(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn results_json() -> serde_json::Value {
        serde_json::json!({
            "hits": {
                "total": { "value": 2, "relation": "eq" },
                "max_score": 1.5,
                "hits": [
                    {
                        "_index": "edgar-filings",
                        "_id": "0000320193-24-000123:aapl-20240928.htm",
                        "_score": 1.5,
                        "_source": {
                            "ciks": ["0000320193"],
                            "file_date": "2024-11-01",
                            "form": "10-K",
                            "adsh": "0000320193-24-000123",
                            "display_names": ["Apple Inc. (AAPL)"],
                            "period_ending": "2024-09-28",
                            "root_forms": ["10-K"],
                            "sics": ["3571"]
                        }
                    },
                    {
                        "_id": "0000789019-24-000090",
                        "_score": 1.1,
                        "_source": {
                            "ciks": ["0000789019"],
                            "form": "8-K",
                            "adsh": "0000789019-24-000090",
                            "display_names": ["MICROSOFT CORP (MSFT)"]
                        }
                    }
                ]
            }
        })
    }

    #[test]
    fn flattens_efts_hits_into_canonical_shape() {
        let results: EdgarSearchResults = serde_json::from_value(results_json()).unwrap();
        let hits = to_search_hits(results);

        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].form.as_deref(), Some("10-K"));
        assert_eq!(
            hits[0].accession_number.as_deref(),
            Some("0000320193-24-000123")
        );
        assert_eq!(hits[0].filed_date.as_deref(), Some("2024-11-01"));
        assert_eq!(hits[0].period_ending.as_deref(), Some("2024-09-28"));
        assert_eq!(hits[0].company_names, vec!["Apple Inc. (AAPL)"]);
        assert_eq!(hits[0].score, Some(1.5));
    }

    #[test]
    fn derives_the_archive_url_from_the_hit_id() {
        let results: EdgarSearchResults = serde_json::from_value(results_json()).unwrap();
        let hits = to_search_hits(results);

        assert_eq!(
            hits[0].url.as_deref(),
            Some(
                "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/aapl-20240928.htm"
            )
        );
        // Second hit's id carries no document half, so no URL is invented.
        assert_eq!(hits[1].url, None);
    }

    #[test]
    fn hit_url_declines_incomplete_inputs() {
        let ciks = vec!["0000320193".to_string()];
        assert_eq!(hit_url(None, &ciks), None);
        assert_eq!(hit_url(Some("0000320193-24-000123:"), &ciks), None);
        assert_eq!(hit_url(Some("a:b.htm"), &[]), None);
        assert_eq!(hit_url(Some("a:b.htm"), &["0000".to_string()]), None);
    }

    #[test]
    fn empty_results_yield_no_hits() {
        let results: EdgarSearchResults = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(to_search_hits(results).is_empty());
    }
}
