//! US Treasury FiscalData — keyless federal fiscal statistics.
//!
//! Requires the **`fiscaldata`** feature flag.
//!
//! `api.fiscaldata.treasury.gov` is the primary source for federal debt,
//! interest rates, and daily Treasury statements. FRED mirrors some of it with
//! a lag and needs a key; this route is keyless with no registration.
//!
//! # Series identifiers
//!
//! Curated short names cover the common series (`"DEBT_TO_PENNY"`,
//! `"AVG_INTEREST_RATE"`, …). Anything else is reachable through the
//! passthrough form `"<dataset path>:<value column>"`, e.g.
//! `"v2/accounting/od/debt_to_penny:tot_pub_debt_out_amt"`.

pub(crate) mod client;
pub(crate) mod economic;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::FiscalDataClient;

/// Self-imposed pacing. FiscalData publishes no quota but is a small public
/// service; stay polite.
const FISCALDATA_RATE_PER_SEC: f64 = 5.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = FISCALDATA_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<FiscalDataClient> {
    FiscalDataClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::FISCALDATA_BASE)
}

pub(crate) use economic::fetch_economic_series_response;

#[cfg(test)]
mod tests {
    use super::client::FiscalDataClient;
    use super::economic::{CURATED, Resolved, parse_value, resolve, to_canonical};
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> FiscalDataClient {
        FiscalDataClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn debt_payload() -> String {
        serde_json::json!({
            "data": [
                { "record_date": "2026-07-29", "tot_pub_debt_out_amt": "39799618642771.12" },
                { "record_date": "2026-07-30", "tot_pub_debt_out_amt": "39841114561022.68" },
                { "record_date": "2026-07-31", "tot_pub_debt_out_amt": "null" }
            ],
            "meta": {
                "count": 3,
                "labels": { "tot_pub_debt_out_amt": "Total Public Debt Outstanding" },
                "dataTypes": { "tot_pub_debt_out_amt": "CURRENCY" },
                "total-count": 3,
                "total-pages": 1
            },
            "links": {}
        })
        .to_string()
    }

    #[test]
    fn curated_ids_are_unique_and_uppercase() {
        let mut seen = std::collections::HashSet::new();
        for c in CURATED {
            assert!(seen.insert(c.id), "duplicate curated id {}", c.id);
            assert_eq!(c.id, c.id.to_uppercase(), "{} is not uppercase", c.id);
        }
    }

    #[test]
    fn curated_ids_resolve_case_insensitively() {
        let r = resolve("debt_to_penny").unwrap();
        match r {
            Resolved::Curated(c) => {
                assert_eq!(c.id, "DEBT_TO_PENNY");
                assert_eq!(c.value_field, "tot_pub_debt_out_amt");
            }
            _ => panic!("expected a curated match"),
        }
    }

    #[test]
    fn passthrough_form_splits_dataset_from_column() {
        let r = resolve("v2/accounting/od/debt_to_penny:debt_held_public_amt").unwrap();
        match r {
            Resolved::Passthrough {
                dataset,
                value_field,
            } => {
                assert_eq!(dataset, "v2/accounting/od/debt_to_penny");
                assert_eq!(value_field, "debt_held_public_amt");
            }
            _ => panic!("expected a passthrough"),
        }
    }

    #[test]
    fn unknown_series_id_lists_the_curated_catalogue() {
        let err = resolve("TOTALLY_MADE_UP").unwrap_err();
        match err {
            FinanceError::InvalidParameter { reason, .. } => {
                assert!(reason.contains("DEBT_TO_PENNY"), "got {reason}");
                assert!(reason.contains("passthrough"), "got {reason}");
            }
            other => panic!("expected InvalidParameter, got {other:?}"),
        }
    }

    #[test]
    fn string_encoded_numbers_and_null_sentinels_parse() {
        assert_eq!(parse_value("39841114561022.68"), Some(39841114561022.68));
        assert_eq!(parse_value("-215024135197.77"), Some(-215024135197.77));
        // FiscalData sends a missing figure as the *string* "null".
        assert_eq!(parse_value("null"), None);
        assert_eq!(parse_value(""), None);
        assert_eq!(parse_value("  "), None);
        assert_eq!(parse_value("not a number"), None);
    }

    #[tokio::test]
    async fn dataset_rows_map_to_canonical_series() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(debt_payload())
            .create_async()
            .await;

        let resolved = resolve("DEBT_TO_PENNY").unwrap();
        let (rows, meta) = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap();
        let series = to_canonical("DEBT_TO_PENNY", &resolved, rows, &meta);

        assert_eq!(series.series_id, "DEBT_TO_PENNY");
        assert_eq!(
            series.title.as_deref(),
            Some("Total Public Debt Outstanding")
        );
        // Curated units beat the CURRENCY data type, which can't tell dollars
        // from millions of dollars.
        assert_eq!(series.units.as_deref(), Some("US Dollars"));
        assert_eq!(series.frequency.as_deref(), Some("Daily"));
        assert_eq!(series.observations.len(), 3);
        assert_eq!(series.observations[0].date, "2026-07-29");
        assert_eq!(series.observations[0].value, Some(39799618642771.12));
        assert_eq!(series.observations[2].value, None);
    }

