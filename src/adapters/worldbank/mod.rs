//! World Bank Open Data — keyless global development and macro indicators.
//!
//! Requires the **`worldbank`** feature flag.
//!
//! Complements FRED (US-centric, keyed) with ~1,600 indicators across 200+
//! economies. No API key and no registration; the client paces itself since
//! the World Bank publishes no documented quota.
//!
//! # Series identifiers
//!
//! The `ECONOMIC` capability takes a single string, so a World Bank series is
//! addressed as `"<COUNTRY>/<INDICATOR>"` — for example
//! `"USA/NY.GDP.MKTP.CD"` (US GDP in current US$). A bare indicator resolves
//! against the world aggregate `WLD`.

pub(crate) mod client;
pub(crate) mod economic;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::WorldBankClient;

/// Self-imposed pacing. The World Bank documents no quota but does throttle
/// abusive clients, so stay well under a burst.
const WORLDBANK_RATE_PER_SEC: f64 = 5.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = WORLDBANK_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<WorldBankClient> {
    WorldBankClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::WORLDBANK_BASE)
}

pub(crate) use economic::fetch_economic_series_response;

#[cfg(test)]
mod tests {
    use super::economic::{normalize_date, split_series_id, to_canonical};
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> WorldBankClient {
        WorldBankClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn payload() -> String {
        serde_json::json!([
            { "page": 1, "pages": 1, "per_page": 20000, "total": 3 },
            [
                {
                    "indicator": { "id": "NY.GDP.MKTP.CD", "value": "GDP (current US$)" },
                    "country": { "id": "US", "value": "United States" },
                    "countryiso3code": "USA",
                    "date": "2023",
                    "value": 27_720_700_000_000.0_f64,
                    "unit": "",
                    "obs_status": "",
                    "decimal": 0
                },
                {
                    "indicator": { "id": "NY.GDP.MKTP.CD", "value": "GDP (current US$)" },
                    "country": { "id": "US", "value": "United States" },
                    "countryiso3code": "USA",
                    "date": "2022",
                    "value": 25_744_100_000_000.0_f64,
                    "unit": "",
                    "obs_status": "",
                    "decimal": 0
                },
                {
                    "indicator": { "id": "NY.GDP.MKTP.CD", "value": "GDP (current US$)" },
                    "country": { "id": "US", "value": "United States" },
                    "countryiso3code": "USA",
                    "date": "2021",
                    "value": null,
                    "unit": "",
                    "obs_status": "",
                    "decimal": 0
                }
            ]
        ])
        .to_string()
    }

    #[test]
    fn series_id_splits_country_from_indicator() {
        assert_eq!(
            split_series_id("USA/NY.GDP.MKTP.CD"),
            ("USA".to_string(), "NY.GDP.MKTP.CD".to_string())
        );
        assert_eq!(
            split_series_id("br/SP.POP.TOTL"),
            ("BR".to_string(), "SP.POP.TOTL".to_string())
        );
    }

    #[test]
    fn bare_indicator_defaults_to_world_aggregate() {
        assert_eq!(
            split_series_id("NY.GDP.MKTP.CD"),
            ("WLD".to_string(), "NY.GDP.MKTP.CD".to_string())
        );
    }

    #[test]
    fn periods_normalize_to_period_start_dates() {
        assert_eq!(normalize_date("2023"), "2023-01-01");
        assert_eq!(normalize_date("2023Q1"), "2023-01-01");
        assert_eq!(normalize_date("2023Q4"), "2023-10-01");
        assert_eq!(normalize_date("2023M04"), "2023-04-01");
        // Unrecognised labels survive rather than being silently dropped.
        assert_eq!(normalize_date("FY2023"), "FY2023");
    }

    #[tokio::test]
    async fn indicator_response_maps_to_canonical_series() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/country/USA/indicator/NY.GDP.MKTP.CD")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(payload())
            .create_async()
            .await;

        let raw = test_client(&server.url())
            .indicator("USA", "NY.GDP.MKTP.CD")
            .await
            .unwrap();
        let series = to_canonical("USA/NY.GDP.MKTP.CD", raw);

        assert_eq!(series.series_id, "USA/NY.GDP.MKTP.CD");
        assert_eq!(
            series.title.as_deref(),
            Some("GDP (current US$) — United States")
        );
        assert_eq!(series.frequency.as_deref(), Some("Annual"));
        // Empty `unit` strings must not become an empty-string unit.
        assert_eq!(series.units, None);
        assert_eq!(series.observations.len(), 3);
        // Reversed into chronological order.
        assert_eq!(series.observations[0].date, "2021-01-01");
        assert_eq!(series.observations[0].value, None);
        assert_eq!(series.observations[2].date, "2023-01-01");
        assert_eq!(series.observations[2].value, Some(27_720_700_000_000.0));
    }

    #[tokio::test]
    async fn rejected_request_surfaces_the_api_message() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/country/USA/indicator/NOT.A.REAL.CODE")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!([{
                    "message": [{
                        "id": "120",
                        "key": "Invalid value",
                        "value": "The provided parameter value is not valid"
                    }]
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url())
            .indicator("USA", "NOT.A.REAL.CODE")
            .await
            .unwrap_err();
        match err {
            FinanceError::MacroDataError { provider, context } => {
                assert_eq!(provider, "World Bank");
                assert!(context.contains("Invalid value"), "got {context}");
            }
            other => panic!("expected MacroDataError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_observation_list_is_a_missing_symbol() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/country/ZZZ/indicator/NY.GDP.MKTP.CD")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!([{ "page": 1, "pages": 0, "per_page": 20000, "total": 0 }, null])
                    .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url())
            .indicator("ZZZ", "NY.GDP.MKTP.CD")
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
            .mock("GET", "/country/USA/indicator/NY.GDP.MKTP.CD")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .indicator("USA", "NY.GDP.MKTP.CD")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }
}
