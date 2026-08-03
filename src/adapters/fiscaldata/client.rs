//! US Treasury FiscalData HTTP client.
//!
//! Keyless and unauthenticated. Every dataset shares one query grammar
//! (`fields` / `filter` / `sort` / `page[size]`), so one request method covers
//! all of them.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

use super::models::{FiscalError, FiscalMeta, FiscalResponse, FiscalRow};
use crate::adapters::common::keyless_http_client;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const FISCALDATA_BASE: &str =
    "https://api.fiscaldata.treasury.gov/services/api/fiscal_service";

/// FiscalData's documented maximum page size.
const PAGE_SIZE: u32 = 10_000;

/// Cap on pages walked for one series — 50k observations is far beyond any
/// dataset the curated series cover, and stops a mis-specified filter from
/// turning into an unbounded crawl.
const MAX_PAGES: u32 = 5;

/// The date column every FiscalData dataset keys its rows by.
pub(super) const DATE_FIELD: &str = "record_date";

/// A dataset query: which dataset, which value column, and an optional
/// row filter for datasets that stack several series in one table.
#[derive(Debug, Clone)]
pub(super) struct SeriesQuery<'a> {
    pub dataset: &'a str,
    pub value_field: &'a str,
    pub filter: Option<&'a str>,
}

pub(super) struct FiscalDataClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl FiscalDataClient {
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

    /// Fetch every row of `query`, oldest first, following pagination.
    ///
    /// Returns the rows together with the response `meta`, which self-describes
    /// the requested column (label and type).
    pub(super) async fn series(
        &self,
        query: &SeriesQuery<'_>,
    ) -> Result<(Vec<FiscalRow>, FiscalMeta)> {
        let url = format!("{}/{}", self.base_url, query.dataset);
        let fields = format!("{DATE_FIELD},{}", query.value_field);

        let mut rows: Vec<FiscalRow> = Vec::new();
        let mut meta = FiscalMeta::default();

        for page in 1..=MAX_PAGES {
            self.limiter.acquire().await;

            let page_str = page.to_string();
            let size_str = PAGE_SIZE.to_string();
            let mut params: Vec<(&str, &str)> = vec![
                ("format", "json"),
                ("fields", &fields),
                ("sort", DATE_FIELD),
                ("page[size]", &size_str),
                ("page[number]", &page_str),
            ];
            if let Some(filter) = query.filter {
                params.push(("filter", filter));
            }

            debug!("FiscalData request: {url} page {page}");
            let resp = self.http.get(&url).query(&params).send().await?;
            let status = resp.status();
            let bytes = resp.bytes().await?;

            if !status.is_success() {
                return Err(Self::map_error(status, &bytes, query));
            }

            let parsed: FiscalResponse = serde_json::from_slice(&bytes).map_err(|e| {
                FinanceError::ResponseStructureError {
                    field: "fiscaldata.response".to_string(),
                    context: format!("unrecognised FiscalData envelope: {e}"),
                }
            })?;

            let total_pages = parsed.meta.total_pages.unwrap_or(1);
            let batch = parsed.data.len();
            rows.extend(parsed.data);
            if page == 1 {
                meta = parsed.meta;
            }

            if batch == 0 || page >= total_pages {
                break;
            }
            if page == MAX_PAGES {
                warn!(
                    "FiscalData series {}/{} has {total_pages} pages; truncated at {MAX_PAGES}",
                    query.dataset, query.value_field
                );
            }
        }

        if rows.is_empty() {
            return Err(FinanceError::SymbolNotFound {
                symbol: Some(format!("{}/{}", query.dataset, query.value_field)),
                context: "FiscalData returned no rows for this dataset/filter combination"
                    .to_string(),
            });
        }
        Ok((rows, meta))
    }

    /// Map a non-2xx response, preferring the API's own explanation of what
    /// was wrong with the query over a bare status code.
    fn map_error(status: StatusCode, body: &[u8], query: &SeriesQuery<'_>) -> FinanceError {
        if status == StatusCode::TOO_MANY_REQUESTS {
            return FinanceError::RateLimited { retry_after: None };
        }
        if let Ok(err) = serde_json::from_slice::<FiscalError>(body)
            && let Some(detail) = err.describe()
        {
            return FinanceError::MacroDataError {
                provider: "US Treasury FiscalData".to_string(),
                context: format!("{}/{}: {detail}", query.dataset, query.value_field),
            };
        }
        if status == StatusCode::NOT_FOUND {
            return FinanceError::SymbolNotFound {
                symbol: Some(query.dataset.to_string()),
                context: "no such FiscalData dataset".to_string(),
            };
        }
        FinanceError::ExternalApiError {
            api: "US Treasury FiscalData".to_string(),
            status: status.as_u16(),
        }
    }
}
