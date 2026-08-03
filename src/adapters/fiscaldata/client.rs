//! US Treasury FiscalData HTTP client.
//!
//! Keyless and unauthenticated. Every dataset shares one query grammar
//! (`fields` / `filter` / `sort` / `page[size]`), so one request method covers
//! all of them.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::{debug, warn};

use futures::stream::{StreamExt, TryStreamExt};

use super::models::{FiscalError, FiscalMeta, FiscalResponse, FiscalRow};
use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const FISCALDATA_BASE: &str =
    "https://api.fiscaldata.treasury.gov/services/api/fiscal_service";

/// How this API names itself in errors.
const API: &str = "US Treasury FiscalData";

/// FiscalData's documented maximum page size.
const PAGE_SIZE: u32 = 10_000;

/// Pages fetched at once once the page count is known. Small enough that a
/// mis-specified filter cannot burst the shared token bucket.
const MAX_CONCURRENT_PAGES: usize = 4;

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
        let fields = format!("{DATE_FIELD},{}", query.value_field);

        // Page 1 self-reports the page count, so the rest are independent and
        // knowable up front — no reason to walk them one round trip at a time.
        let mut first = self.page(query, &fields, 1).await?;
        let reported_pages = first.meta.total_pages.unwrap_or(1);
        let total_pages = reported_pages.min(MAX_PAGES);
        if reported_pages > MAX_PAGES {
            warn!(
                "FiscalData series {}/{} has {reported_pages} pages; truncated at {MAX_PAGES}",
                query.dataset, query.value_field
            );
        }

        let mut rows = std::mem::take(&mut first.data);
        if !rows.is_empty() && total_pages > 1 {
            // `buffered` preserves input order, so the pages reassemble
            // chronologically however the responses interleave.
            let rest: Vec<FiscalResponse> = futures::stream::iter(2..=total_pages)
                .map(|page| self.page(query, &fields, page))
                .buffered(MAX_CONCURRENT_PAGES)
                .try_collect()
                .await?;
            rows.extend(rest.into_iter().flat_map(|page| page.data));
        }

        if rows.is_empty() {
            return Err(FinanceError::SymbolNotFound {
                symbol: Some(format!("{}/{}", query.dataset, query.value_field)),
                context: "FiscalData returned no rows for this dataset/filter combination"
                    .to_string(),
            });
        }
        Ok((rows, first.meta))
    }

    /// Fetch one page of `query`. The rate limiter still paces every call, so
    /// concurrency here never outruns the token bucket.
    async fn page(
        &self,
        query: &SeriesQuery<'_>,
        fields: &str,
        page: u32,
    ) -> Result<FiscalResponse> {
        self.limiter.acquire().await;

        let url = format!("{}/{}", self.base_url, query.dataset);
        let page_str = page.to_string();
        let size_str = PAGE_SIZE.to_string();
        let mut params: Vec<(&str, &str)> = vec![
            ("format", "json"),
            ("fields", fields),
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

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "fiscaldata.response".to_string(),
            context: format!("unrecognised FiscalData envelope: {e}"),
        })
    }

    /// Map a non-2xx response, preferring the API's own explanation of what
    /// was wrong with the query over a bare status code.
    fn map_error(status: StatusCode, body: &[u8], query: &SeriesQuery<'_>) -> FinanceError {
        // Checked before the body: a throttled response carries no useful
        // explanation of the query.
        if status == StatusCode::TOO_MANY_REQUESTS {
            return status_error(API, status);
        }
        if let Ok(err) = serde_json::from_slice::<FiscalError>(body)
            && let Some(detail) = err.describe()
        {
            return FinanceError::MacroDataError {
                provider: API.to_string(),
                context: format!("{}/{}: {detail}", query.dataset, query.value_field),
            };
        }
        if status == StatusCode::NOT_FOUND {
            return FinanceError::SymbolNotFound {
                symbol: Some(query.dataset.to_string()),
                context: "no such FiscalData dataset".to_string(),
            };
        }
        status_error(API, status)
    }
}
