//! Nasdaq calendar HTTP client.
//!
//! Bot-detection on `api.nasdaq.com` drops any request whose `User-Agent`
//! doesn't look like a browser — even a self-identifying agent like
//! `finance-query/x.y.z` gets the connection reset, confirmed against the
//! live API. This client sends a plain browser-shaped agent instead of
//! `adapters::common::keyless_http_client`'s default.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;

use super::models::{
    NasdaqDividendRow, NasdaqDividendsEnvelope, NasdaqEarningsEnvelope, NasdaqEarningsRow,
    NasdaqIpoEnvelope, NasdaqIpoRow, NasdaqSplitRow, NasdaqSplitsEnvelope,
};
use crate::adapters::common::check_status;
use crate::error::{FinanceError, Result};
use crate::rate_limiter::RateLimiter;

pub(super) const NASDAQ_BASE: &str = "https://api.nasdaq.com/api";

const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/120.0 Safari/537.36";

pub(super) struct NasdaqClient {
    http: Client,
    limiter: Arc<RateLimiter>,
    base_url: String,
}

impl NasdaqClient {
    pub(super) fn new(
        timeout: Duration,
        limiter: Arc<RateLimiter>,
        base_url: impl Into<String>,
    ) -> Result<Self> {
        let http = Client::builder()
            .timeout(timeout)
            .user_agent(BROWSER_USER_AGENT)
            .build()?;
        Ok(Self {
            http,
            limiter,
            base_url: base_url.into(),
        })
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        self.limiter.acquire().await;
        let url = format!("{}/{path}", self.base_url);
        let resp = self.http.get(&url).query(query).send().await?;
        check_status("Nasdaq", resp.status())?;
        let bytes = resp.bytes().await?;
        serde_json::from_slice(&bytes).map_err(|e| FinanceError::ResponseStructureError {
            field: "nasdaq.response".to_string(),
            context: format!("unrecognised Nasdaq envelope: {e}"),
        })
    }

    /// Earnings scheduled to report on `date` (`YYYY-MM-DD`).
    pub(super) async fn earnings_for_date(&self, date: &str) -> Result<Vec<NasdaqEarningsRow>> {
        let envelope: NasdaqEarningsEnvelope = self
            .get_json("calendar/earnings", &[("date", date)])
            .await?;
        Ok(envelope.data.and_then(|d| d.rows).unwrap_or_default())
    }

    /// Dividends going ex-date on `date` (`YYYY-MM-DD`).
    pub(super) async fn dividends_for_date(&self, date: &str) -> Result<Vec<NasdaqDividendRow>> {
        let envelope: NasdaqDividendsEnvelope = self
            .get_json("calendar/dividends", &[("date", date)])
            .await?;
        Ok(envelope
            .data
            .and_then(|d| d.calendar)
            .and_then(|c| c.rows)
            .unwrap_or_default())
    }

    /// Splits taking effect on `date` (`YYYY-MM-DD`).
    pub(super) async fn splits_for_date(&self, date: &str) -> Result<Vec<NasdaqSplitRow>> {
        let envelope: NasdaqSplitsEnvelope =
            self.get_json("calendar/splits", &[("date", date)]).await?;
        Ok(envelope.data.and_then(|d| d.rows).unwrap_or_default())
    }

    /// Priced, upcoming, filed, and withdrawn IPOs for `month` (`YYYY-MM`),
    /// each tagged with the deal-status label they were fetched under.
    pub(super) async fn ipos_for_month(
        &self,
        month: &str,
    ) -> Result<Vec<(&'static str, NasdaqIpoRow)>> {
        let envelope: NasdaqIpoEnvelope = self.get_json("ipo/calendar", &[("date", month)]).await?;
        let Some(data) = envelope.data else {
            return Ok(Vec::new());
        };

        let mut out = Vec::new();
        out.extend(
            data.priced
                .and_then(|r| r.rows)
                .unwrap_or_default()
                .into_iter()
                .map(|r| ("priced", r)),
        );
        out.extend(
            data.upcoming
                .map(|u| u.upcoming_table)
                .and_then(|r| r.rows)
                .unwrap_or_default()
                .into_iter()
                .map(|r| ("expected", r)),
        );
        out.extend(
            data.filed
                .and_then(|r| r.rows)
                .unwrap_or_default()
                .into_iter()
                .map(|r| ("filed", r)),
        );
        out.extend(
            data.withdrawn
                .and_then(|r| r.rows)
                .unwrap_or_default()
                .into_iter()
                .map(|r| ("withdrawn", r)),
        );
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limiter::RateLimiter;

    fn test_client(base_url: &str) -> NasdaqClient {
        NasdaqClient::new(
            Duration::from_secs(5),
            Arc::new(RateLimiter::new(100.0)),
            base_url,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn earnings_for_date_parses_rows() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/calendar/earnings")
            .match_query(mockito::Matcher::UrlEncoded(
                "date".into(),
                "2026-08-25".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": {
                        "rows": [{
                            "symbol": "BMO",
                            "time": "time-pre-market",
                            "fiscalQuarterEnding": "Jul/2026",
                            "epsForecast": "$2.72"
                        }]
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .earnings_for_date("2026-08-25")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].symbol.as_deref(), Some("BMO"));
        assert_eq!(rows[0].eps_forecast.as_deref(), Some("$2.72"));
    }

    #[tokio::test]
    async fn dividends_for_date_reads_the_nested_calendar_key() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/calendar/dividends")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": {
                        "calendar": {
                            "rows": [{
                                "symbol": "CDW",
                                "dividend_Ex_Date": "8/25/2026",
                                "dividend_Rate": 0.63
                            }]
                        }
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .dividends_for_date("2026-08-25")
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].dividend_rate, Some(0.63));
    }

    #[tokio::test]
    async fn splits_for_date_returns_empty_when_no_rows() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/calendar/splits")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::json!({"data": {"rows": null}}).to_string())
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .splits_for_date("2026-08-25")
            .await
            .unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn ipos_for_month_tags_each_row_by_deal_status() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/ipo/calendar")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": {
                        "priced": {"rows": [{"proposedTickerSymbol": "LYNX", "pricedDate": "8/19/2026"}]},
                        "upcoming": {"upcomingTable": {"rows": [{"proposedTickerSymbol": "PTT", "expectedPriceDate": "9/08/2026"}]}},
                        "filed": {"rows": []},
                        "withdrawn": {"rows": []}
                    }
                })
                .to_string(),
            )
            .create_async()
            .await;

        let rows = test_client(&server.url())
            .ipos_for_month("2026-08")
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, "priced");
        assert_eq!(rows[0].1.proposed_ticker_symbol.as_deref(), Some("LYNX"));
        assert_eq!(rows[1].0, "expected");
        assert_eq!(rows[1].1.proposed_ticker_symbol.as_deref(), Some("PTT"));
    }

    #[tokio::test]
    async fn non_2xx_status_maps_to_external_api_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", "/calendar/earnings")
            .match_query(mockito::Matcher::Any)
            .with_status(503)
            .create_async()
            .await;

        let err = test_client(&server.url())
            .earnings_for_date("2026-08-25")
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            FinanceError::ExternalApiError { status: 503, .. }
        ));
    }
}
