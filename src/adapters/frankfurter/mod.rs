//! Frankfurter — keyless ECB reference exchange rates.
//!
//! Requires the **`frankfurter`** feature flag.
//!
//! `FOREX` was the one capability with no keyless route at all: Polygon, FMP,
//! and Alpha Vantage each need a key, so `providers.forex()` was unusable out
//! of the box. Frankfurter closes that gap with the European Central Bank's
//! published reference rates.
//!
//! # What this is and is not
//!
//! ECB reference rates are a **daily fix** published around 16:00 CET on
//! TARGET working days — not a live market feed, and not a tradable two-way
//! price. Quotes therefore carry a price with no bid/ask, and the timestamp is
//! the publication date. Route `FOREX` to a keyed provider first if you need
//! intraday rates.

pub(crate) mod client;
pub(crate) mod forex;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::FrankfurterClient;

/// Self-imposed pacing. Frankfurter is free and unmetered but community-run;
/// stay polite.
const FRANKFURTER_RATE_PER_SEC: f64 = 5.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = FRANKFURTER_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<FrankfurterClient> {
    FrankfurterClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::FRANKFURTER_BASE)
}

pub(crate) use forex::fetch_forex_quote_response;

#[cfg(test)]
mod tests {
    use super::forex::{date_to_timestamp, to_quote};
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> FrankfurterClient {
        FrankfurterClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    fn series_payload() -> String {
        serde_json::json!({
            "amount": 1.0,
            "base": "USD",
            "start_date": "2026-07-29",
            "end_date": "2026-07-31",
            "rates": {
                "2026-07-31": { "EUR": 0.8707 },
                "2026-07-29": { "EUR": 0.87873 },
                "2026-07-30": { "EUR": 0.87138 }
            }
        })
        .to_string()
    }

    #[test]
    fn iso_dates_become_midnight_utc() {
        // 2026-07-31T00:00:00Z
        assert_eq!(date_to_timestamp("2026-07-31"), Some(1_785_456_000));
        assert_eq!(date_to_timestamp("not a date"), None);
    }

    #[test]
    fn quote_uses_the_latest_rate_and_the_prior_close_for_change() {
        let series = vec![
            ("2026-07-29".to_string(), 0.87873),
            ("2026-07-30".to_string(), 0.87138),
            ("2026-07-31".to_string(), 0.8707),
        ];
        let q = to_quote("USD", "EUR", &series).unwrap();

        assert_eq!(q.symbol, "USDEUR");
        assert_eq!(q.base_currency.as_deref(), Some("USD"));
        assert_eq!(q.quote_currency.as_deref(), Some("EUR"));
        assert_eq!(q.price, Some(0.8707));
        // ECB publishes a reference fix, not a two-way price.
        assert_eq!(q.bid, None);
        assert_eq!(q.ask, None);
        let change = q.change.unwrap();
        assert!((change - (0.8707 - 0.87138)).abs() < 1e-12, "{change}");
        let pct = q.change_percent.unwrap();
        assert!(
            (pct - ((0.8707 - 0.87138) / 0.87138 * 100.0)).abs() < 1e-9,
            "{pct}"
        );
        assert_eq!(q.timestamp, date_to_timestamp("2026-07-31"));
    }

    #[test]
    fn a_single_published_day_yields_no_change() {
        let series = vec![("2026-07-31".to_string(), 0.8707)];
        let q = to_quote("USD", "EUR", &series).unwrap();
        assert_eq!(q.price, Some(0.8707));
        assert_eq!(q.change, None);
        assert_eq!(q.change_percent, None);
    }

    #[test]
    fn an_empty_series_yields_no_quote() {
        assert!(to_quote("USD", "EUR", &[]).is_none());
    }

    #[tokio::test]
    async fn rates_come_back_in_chronological_order() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/\d{4}-\d{2}-\d{2}\.\.$".into()),
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("base".into(), "USD".into()),
                mockito::Matcher::UrlEncoded("symbols".into(), "EUR".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(series_payload())
            .create_async()
            .await;

        let series = test_client(&server.url())
            .recent_rates("USD", "EUR")
            .await
            .unwrap();
        // The response object is unordered; dates must still come out sorted.
        assert_eq!(
            series.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>(),
            ["2026-07-29", "2026-07-30", "2026-07-31"]
        );
    }

    #[tokio::test]
    async fn unknown_currency_maps_to_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "message": "not found" }).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url())
            .recent_rates("USD", "ZZZ")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn invalid_pair_maps_to_invalid_parameter() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({ "message": "bad currency pair" }).to_string())
            .create_async()
            .await;

        let err = test_client(&server.url())
            .recent_rates("USD", "USD")
            .await
            .unwrap_err();
        match err {
            FinanceError::InvalidParameter { reason, .. } => {
                assert!(reason.contains("bad currency pair"), "got {reason}");
            }
            other => panic!("expected InvalidParameter, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn identical_currencies_short_circuit_without_a_request() {
        // Frankfurter answers USD->USD with HTTP 422; the trivially correct
        // answer is served locally instead. No mock server means any request
        // would fail the test.
        let q = fetch_forex_quote_response("usd", "USD").await.unwrap();
        assert_eq!(q.symbol, "USDUSD");
        assert_eq!(q.price, Some(1.0));
        assert_eq!(q.change, Some(0.0));
    }
}
