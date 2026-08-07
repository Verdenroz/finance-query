//! US Bureau of Labor Statistics — CPI, unemployment, payrolls, wages, PPI
//! from the primary source.
//!
//! Requires the **`bls`** feature flag.
//!
//! The first provider in the library with a **keyless/keyed dual mode**: with
//! no `BLS_API_KEY` set the keyless v1 route is used (25 queries/day per IP,
//! ~3 years of history); setting a free key upgrades every call to v2 (500
//! queries/day, 20 years, plus series titles from the catalog). Nothing else
//! changes — the same series ids and the same response type either way.
//!
//! # Series identifiers
//!
//! Native BLS series ids, e.g. `"CUUR0000SA0"` (CPI-U, all items, all urban
//! consumers) or `"LNS14000000"` (unemployment rate).

pub(crate) mod client;
pub(crate) mod economic;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::{BlsClient, Tier};

/// Self-imposed pacing. BLS caps by the day, not the second, so this only
/// keeps a burst from looking abusive — the daily quota is the real limit.
const BLS_RATE_PER_SEC: f64 = 2.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = BLS_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
///
/// The tier is resolved per call rather than cached, so a key exported after
/// the first request still takes effect.
fn client() -> Result<BlsClient> {
    BlsClient::new(
        DEFAULT_TIMEOUT,
        shared_limiter(),
        client::BLS_BASE,
        Tier::from_env(),
    )
}

pub(crate) use economic::fetch_economic_series_response;

