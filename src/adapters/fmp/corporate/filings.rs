#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SecFilingDTO {
    pub symbol: Option<String>,
    pub cik: Option<String>,
    #[serde(rename = "filingDate")]
    pub filing_date: Option<String>,
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    #[serde(rename = "formType")]
    pub form_type: Option<String>,
    pub link: Option<String>,
    #[serde(rename = "finalLink")]
    pub final_link: Option<String>,
}

pub async fn sec_filings_by_symbol(
    symbol: &str,
    from: &str,
    to: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<SecFilingDTO>> {
    let page = page.to_string();
    let limit = limit.to_string();
    build_client()?
        .get(
            "/stable/sec-filings-search/symbol",
            &[
                ("symbol", symbol),
                ("from", from),
                ("to", to),
                ("page", &page),
                ("limit", &limit),
            ],
        )
        .await
}

pub async fn sec_filings_by_form_type(
    form_type: &str,
    from: &str,
    to: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<SecFilingDTO>> {
    let page = page.to_string();
    let limit = limit.to_string();
    build_client()?
        .get(
            "/stable/sec-filings-search/form-type",
            &[
                ("formType", form_type),
                ("from", from),
                ("to", to),
                ("page", &page),
                ("limit", &limit),
            ],
        )
        .await
}

pub async fn sec_filings_by_cik(
    cik: &str,
    from: &str,
    to: &str,
    page: u32,
    limit: u32,
) -> Result<Vec<SecFilingDTO>> {
    let page = page.to_string();
    let limit = limit.to_string();
    build_client()?
        .get(
            "/stable/sec-filings-search/cik",
            &[
                ("cik", cik),
                ("from", from),
                ("to", to),
                ("page", &page),
                ("limit", &limit),
            ],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filing_payload_deserializes() {
        let filing: SecFilingDTO = serde_json::from_str(
            r#"{"symbol":"AAPL","cik":"0000320193","filingDate":"2026-07-31","acceptedDate":"2026-07-31 06:01:02","formType":"10-Q","link":"https://example.com","finalLink":"https://example.com/aapl.htm"}"#,
        )
        .unwrap();
        assert_eq!(filing.form_type.as_deref(), Some("10-Q"));
    }
}