    #[tokio::test]
    async fn filtered_series_sends_the_curated_filter() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/accounting/od/avg_interest_rates")
            .match_query(mockito::Matcher::UrlEncoded(
                "filter".into(),
                "security_desc:eq:Total Interest-bearing Debt".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": [{ "record_date": "2026-06-30", "avg_interest_rate_amt": "3.383" }],
                    "meta": {
                        "labels": { "avg_interest_rate_amt": "Average Interest Rate Amount" },
                        "dataTypes": { "avg_interest_rate_amt": "PERCENTAGE" },
                        "total-pages": 1
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let resolved = resolve("AVG_INTEREST_RATE").unwrap();
        let (rows, meta) = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap();
        let series = to_canonical("AVG_INTEREST_RATE", &resolved, rows, &meta);
        assert_eq!(series.units.as_deref(), Some("Percent"));
        assert_eq!(series.observations[0].value, Some(3.383));
    }

    #[tokio::test]
    async fn passthrough_units_fall_back_to_the_column_type() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(debt_payload())
            .create_async()
            .await;

        let id = "v2/accounting/od/debt_to_penny:tot_pub_debt_out_amt";
        let resolved = resolve(id).unwrap();
        let (rows, meta) = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap();
        let series = to_canonical(id, &resolved, rows, &meta);
        assert_eq!(series.units.as_deref(), Some("US Dollars"));
        // No curated entry means no frequency claim rather than a guessed one.
        assert_eq!(series.frequency, None);
    }

    #[tokio::test]
    async fn query_error_surfaces_the_api_message() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "error": "Invalid Query Param",
                    "message": "Field 'nope' does not exist."
                })
                .to_string(),
            )
            .create_async()
            .await;

        let resolved = resolve("v2/accounting/od/debt_to_penny:nope").unwrap();
        let err = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap_err();
        match err {
            FinanceError::MacroDataError { provider, context } => {
                assert_eq!(provider, "US Treasury FiscalData");
                assert!(context.contains("does not exist"), "got {context}");
            }
            other => panic!("expected MacroDataError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_data_array_is_a_missing_symbol() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "data": [], "meta": { "total-pages": 0 } }).to_string())
            .create_async()
            .await;

        let resolved = resolve("DEBT_TO_PENNY").unwrap();
        let err = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn multi_page_series_is_followed_to_the_end() {
        let mut server = mockito::Server::new_async().await;
        let _p1 = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::UrlEncoded(
                "page[number]".into(),
                "1".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": [{ "record_date": "2026-07-29", "tot_pub_debt_out_amt": "1.0" }],
                    "meta": { "labels": {}, "dataTypes": {}, "total-pages": 2 }
                })
                .to_string(),
            )
            .create_async()
            .await;
        let _p2 = server
            .mock("GET", "/v2/accounting/od/debt_to_penny")
            .match_query(mockito::Matcher::UrlEncoded(
                "page[number]".into(),
                "2".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": [{ "record_date": "2026-07-30", "tot_pub_debt_out_amt": "2.0" }],
                    "meta": { "labels": {}, "dataTypes": {}, "total-pages": 2 }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let resolved = resolve("DEBT_TO_PENNY").unwrap();
        let (rows, _) = test_client(&server.url())
            .series(&resolved.query())
            .await
            .unwrap();
        assert_eq!(rows.len(), 2, "pagination stopped after the first page");
    }
}
