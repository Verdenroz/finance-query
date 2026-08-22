//! House Clerk Financial Disclosure HTTP client.
//!
//! `disclosures-clerk.house.gov` serves two kinds of resources without
//! credentials: a per-year bulk archive (`{year}FD.zip`, containing an index
//! of every filing that year) and one PDF per filing (`ptr-pdfs/{year}/{doc_id}.pdf`).
//! There is no per-symbol or per-filing-type query endpoint, and no structured
//! per-filing data — every individual disclosure is a PDF, so the index is
//! only ever used to find which filings to open.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const HOUSE_BASE: &str = "https://disclosures-clerk.house.gov";

#[derive(Clone)]
pub(super) struct HouseTradesClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl HouseTradesClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self {
            http: keyless_http_client(timeout)?,
            limiter,
            base_url: base_url.into(),
        })
    }

    /// Fetch the bulk ZIP archive for one filing year (index + no documents;
    /// individual filings are fetched separately by doc id).
    pub(super) async fn fetch_year_archive(&self, year: i32) -> Result<Vec<u8>> {
        self.limiter.acquire().await;
        let url = format!("{}/public_disc/financial-pdfs/{year}FD.zip", self.base_url);
        debug!("House disclosures request: {year}FD.zip");
        self.fetch_bytes(&url).await
    }

    pub(super) async fn fetch_filing_pdf(&self, year: i32, doc_id: &str) -> Result<Vec<u8>> {
        self.limiter.acquire().await;
        let url = format!("{}/public_disc/ptr-pdfs/{year}/{doc_id}.pdf", self.base_url);
        debug!("House disclosures request: ptr-pdfs/{year}/{doc_id}.pdf");
        self.fetch_bytes(&url).await
    }

    async fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>> {
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(FinanceError::SymbolNotFound {
                symbol: None,
                context: format!("no House filing at {url}"),
            });
        }
        if !status.is_success() {
            return Err(status_error("House Clerk", status));
        }
        Ok(resp.bytes().await?.to_vec())
    }
}