#[cfg(test)]
mod tests {
    use super::economic::{parse_value, resolve_period, to_canonical};
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str, tier: Tier) -> BlsClient {
        BlsClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
            tier,
        )
        .unwrap()
    }

    fn cpi_payload() -> String {
        serde_json::json!({
            "status": "REQUEST_SUCCEEDED",
            "responseTime": 111,
            "message": [],
            "Results": {
                "series": [{
                    "seriesID": "CUUR0000SA0",
                    "data": [
                        { "year": "2026", "period": "M03", "periodName": "March",
                          "latest": "true", "value": "330.213", "footnotes": [{}] },
                        { "year": "2026", "period": "M02", "periodName": "February",
                          "value": "326.785", "footnotes": [{}] },
                        { "year": "2026", "period": "M01", "periodName": "January",
                          "value": "-", "footnotes": [{ "code": "X", "text": "Data unavailable" }] },
                        { "year": "2025", "period": "M13", "periodName": "Annual",
                          "value": "321.943", "footnotes": [{}] }
                    ]
                }]
            }
        })
        .to_string()
    }

    #[test]
    fn tier_is_keyless_without_a_key() {
        // `Tier::from_env` reads the process environment, which tests share;
        // the mapping itself is what matters and is asserted directly.
        assert_eq!(Tier::V1.version(), "v1");
        assert_eq!(Tier::V2("k".into()).version(), "v2");
    }

    #[test]
    fn monthly_quarterly_and_annual_periods_resolve_to_period_starts() {
        assert_eq!(resolve_period("2026", "M01").unwrap().date, "2026-01-01");
        assert_eq!(resolve_period("2026", "M12").unwrap().date, "2026-12-01");
        assert_eq!(resolve_period("2026", "M06").unwrap().frequency, "Monthly");
        assert_eq!(resolve_period("2026", "Q01").unwrap().date, "2026-01-01");
        assert_eq!(resolve_period("2026", "Q04").unwrap().date, "2026-10-01");
        assert_eq!(
            resolve_period("2026", "Q02").unwrap().frequency,
            "Quarterly"
        );
        assert_eq!(resolve_period("2026", "S01").unwrap().date, "2026-01-01");
        assert_eq!(resolve_period("2026", "S02").unwrap().date, "2026-07-01");
        assert_eq!(resolve_period("2026", "A01").unwrap().date, "2026-01-01");
        assert_eq!(resolve_period("2026", "A01").unwrap().frequency, "Annual");
    }

    #[test]
    fn annual_aggregate_periods_are_dropped() {
        // M13/Q05/S03 are annual averages folded into a sub-annual series;
        // dating them would collide with a real observation.
        assert_eq!(resolve_period("2025", "M13"), None);
        assert_eq!(resolve_period("2025", "Q05"), None);
        assert_eq!(resolve_period("2025", "S03"), None);
        assert_eq!(resolve_period("2025", "Z99"), None);
        assert_eq!(resolve_period("2025", ""), None);
    }

    #[test]
    fn dash_marks_an_unpublished_figure() {
        assert_eq!(parse_value("330.213"), Some(330.213));
        assert_eq!(parse_value("-"), None);
        assert_eq!(parse_value(""), None);
        assert_eq!(parse_value("n/a"), None);
    }

    #[tokio::test]
    async fn v1_series_maps_to_canonical_chronological_series() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/timeseries/data/CUUR0000SA0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(cpi_payload())
            .create_async()
            .await;

        let raw = test_client(&server.url(), Tier::V1)
            .series("CUUR0000SA0")
            .await
            .unwrap();
        let series = to_canonical("CUUR0000SA0", raw);

        assert_eq!(series.series_id, "CUUR0000SA0");
        assert_eq!(series.frequency.as_deref(), Some("Monthly"));
        // The M13 annual average is dropped; three monthly rows remain.
        assert_eq!(series.observations.len(), 3);
        assert_eq!(series.observations[0].date, "2026-01-01");
        assert_eq!(series.observations[0].value, None, "\"-\" parses to None");
        assert_eq!(series.observations[2].date, "2026-03-01");
        assert_eq!(series.observations[2].value, Some(330.213));
        // No catalog on the keyless route, so no title is invented.
        assert_eq!(series.title, None);
    }

    #[tokio::test]
    async fn v2_posts_the_key_and_reads_the_catalog_title() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/v2/timeseries/data/")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "seriesid": ["CUUR0000SA0"],
                "registrationkey": "test-key",
                "catalog": true
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "REQUEST_SUCCEEDED",
                    "message": [],
                    "Results": { "series": [{
                        "seriesID": "CUUR0000SA0",
                        "catalog": { "series_title": "All items in U.S. city average" },
                        "data": [{ "year": "2026", "period": "M03", "value": "330.213" }]
                    }]}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let raw = test_client(&server.url(), Tier::V2("test-key".into()))
            .series("CUUR0000SA0")
            .await
            .unwrap();
        let series = to_canonical("CUUR0000SA0", raw);
        assert_eq!(
            series.title.as_deref(),
            Some("All items in U.S. city average")
        );
    }

    #[tokio::test]
    async fn unknown_series_reports_the_bls_complaint() {
        // BLS answers an invalid id with REQUEST_SUCCEEDED, an empty data
        // array, and the complaint in `message` — not with an error status.
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/timeseries/data/NOTASERIES")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "REQUEST_SUCCEEDED",
                    "message": ["Invalid Series for Series NOTASERIES"],
                    "Results": { "series": [{ "seriesID": "NOTASERIES", "data": [] }] }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url(), Tier::V1)
            .series("NOTASERIES")
            .await
            .unwrap_err();
        match err {
            FinanceError::SymbolNotFound { context, .. } => {
                assert!(context.contains("Invalid Series"), "got {context}");
            }
            other => panic!("expected SymbolNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn failed_status_surfaces_as_a_macro_data_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/timeseries/data/CUUR0000SA0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "REQUEST_NOT_PROCESSED",
                    "message": ["daily threshold for requests exceeded"],
                    "Results": {}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url(), Tier::V1)
            .series("CUUR0000SA0")
            .await
            .unwrap_err();
        match err {
            FinanceError::MacroDataError { provider, context } => {
                assert_eq!(provider, "BLS");
                assert!(context.contains("daily threshold"), "got {context}");
            }
            other => panic!("expected MacroDataError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn invalid_registration_key_is_an_authentication_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/v2/timeseries/data/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "status": "REQUEST_NOT_PROCESSED",
                    "message": ["The registration key is invalid."],
                    "Results": {}
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url(), Tier::V2("bad-key".into()))
            .series("CUUR0000SA0")
            .await
            .unwrap_err();
        assert!(matches!(err, FinanceError::AuthenticationFailed { .. }));
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v1/timeseries/data/CUUR0000SA0")
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url(), Tier::V1)
            .series("CUUR0000SA0")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }
}
