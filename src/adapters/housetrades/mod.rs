//! House of Representatives Periodic Transaction Report (PTR) disclosures —
//! keyless, from the primary source.
//!
//! Requires the **`housetrades`** feature flag.
//!
//! The STOCK Act of 2012 requires members to disclose trades over $1,000
//! within 45 days via a Periodic Transaction Report. The House Clerk
//! publishes these free of charge and without credentials at
//! `disclosures-clerk.house.gov` (see [`client`] for the resource shape).
//! Filings typed through fd.house.gov's e-filing system carry a text layer
//! `pdf_extract` can read; older or hand-signed ones are scanned images and
//! are silently skipped rather than OCR'd — see [`filings`] for that boundary.
//!
//! Senate PTRs are a separate adapter (`crate::adapters::senatetrades`) —
//! `efdsearch.senate.gov` renders everything client-side behind Akamai bot
//! protection, so it needs a real browser rather than a plain HTTP client.

pub(crate) mod client;
pub(crate) mod filings;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::{HOUSE_BASE, HouseTradesClient};

/// Self-imposed pacing. The House Clerk site publishes no documented rate
/// limit; this only keeps a symbol lookup's burst of PDF fetches polite.
const HOUSE_RATE_PER_SEC: f64 = 5.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = HOUSE_RATE_PER_SEC);

/// Build a client against the live site, reusing the shared token bucket.
fn client() -> Result<HouseTradesClient> {
    HouseTradesClient::new(DEFAULT_TIMEOUT, shared_limiter(), HOUSE_BASE)
}

pub(crate) use filings::fetch_congressional_trades_response;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> HouseTradesClient {
        HouseTradesClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn fetches_a_year_archive() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/public_disc/financial-pdfs/2025FD.zip")
            .with_status(200)
            .with_header("content-type", "application/x-zip-compressed")
            .with_body(b"PK\x03\x04fake-zip-bytes")
            .create_async()
            .await;

        let bytes = test_client(&server.url())
            .fetch_year_archive(2025)
            .await
            .unwrap();
        assert!(bytes.starts_with(b"PK"));
    }

    #[tokio::test]
    async fn fetches_a_filing_pdf() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/public_disc/ptr-pdfs/2025/20032062.pdf")
            .with_status(200)
            .with_header("content-type", "application/pdf")
            .with_body(b"%PDF-1.5 fake")
            .create_async()
            .await;

        let bytes = test_client(&server.url())
            .fetch_filing_pdf(2025, "20032062")
            .await
            .unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn a_missing_filing_maps_to_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/public_disc/ptr-pdfs/2025/00000000.pdf")
            .with_status(404)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .fetch_filing_pdf(2025, "00000000")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/public_disc/financial-pdfs/2025FD.zip")
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .fetch_year_archive(2025)
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }
}
