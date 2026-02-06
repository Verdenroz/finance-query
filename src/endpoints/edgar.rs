//! SEC EDGAR API endpoint URLs.
//!
//! All EDGAR API endpoints are free, public, and require no authentication
//! (only a proper User-Agent header with contact email).

/// All ticker-to-CIK mappings (JSON, ~2MB).
pub const COMPANY_TICKERS: &str = "https://www.sec.gov/files/company_tickers.json";

/// Build the submissions URL for a CIK (filing history + company metadata).
///
/// CIK is zero-padded to 10 digits as required by the EDGAR API.
pub fn submissions(cik: u64) -> String {
    format!("https://data.sec.gov/submissions/CIK{:010}.json", cik)
}

/// Build the company facts URL for a CIK (structured XBRL financial data).
///
/// CIK is zero-padded to 10 digits as required by the EDGAR API.
pub fn company_facts(cik: u64) -> String {
    format!(
        "https://data.sec.gov/api/xbrl/companyfacts/CIK{:010}.json",
        cik
    )
}

/// Full-text search endpoint (EFTS).
pub const FULL_TEXT_SEARCH: &str = "https://efts.sec.gov/LATEST/search-index";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submissions_url_padding() {
        assert_eq!(
            submissions(320193),
            "https://data.sec.gov/submissions/CIK0000320193.json"
        );
    }

    #[test]
    fn test_company_facts_url_padding() {
        assert_eq!(
            company_facts(320193),
            "https://data.sec.gov/api/xbrl/companyfacts/CIK0000320193.json"
        );
    }

    #[test]
    fn test_small_cik_padding() {
        assert_eq!(
            submissions(1),
            "https://data.sec.gov/submissions/CIK0000000001.json"
        );
    }
}
