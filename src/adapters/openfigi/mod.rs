//! OpenFIGI — keyless CUSIP/ISIN/SEDOL/FIGI to ticker mapping.
//!
//! Requires the **`openfigi`** feature flag.
//!
//! Exposed through [`crate::openfigi`], alongside [`crate::edgar`] and
//! [`crate::fred`], rather than through the Providers API: identifier
//! resolution is not tied to a symbol handle and maps onto no existing
//! [`Capability`](crate::Capability).
//!
//! # Keys
//!
//! None required. `OPENFIGI_API_KEY` is optional and raises the quota from
//! 25 requests/minute and 10 jobs per request to 25 requests/6 seconds and
//! 100 jobs per request.

pub(crate) mod client;
pub(crate) mod models;

use std::time::Duration;

use crate::error::{FinanceError, Result};
use crate::models::discovery::figi::{SecurityIdKind, SecurityMapping};
use client::OpenFigiClient;
use models::{FigiRecord, MappingJob};

const OPENFIGI_ANONYMOUS_RATE_PER_SEC: f64 = 25.0 / 60.0;
const OPENFIGI_AUTHENTICATED_RATE_PER_SEC: f64 = 25.0 / 6.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

static ANONYMOUS_LIMITER: std::sync::OnceLock<std::sync::Arc<crate::rate_limiter::RateLimiter>> =
    std::sync::OnceLock::new();
static AUTHENTICATED_LIMITER: std::sync::OnceLock<
    std::sync::Arc<crate::rate_limiter::RateLimiter>,
> = std::sync::OnceLock::new();

fn limiter(authenticated: bool) -> std::sync::Arc<crate::rate_limiter::RateLimiter> {
    let (slot, rate) = if authenticated {
        (&AUTHENTICATED_LIMITER, OPENFIGI_AUTHENTICATED_RATE_PER_SEC)
    } else {
        (&ANONYMOUS_LIMITER, OPENFIGI_ANONYMOUS_RATE_PER_SEC)
    };
    std::sync::Arc::clone(
        slot.get_or_init(|| std::sync::Arc::new(crate::rate_limiter::RateLimiter::new(rate))),
    )
}

