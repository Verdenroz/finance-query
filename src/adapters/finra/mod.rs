//! FINRA — free daily short-sale volume from the primary source.
//!
//! Requires the **`finra`** feature flag.
//!
//! `Ticker::short_volume()` previously needed Polygon. FINRA publishes the
//! same daily aggregated short volume itself, free and without credentials,
//! through the `otcMarket/regShoDaily` dataset — so routing `FUNDAMENTALS`
//! here gives the method a keyless provider.
//!
//! # Licence
//!
//! FINRA's Query API is free for **non-commercial** use. Commercial use needs
//! an agreement with FINRA; that is a licensing question, not a technical one,
//! and this adapter does not change it.

pub(crate) mod client;
pub(crate) mod fundamentals;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::FinraClient;

/// Self-imposed pacing. FINRA's anonymous tier is metered per day rather than
/// per second; this only keeps a burst from looking abusive.
const FINRA_RATE_PER_SEC: f64 = 2.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = FINRA_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<FinraClient> {
    FinraClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::FINRA_BASE)
}

pub(crate) use fundamentals::fetch_short_volume_response;

#[cfg(test)]
mod tests {
    use super::fundamentals::consolidate;
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> FinraClient {
        FinraClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    /// Verbatim shape from api.finra.org: two trade dates, three reporting
    /// facilities each, deliberately out of date order.
    fn rows_payload() -> String {
        serde_json::json!([
            { "reportingFacilityCode": "NQTRF", "marketCode": "Q",
              "tradeReportDate": "2026-07-29",
              "securitiesInformationProcessorSymbolIdentifier": "AAPL",
              "shortParQuantity": 8104492.271627_f64,
              "shortExemptParQuantity": 43220.0_f64,
              "totalParQuantity": 16890321.3741_f64 },
            { "reportingFacilityCode": "NCTRF", "marketCode": "B",
              "tradeReportDate": "2026-07-28",
              "securitiesInformationProcessorSymbolIdentifier": "AAPL",
              "shortParQuantity": 71287.391347_f64,
              "shortExemptParQuantity": 2.0_f64,
              "totalParQuantity": 139754.385672_f64 },
            { "reportingFacilityCode": "NQTRF", "marketCode": "Q",
              "tradeReportDate": "2026-07-28",
              "securitiesInformationProcessorSymbolIdentifier": "AAPL",
              "shortParQuantity": 8101622.503547_f64,
              "shortExemptParQuantity": 41852.0_f64,
              "totalParQuantity": 17193767.649164_f64 },
            { "reportingFacilityCode": "NYTRF", "marketCode": "N",
              "tradeReportDate": "2026-07-28",
              "securitiesInformationProcessorSymbolIdentifier": "AAPL",
              "shortParQuantity": 1433075.91414_f64,
              "shortExemptParQuantity": 430.0_f64,
              "totalParQuantity": 1756412.25786_f64 }
        ])
        .to_string()
    }

    #[tokio::test]
    async fn per_facility_rows_are_summed_into_one_figure_per_date() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/otcMarket/name/regShoDaily")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(rows_payload())
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .reg_sho_daily("AAPL", "2026-07-28".into(), "2026-07-29".into())
            .await
            .unwrap();
        let series = consolidate(rows);

        // Four raw rows across two dates collapse to two daily figures.
        assert_eq!(series.len(), 2);
        // Chronological despite the payload being out of order — FINRA rejects
        // server-side sorting on a date-range query.
        assert_eq!(series[0].date.as_deref(), Some("2026-07-28"));
        assert_eq!(series[1].date.as_deref(), Some("2026-07-29"));

        let day = &series[0];
        let expected_short = 71287.391347 + 8101622.503547 + 1433075.91414;
        let expected_total = 139754.385672 + 17193767.649164 + 1756412.25786;
        assert!((day.short_volume.unwrap() - expected_short).abs() < 1e-6);
        assert!((day.total_volume.unwrap() - expected_total).abs() < 1e-6);
        assert!((day.short_exempt_volume.unwrap() - (2.0 + 41852.0 + 430.0)).abs() < 1e-9);
    }

    #[test]
    fn a_field_absent_from_every_facility_stays_none() {
        let rows =
            serde_json::from_value::<Vec<super::models::RegShoDailyRow>>(serde_json::json!([
                { "tradeReportDate": "2026-07-28", "shortParQuantity": 10.0_f64 },
                { "tradeReportDate": "2026-07-28", "shortParQuantity": 5.0_f64 }
            ]))
            .unwrap();

        let series = consolidate(rows);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].short_volume, Some(15.0));
        // Nothing reported a total, so it must not become 0.0.
        assert_eq!(series[0].total_volume, None);
    }

    #[tokio::test]
    async fn no_content_means_an_empty_series_not_an_error() {
        // FINRA answers "matched nothing" with 204 and an empty body — a
        // symbol with no reportable short volume, not a failure.
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/otcMarket/name/regShoDaily")
            .with_status(204)
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .reg_sho_daily("NOSUCHSYM", "2026-01-01".into(), "2026-07-30".into())
            .await
            .unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn a_rejected_request_carries_finras_message() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/otcMarket/name/regShoDaily")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "statusCode": 400,
                    "message": "The following fields are not available in this dataset: [nope]"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url())
            .reg_sho_daily("AAPL", "2026-01-01".into(), "2026-07-30".into())
            .await
            .unwrap_err();
        match err {
            FinanceError::ApiError(msg) => {
                assert!(msg.contains("not available in this dataset"), "got {msg}");
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/otcMarket/name/regShoDaily")
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .reg_sho_daily("AAPL", "2026-01-01".into(), "2026-07-30".into())
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }
}
