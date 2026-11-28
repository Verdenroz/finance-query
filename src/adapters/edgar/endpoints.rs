//! SEC EDGAR API endpoint URLs.
//!
//! All EDGAR API endpoints are free, public, and require no authentication
//! (only a proper User-Agent header with contact email).

use crate::error::{FinanceError, Result};
use std::borrow::Cow;

const HOSTS: [&str; 3] = [
    "https://www.sec.gov",
    "https://data.sec.gov",
    "https://efts.sec.gov",
];

/// Swap the SEC host in `url` for `origin`, leaving the URL untouched when
/// there is no origin. A host outside the known set is refused so an override
/// never lets a request fall through to live SEC. A URL already on `origin`
/// passes through.
pub(crate) fn resolve<'a>(origin: Option<&str>, url: &'a str) -> Result<Cow<'a, str>> {
    let Some(origin) = origin else {
        return Ok(Cow::Borrowed(url));
    };
    let on_boundary = |rest: &&str| matches!(rest.chars().next(), None | Some('/' | '?'));
    if url
        .strip_prefix(origin)
        .is_some_and(|rest| on_boundary(&rest))
    {
        return Ok(Cow::Borrowed(url));
    }
    HOSTS
        .iter()
        .find_map(|host| url.strip_prefix(host))
        .filter(on_boundary)
        .map(|rest| Cow::Owned(format!("{origin}{rest}")))
        .ok_or_else(|| FinanceError::InvalidParameter {
            param: "endpoint".into(),
            reason: format!("`{url}` is not an SEC URL the endpoint override can serve"),
        })
}

/// All ticker-to-CIK mappings (JSON, ~2MB).
pub const COMPANY_TICKERS: &str = "https://www.sec.gov/files/company_tickers.json";

/// Ticker-to-listing-exchange mappings (JSON, ~1MB). Covers active listings
/// only — SEC keeps no delisted-securities file.
pub const COMPANY_TICKERS_EXCHANGE: &str =
    "https://www.sec.gov/files/company_tickers_exchange.json";

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

/// Build the filing index URL for a CIK and accession (no dashes).
///
/// Filing documents live under `www.sec.gov/Archives`, not `data.sec.gov`
/// (which serves only the submissions/XBRL JSON APIs above) — `data.sec.gov`
/// 404s on this path.
pub fn filing_index(cik: &str, accession_no_dashes: &str) -> String {
    format!(
        "https://www.sec.gov/Archives/edgar/data/{}/{}/index.json",
        cik, accession_no_dashes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_url() -> Vec<String> {
        vec![
            COMPANY_TICKERS.into(),
            COMPANY_TICKERS_EXCHANGE.into(),
            submissions(320193),
            company_facts(320193),
            FULL_TEXT_SEARCH.into(),
            filing_index("320193", "000032019324000123"),
            "https://www.sec.gov/files/data/fails-deliver-data/cnsfails202401a.zip".into(),
        ]
    }

    #[test]
    fn resolve_without_an_origin_returns_every_url_unchanged() {
        for url in every_url() {
            assert_eq!(resolve(None, &url).unwrap(), url);
        }
    }

    #[test]
    fn resolve_keeps_each_path_distinct_under_one_origin() {
        let resolved: Vec<_> = every_url()
            .iter()
            .map(|url| {
                resolve(Some("http://127.0.0.1:9"), url)
                    .unwrap()
                    .into_owned()
            })
            .collect();
        assert!(
            resolved
                .iter()
                .all(|u| u.starts_with("http://127.0.0.1:9/"))
        );
        let mut unique = resolved.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), resolved.len());
    }

    #[test]
    fn resolve_refuses_unknown_and_look_alike_hosts() {
        for url in [
            "https://xbrl.sec.gov/dei/2026/dei-2026.xsd",
            "https://www.sec.gov.evil.test/files/x.json",
            "https://example.com/submissions/CIK0000320193.json",
        ] {
            assert!(resolve(Some("http://127.0.0.1:9"), url).is_err(), "{url}");
        }
    }

    #[test]
    fn resolve_passes_a_url_already_on_the_origin() {
        let url = "http://127.0.0.1:9/Archives/edgar/data/1/2/x.htm";
        assert_eq!(resolve(Some("http://127.0.0.1:9"), url).unwrap(), url);
        assert!(resolve(Some("http://127.0.0.1:9"), "http://127.0.0.1:99/x").is_err());
    }

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

    #[test]
    fn test_filing_index_uses_www_not_data_host() {
        assert_eq!(
            filing_index("320193", "000032019324000123"),
            "https://www.sec.gov/Archives/edgar/data/320193/000032019324000123/index.json"
        );
    }
}
