//! CFTC Commitments of Traders HTTP client (Socrata public reporting API).
//!
//! Keyless and unauthenticated. Socrata's anonymous tier throttles by
//! rolling-hour request count rather than per-second, so the client only
//! self-paces enough to avoid looking like a burst.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tracing::debug;

use super::models::{CftcError, CotRow};
use crate::adapters::common::keyless_http_client;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

/// The disaggregated futures-only combined report (Socrata dataset
/// `72hh-3qpy`) — what "Commitments of Traders" means for the physical
/// commodities it covers (agriculture, energy, metals). Financial futures
/// (equity indices, rates, currencies) are reported separately by the CFTC
/// in the Traders in Financial Futures report, which this adapter does not
/// serve.
pub(super) const CFTC_BASE: &str = "https://publicreporting.cftc.gov/resource/72hh-3qpy.json";

/// Weekly reports; 104 rows covers roughly two years of history per call.
const ROW_LIMIT: u32 = 104;

pub(super) struct CftcClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl CftcClient {
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

    /// Fetch weekly COT rows for one `cftc_contract_market_code`, newest
    /// first (the order requested via Socrata's `$order`).
    pub(super) async fn commitments_of_traders(&self, contract_code: &str) -> Result<Vec<CotRow>> {
        self.limiter.acquire().await;

        let where_clause = format!("cftc_contract_market_code='{}'", escape_soql(contract_code));
        debug!("CFTC request: {where_clause}");

        let resp = self
            .http
            .get(&self.base_url)
            .query(&[
                ("$where", where_clause.as_str()),
                ("$order", "report_date_as_yyyy_mm_dd DESC"),
                ("$limit", &ROW_LIMIT.to_string()),
            ])
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            return Err(Self::map_error(status, &bytes, contract_code));
        }

        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "cftc.commitments_of_traders".to_string(),
            context: format!("unrecognised CFTC payload: {e}"),
        })
    }

    fn map_error(status: StatusCode, body: &[u8], contract_code: &str) -> FinanceError {
        // Checked before the body: a throttled response carries no useful
        // explanation of the query.
        if status == StatusCode::TOO_MANY_REQUESTS {
            return FinanceError::RateLimited { retry_after: None };
        }
        if let Ok(err) = serde_json::from_slice::<CftcError>(body)
            && let Some(message) = err.message
        {
            return FinanceError::ApiError(format!("CFTC ({contract_code}): {message}"));
        }
        FinanceError::ExternalApiError {
            api: "CFTC".to_string(),
            status: status.as_u16(),
        }
    }
}

/// Escape a value for safe inclusion inside a SoQL string literal by
/// doubling embedded single quotes (SoQL's own escaping convention),
/// preventing a crafted symbol from breaking out of the `$where` clause.
fn escape_soql(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_quotes_are_doubled() {
        assert_eq!(escape_soql("088691"), "088691");
        assert_eq!(escape_soql("O'BRIEN"), "O''BRIEN");
        assert_eq!(escape_soql("088691' OR '1'='1"), "088691'' OR ''1''=''1");
    }
}
