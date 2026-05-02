//! Corporate action endpoints: dividends, splits, IPOs, ticker events.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::build_client;
use super::super::models::PaginatedResponse;

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
pub struct TickerEvent {
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
pub struct TickerEventsResponse {
    /// Request ID.
    pub request_id: Option<String>,
    /// Response status.
    pub status: Option<String>,
    /// Ticker name.
    pub name: Option<String>,
    /// Events list.
    pub events: Option<Vec<TickerEvent>>,
}

/// Fetch historical dividends.
pub async fn stock_dividends(params: &[(&str, &str)]) -> Result<PaginatedResponse<Dividend>> {
    let client = build_client()?;
    client.get("/v3/reference/dividends", params).await
}

/// Fetch historical stock splits.
pub async fn stock_splits(params: &[(&str, &str)]) -> Result<PaginatedResponse<Split>> {
    let client = build_client()?;
    client.get("/v3/reference/splits", params).await
}

/// Fetch IPO data.
pub async fn stock_ipos(params: &[(&str, &str)]) -> Result<PaginatedResponse<Ipo>> {
    let client = build_client()?;
    client.get("/v1/reference/ipos", params).await
}

/// Fetch ticker events (name changes, mergers, etc.).
pub async fn stock_ticker_events(ticker: &str) -> Result<TickerEventsResponse> {
    let client = build_client()?;
    let path = format!("/vX/reference/tickers/{}/events", ticker);
    let json = client.get_raw(&path, &[]).await?;
    serde_json::from_value(json).map_err(|e| crate::error::FinanceError::ResponseStructureError {
        field: "ticker_events".to_string(),
        context: format!("Failed to parse ticker events: {e}"),
    })
}