/// Build a client against the live API, reusing the shared token bucket.
///
/// The optional key is read per call, so exporting it mid-process takes
/// effect immediately.
fn client() -> Result<OpenFigiClient> {
    let api_key = std::env::var("OPENFIGI_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty());
    let request_limiter = limiter(api_key.is_some());
    OpenFigiClient::new(
        DEFAULT_TIMEOUT,
        request_limiter,
        client::OPENFIGI_BASE,
        api_key,
    )
}

/// Map an OpenFIGI record onto the public [`SecurityMapping`].
fn to_mapping(record: FigiRecord) -> SecurityMapping {
    SecurityMapping {
        figi: record.figi,
        ticker: record.ticker,
        name: record.name,
        exchange_code: record.exch_code,
        composite_figi: record.composite_figi,
        share_class_figi: record.share_class_figi,
        security_type: record.security_type,
        market_sector: record.market_sector,
    }
}

/// Resolve one identifier to every instrument carrying it.
///
/// An identifier that is well-formed but matches nothing yields an empty
/// list; a malformed one is an error.
pub(crate) async fn resolve(kind: SecurityIdKind, id: &str) -> Result<Vec<SecurityMapping>> {
    let mut results = resolve_many(kind, std::slice::from_ref(&id)).await?;
    Ok(results.pop().unwrap_or_default())
}

/// Resolve several identifiers of the same kind in as few requests as
/// possible.
///
/// The returned vector is positional: element `i` answers `ids[i]`, with an
/// empty list where nothing matched. A malformed identifier fails the whole
/// call rather than silently reading as "no match".
pub(crate) async fn resolve_many(
    kind: SecurityIdKind,
    ids: &[&str],
) -> Result<Vec<Vec<SecurityMapping>>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let client = client()?;
    let max_jobs = client.max_jobs_per_request();
    let mut out: Vec<Vec<SecurityMapping>> = Vec::with_capacity(ids.len());

    for chunk in ids.chunks(max_jobs) {
        let jobs: Vec<MappingJob<'_>> = chunk
            .iter()
            .map(|id| MappingJob {
                id_type: kind.as_str(),
                id_value: id,
            })
            .collect();

        for (result, id) in client.map(&jobs).await?.into_iter().zip(chunk) {
            if let Some(error) = result.error {
                return Err(FinanceError::InvalidParameter {
                    param: format!("{kind}"),
                    reason: format!("OpenFIGI rejected '{id}': {error}"),
                });
            }
            // A `warning` means the identifier was well-formed but matched
            // nothing — an empty result, not a failure.
            if let Some(warning) = result.warning {
                tracing::debug!("OpenFIGI: {kind} '{id}' matched nothing ({warning})");
            }
            out.push(
                result
                    .data
                    .unwrap_or_default()
                    .into_iter()
                    .map(to_mapping)
                    .collect(),
            );
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> OpenFigiClient {
        OpenFigiClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
            None,
        )
        .unwrap()
    }

    fn keyed_test_client(base_url: &str) -> OpenFigiClient {
        OpenFigiClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
            Some("test-key".to_string()),
        )
        .unwrap()
    }

    #[test]
    fn api_key_enables_the_documented_batch_size() {
        let limiter = Arc::new(RateLimiter::new(100.0));
        let anonymous = OpenFigiClient::new(
            Duration::from_secs(5),
            Arc::clone(&limiter),
            client::OPENFIGI_BASE,
            None,
        )
        .unwrap();
        let authenticated = OpenFigiClient::new(
            Duration::from_secs(5),
            limiter,
            client::OPENFIGI_BASE,
            Some("test-key".to_string()),
        )
        .unwrap();

        assert_eq!(anonymous.max_jobs_per_request(), 10);
        assert_eq!(authenticated.max_jobs_per_request(), 100);
    }

    #[tokio::test]
    async fn api_key_is_sent_in_the_documented_header() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("POST", "/mapping")
            .match_header("X-OPENFIGI-APIKEY", "test-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"warning":"No identifier found."}]"#)
            .create_async()
            .await;
        let jobs = [MappingJob {
            id_type: "TICKER",
            id_value: "UNKNOWN",
        }];

        keyed_test_client(&server.url()).map(&jobs).await.unwrap();
    }

    fn apple_payload() -> String {
        // Verbatim shape from api.openfigi.com/v3/mapping.
        serde_json::json!([{
            "data": [
                { "figi": "BBG000B9XRY4", "name": "APPLE INC", "ticker": "AAPL",
                  "exchCode": "US", "compositeFIGI": "BBG000B9XRY4",
                  "securityType": "Common Stock", "marketSector": "Equity",
                  "shareClassFIGI": "BBG001S5N8V8", "securityType2": "Common Stock",
                  "securityDescription": "AAPL" },
                { "figi": "BBG000B9XVV8", "name": "APPLE INC", "ticker": "AAPL",
                  "exchCode": "UN", "compositeFIGI": "BBG000B9XRY4",
                  "securityType": "Common Stock", "marketSector": "Equity",
                  "shareClassFIGI": "BBG001S5N8V8", "securityType2": "Common Stock",
                  "securityDescription": "AAPL" }
            ]
        }])
        .to_string()
    }

    #[test]
    fn id_kinds_use_openfigis_own_type_names() {
        assert_eq!(SecurityIdKind::Cusip.as_str(), "ID_CUSIP");
        assert_eq!(SecurityIdKind::Isin.as_str(), "ID_ISIN");
        assert_eq!(SecurityIdKind::Sedol.as_str(), "ID_SEDOL");
        // A FIGI is looked up under Bloomberg's own id type.
        assert_eq!(SecurityIdKind::Figi.as_str(), "ID_BB_GLOBAL");
        assert_eq!(SecurityIdKind::Ticker.as_str(), "TICKER");
    }

    #[tokio::test]
    async fn a_cusip_resolves_to_every_listing_it_covers() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/mapping")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!([{
                "idType": "ID_CUSIP",
                "idValue": "037833100"
            }])))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(apple_payload())
            .create_async()
            .await;

        let jobs = vec![MappingJob {
            id_type: "ID_CUSIP",
            id_value: "037833100",
        }];
        let results = test_client(&server.url()).map(&jobs).await.unwrap();
        let mappings: Vec<_> = results[0]
            .data
            .clone()
            .unwrap()
            .into_iter()
            .map(to_mapping)
            .collect();

        // One CUSIP covers many venue listings.
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].figi, "BBG000B9XRY4");
        assert_eq!(mappings[0].ticker.as_deref(), Some("AAPL"));
        assert_eq!(mappings[0].name.as_deref(), Some("APPLE INC"));
        assert_eq!(mappings[0].exchange_code.as_deref(), Some("US"));
        assert_eq!(mappings[1].exchange_code.as_deref(), Some("UN"));
        // Both roll up to the same composite and share class. Asserted
        // against the literal values, not against each other: OpenFIGI
        // capitalises these keys (`compositeFIGI`), so a rename bug would
        // leave both `None` and pass an equality-only check.
        assert_eq!(mappings[0].composite_figi.as_deref(), Some("BBG000B9XRY4"));
        assert_eq!(mappings[1].composite_figi.as_deref(), Some("BBG000B9XRY4"));
        assert_eq!(
            mappings[0].share_class_figi.as_deref(),
            Some("BBG001S5N8V8")
        );
    }

    #[tokio::test]
    async fn a_length_mismatch_is_rejected_rather_than_mispaired() {
        // Results are positional; a short array would silently pair answers
        // with the wrong identifiers.
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/mapping")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!([{ "warning": "No identifier found." }]).to_string())
            .create_async()
            .await;

        let jobs = vec![
            MappingJob {
                id_type: "ID_CUSIP",
                id_value: "037833100",
            },
            MappingJob {
                id_type: "ID_CUSIP",
                id_value: "594918104",
            },
        ];
        let err = test_client(&server.url()).map(&jobs).await.unwrap_err();
        assert!(
            matches!(err, FinanceError::ResponseStructureError { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn an_oversized_batch_explains_the_limit() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/mapping")
            .with_status(413)
            .create_async()
            .await;

        let jobs = vec![MappingJob {
            id_type: "ID_CUSIP",
            id_value: "037833100",
        }];
        let err = test_client(&server.url()).map(&jobs).await.unwrap_err();
        match err {
            FinanceError::InvalidParameter { reason, .. } => {
                assert!(reason.contains("at most 10"), "got {reason}");
            }
            other => panic!("expected InvalidParameter, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn rate_limit_maps_to_rate_limited() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/mapping")
            .with_status(429)
            .create_async()
            .await;

        let jobs = vec![MappingJob {
            id_type: "ID_CUSIP",
            id_value: "037833100",
        }];
        let err = test_client(&server.url()).map(&jobs).await.unwrap_err();
        assert!(matches!(err, FinanceError::RateLimited { .. }), "{err:?}");
    }

    #[tokio::test]
    async fn resolving_nothing_makes_no_request() {
        // No mock server exists, so any HTTP call would fail the test.
        assert!(
            resolve_many(SecurityIdKind::Cusip, &[])
                .await
                .unwrap()
                .is_empty()
        );
    }
}
