//! CFTC Commitments of Traders — keyless weekly futures positioning.
//!
//! Requires the **`cftc`** feature flag.
//!
//! Nothing in the library exposed positioning data (long/short/spread by
//! trader category) before this adapter; `Capability::FUTURES` previously
//! meant Polygon-only price quotes. The CFTC publishes its weekly
//! Commitments of Traders reports itself, keylessly, through
//! `publicreporting.cftc.gov`'s Socrata API — this adapter serves the
//! disaggregated futures-only combined report, the one most commonly meant
//! by "COT data" for physical commodities.
//!
//! # Scope
//!
//! Only physical-commodity futures are covered (agriculture, energy,
//! metals) via a curated table of benchmark contracts, or any raw
//! `cftc_contract_market_code` passed straight through. Financial futures (equity indices, rates,
//! currencies) are reported separately by the CFTC in the Traders in
//! Financial Futures report, which this adapter does not serve — the CFTC
//! itself has no price-quote data at all, so [`Provider::Cftc`](crate::Provider::Cftc)
//! only ever answers [`FuturesContract::commitments_of_traders`](crate::FuturesContract::commitments_of_traders),
//! reporting `NotSupported` for a plain quote.

pub(crate) mod client;
pub(crate) mod futures;
pub(crate) mod models;

use std::time::Duration;

use crate::adapters::singleton::keyless_limiter;
use crate::error::Result;
use client::CftcClient;

/// Self-imposed pacing. Socrata's anonymous tier throttles by rolling-hour
/// count rather than per-second; this only keeps a burst from looking
/// abusive.
const CFTC_RATE_PER_SEC: f64 = 5.0;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

keyless_limiter!(rate = CFTC_RATE_PER_SEC);

/// Build a client against the live API, reusing the shared token bucket.
fn client() -> Result<CftcClient> {
    CftcClient::new(DEFAULT_TIMEOUT, shared_limiter(), client::CFTC_BASE)
}

pub(crate) use futures::fetch_commitments_of_traders_response;

#[cfg(test)]
mod tests {
    use super::futures::{resolve_contract_code, to_canonical};
    use super::*;
    use crate::error::FinanceError;
    use crate::rate_limiter::RateLimiter;
    use std::sync::Arc;

    fn test_client(base_url: &str) -> CftcClient {
        CftcClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    /// Verbatim shape (trimmed to the mapped columns) from
    /// `publicreporting.cftc.gov/resource/72hh-3qpy.json`, three weekly gold
    /// rows, newest first.
    fn rows_payload() -> String {
        serde_json::json!([
            {
                "market_and_exchange_names": "GOLD - COMMODITY EXCHANGE INC.",
                "cftc_contract_market_code": "088691",
                "report_date_as_yyyy_mm_dd": "2026-07-28T00:00:00.000",
                "open_interest_all": "384603",
                "prod_merc_positions_long": "15367",
                "prod_merc_positions_short": "35916",
                "swap_positions_long_all": "23661",
                "swap__positions_short_all": "215421",
                "swap__positions_spread_all": "36432",
                "m_money_positions_long_all": "135093",
                "m_money_positions_short_all": "15298",
                "m_money_positions_spread": "18384",
                "other_rept_positions_long": "84529",
                "other_rept_positions_short": "22254",
                "other_rept_positions_spread": "9753",
                "tot_rept_positions_long_all": "323219",
                "tot_rept_positions_short": "353458",
                "nonrept_positions_long_all": "61384",
                "nonrept_positions_short_all": "31145"
            },
            {
                "market_and_exchange_names": "GOLD - COMMODITY EXCHANGE INC.",
                "cftc_contract_market_code": "088691",
                "report_date_as_yyyy_mm_dd": "2026-07-21T00:00:00.000",
                "open_interest_all": "383368",
                "prod_merc_positions_long": "15561",
                "prod_merc_positions_short": "34882",
                "swap_positions_long_all": "24959",
                "swap__positions_short_all": "218837",
                "swap__positions_spread_all": "39937",
                "m_money_positions_long_all": "141487",
                "m_money_positions_short_all": "16656",
                "m_money_positions_spread": "17872",
                "other_rept_positions_long": "83298",
                "other_rept_positions_short": "24219",
                "other_rept_positions_spread": "14111",
                "tot_rept_positions_long_all": "337225",
                "tot_rept_positions_short": "366514",
                "nonrept_positions_long_all": "46143",
                "nonrept_positions_short_all": "16854"
            }
        ])
        .to_string()
    }

    #[tokio::test]
    async fn gold_rows_map_to_a_chronological_canonical_series() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(rows_payload())
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .commitments_of_traders(&resolve_contract_code("GC=F"))
            .await
            .unwrap();
        let series = to_canonical("GC=F", rows).unwrap();

        assert_eq!(series.symbol, "GC=F");
        assert_eq!(
            series.market_and_exchange_name,
            "GOLD - COMMODITY EXCHANGE INC."
        );
        assert_eq!(series.cftc_contract_market_code, "088691");
        assert_eq!(series.observations.len(), 2);
        // Reversed into chronological order despite the API's newest-first payload.
        assert_eq!(series.observations[0].report_date, "2026-07-21");
        assert_eq!(series.observations[1].report_date, "2026-07-28");
        assert_eq!(series.observations[1].open_interest, Some(384603));
        assert_eq!(series.observations[1].managed_money_long, Some(135093));
        assert_eq!(series.observations[1].managed_money_short, Some(15298));
        assert_eq!(series.observations[1].swap_dealer_short, Some(215421));
    }

    #[tokio::test]
    async fn unknown_contract_code_maps_to_symbol_not_found() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .commitments_of_traders("999999")
            .await
            .unwrap();
        let err = to_canonical("999999", rows).unwrap_err();
        assert!(
            matches!(err, FinanceError::SymbolNotFound { .. }),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn a_rejected_query_carries_cftcs_message() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "message": "Query coordinator error: query.soql.no-such-column"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let err = test_client(&server.url())
            .commitments_of_traders("088691")
            .await
            .unwrap_err();
        match err {
            FinanceError::ApiError(msg) => {
                assert!(msg.contains("no-such-column"), "got {msg}");
            }
            other => panic!("expected ApiError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn http_error_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .commitments_of_traders("088691")
            .await
            .unwrap_err();
        assert!(
            matches!(err, FinanceError::ExternalApiError { status: 503, .. }),
            "{err:?}"
        );
    }
}
