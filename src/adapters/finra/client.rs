//! FINRA Query API HTTP client.
//!
//! `api.finra.org/data/*` serves the public datasets without credentials
//! (a free account raises the quota, but none is required). Requests are
//! POSTed as JSON; `Accept: application/json` is required or FINRA answers
//! with CSV.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{CompareFilter, DataRequest, DateRangeFilter, FinraError, RegShoDailyRow};
use crate::adapters::common::{keyless_http_client, status_error};
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const FINRA_BASE: &str = "https://api.finra.org/data/group";

/// Rows requested per call. Three reporting facilities report per symbol per
/// day, so this covers roughly a year of trading days.
const ROW_LIMIT: u32 = 1_000;

pub(super) struct FinraClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl FinraClient {
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

    /// Fetch daily short-sale volume rows for `symbol` over `[start, end]`
    /// (`YYYY-MM-DD`), one row per reporting facility per date.
    pub(super) async fn reg_sho_daily(
        &self,
        symbol: &str,
        start: String,
        end: String,
    ) -> Result<Vec<RegShoDailyRow>> {
        self.limiter.acquire().await;

        let url = format!("{}/otcMarket/name/regShoDaily", self.base_url);
        let body = DataRequest {
            limit: ROW_LIMIT,
            compare_filters: vec![CompareFilter {
                field_name: "securitiesInformationProcessorSymbolIdentifier",
                field_value: symbol,
                compare_type: "EQUAL",
            }],
            date_range_filters: vec![DateRangeFilter {
                field_name: "tradeReportDate",
                start_date: start,
                end_date: end,
            }],
        };
        debug!("FINRA request: regShoDaily {symbol}");

        let resp = self
            .http
            .post(&url)
            // Without this FINRA answers with CSV, not JSON.
            .header(reqwest::header::ACCEPT, "application/json")
            .json(&body)
            .send()
            .await?;
        let status = resp.status();

        // FINRA signals "matched nothing" with 204 and an empty body, which is
        // not an error — the symbol simply had no reportable short volume.
        if status == StatusCode::NO_CONTENT {
            return Ok(Vec::new());
        }

        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Self::map_error(status, &bytes, symbol));
        }

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "finra.regShoDaily".to_string(),
            context: format!("unrecognised FINRA payload: {e}"),
        })
    }

    /// Map a non-2xx response, preferring FINRA's own explanation.
    fn map_error(status: StatusCode, body: &[u8], symbol: &str) -> FinanceError {
        // Checked before the body: a throttled response carries no useful
        // explanation of the query.
        if status == StatusCode::TOO_MANY_REQUESTS {
            return status_error("FINRA", status);
        }
        if let Ok(err) = serde_json::from_slice::<FinraError>(body)
            && let Some(message) = err.message
        {
            return FinanceError::ApiError(format!("FINRA ({symbol}): {message}"));
        }
        status_error("FINRA", status)
    }
}
