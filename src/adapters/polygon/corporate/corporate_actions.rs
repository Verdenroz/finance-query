//! Corporate action endpoints: dividends, splits, IPOs, ticker events.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::chart::events::{ChartEvents, DividendEvent, SplitEvent};

use super::super::build_client;
use super::super::models::PaginatedResponseDTO;

/// Dividend event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Dividend {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Cash amount per share.
    pub cash_amount: Option<f64>,
    /// Currency.
    pub currency: Option<String>,
    /// Declaration date.
    pub declaration_date: Option<String>,
    /// Dividend type (e.g., `"CD"` for cash).
    pub dividend_type: Option<String>,
    /// Ex-dividend date.
    pub ex_dividend_date: Option<String>,
    /// Frequency (e.g., `4` for quarterly).
    pub frequency: Option<u32>,
    /// Pay date.
    pub pay_date: Option<String>,
    /// Record date.
    pub record_date: Option<String>,
}

/// Stock split event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Split {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Execution date.
    pub execution_date: Option<String>,
    /// Split from factor.
    pub split_from: Option<f64>,
    /// Split to factor.
    pub split_to: Option<f64>,
}

/// IPO event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)]
pub struct Ipo {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Listing date.
    pub listing_date: Option<String>,
    /// IPO price.
    pub ipo_price: Option<f64>,
    /// Currency.
    pub currency: Option<String>,
    /// Exchange.
    pub primary_exchange: Option<String>,
    /// Share price range low.
    pub lot_size: Option<u64>,
    /// IPO status.
    pub ipo_status: Option<String>,
}

/// Ticker event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)]
pub struct TickerEventDTO {
    /// Event type.
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Event date.
    pub date: Option<String>,
    /// Ticker change details.
    pub ticker_change: Option<serde_json::Value>,
}

/// Ticker events response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)]
pub struct TickerEventsResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Ticker name.
    pub name: Option<String>,
    /// Events list.
    pub events: Option<Vec<TickerEventDTO>>,
}

/// Fetch historical dividends.
pub async fn stock_dividends(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<Dividend>> {
    let client = build_client()?;
    client.get("/v3/reference/dividends", params).await
}

/// Fetch historical stock splits.
pub async fn stock_splits(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<Split>> {
    let client = build_client()?;
    client.get("/v3/reference/splits", params).await
}

/// Fetch IPO data.
#[allow(dead_code)]
pub async fn stock_ipos(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<Ipo>> {
    let client = build_client()?;
    client.get("/v1/reference/ipos", params).await
}

/// Fetch ticker events (name changes, mergers, etc.).
#[allow(dead_code)]
pub async fn stock_ticker_events(ticker: &str) -> Result<TickerEventsResponseDTO> {
    let client = build_client()?;
    let path = format!("/vX/reference/tickers/{}/events", ticker);
    client
        .get_as(&path, &[], "ticker_events", "ticker events")
        .await
}

/// Helper to parse a "YYYY-MM-DD" date string into a Unix timestamp.
fn parse_date(d: &Option<String>) -> i64 {
    d.as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        .and_then(|dt| dt.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0)
}

/// Fetch chart events (dividends + splits, canonical) for a stock ticker.
pub async fn fetch_events_response(symbol: &str) -> Result<ChartEvents> {
    let dividends = stock_dividends(&[("ticker", symbol)]).await?;
    let splits = stock_splits(&[("ticker", symbol)]).await?;

    let mut chart_events = ChartEvents::default();
    chart_events.dividends = dividends
        .results
        .into_iter()
        .flatten()
        .map(|d| {
            let date = parse_date(&d.pay_date);
            (
                date.to_string(),
                DividendEvent {
                    amount: d.cash_amount.unwrap_or(0.0),
                    date,
                },
            )
        })
        .collect();
    chart_events.splits = splits
        .results
        .into_iter()
        .flatten()
        .map(|s| {
            let date = parse_date(&s.execution_date);
            let numerator = s.split_to.unwrap_or(1.0);
            let denominator = s.split_from.unwrap_or(1.0);
            (
                date.to_string(),
                SplitEvent {
                    date,
                    numerator,
                    denominator,
                    split_ratio: format!("{}:{}", numerator, denominator),
                },
            )
        })
        .collect();
    Ok(chart_events)
}
