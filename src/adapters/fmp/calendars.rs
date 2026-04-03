//! Calendar endpoints: earnings, IPO, stock split, dividend, and economic calendars.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Earnings calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsCalendarEntry {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Estimated EPS.
    pub eps: Option<f64>,
    /// Estimated EPS.
    #[serde(rename = "epsEstimated")]
    pub eps_estimated: Option<f64>,
    /// Time of announcement (bmo = before market open, amc = after market close).
    pub time: Option<String>,
    /// Revenue.
    pub revenue: Option<f64>,
    /// Estimated revenue.
    #[serde(rename = "revenueEstimated")]
    pub revenue_estimated: Option<f64>,
    /// Fiscal date ending.
    #[serde(rename = "fiscalDateEnding")]
    pub fiscal_date_ending: Option<String>,
    /// Updated from date.
    #[serde(rename = "updatedFromDate")]
    pub updated_from_date: Option<String>,
}

/// IPO calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IpoCalendarEntry {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Company name.
    pub company: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Number of actions (shares offered).
    pub actions: Option<String>,
    /// Shares offered.
    pub shares: Option<f64>,
    /// Price range.
    #[serde(rename = "priceRange")]
    pub price_range: Option<String>,
    /// Market cap.
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
}

/// Stock split calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockSplitCalendarEntry {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Numerator.
    pub numerator: Option<f64>,
    /// Denominator.
    pub denominator: Option<f64>,
}

/// Dividend calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DividendCalendarEntry {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Dividend amount.
    pub dividend: Option<f64>,
    /// Adjusted dividend.
    #[serde(rename = "adjDividend")]
    pub adj_dividend: Option<f64>,
    /// Record date.
    #[serde(rename = "recordDate")]
    pub record_date: Option<String>,
    /// Payment date.
    #[serde(rename = "paymentDate")]
    pub payment_date: Option<String>,
    /// Declaration date.
    #[serde(rename = "declarationDate")]
    pub declaration_date: Option<String>,
}

/// Economic calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicCalendarEntry {
    /// Event name.
    pub event: Option<String>,
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Country.
    pub country: Option<String>,
    /// Actual value.
    pub actual: Option<f64>,
    /// Previous value.
    pub previous: Option<f64>,
    /// Change value.
    pub change: Option<f64>,
    /// Change percentage.
    #[serde(rename = "changePercentage")]
    pub change_percentage: Option<f64>,
    /// Estimate.
    pub estimate: Option<f64>,
    /// Impact level.
    pub impact: Option<String>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch earnings calendar within a date range.
///
/// * `from` - Start date (YYYY-MM-DD)
/// * `to` - End date (YYYY-MM-DD)
pub async fn earnings_calendar(from: &str, to: &str) -> Result<Vec<EarningsCalendarEntry>> {
    let client = build_client()?;
    client
        .get("/api/v3/earning_calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch IPO calendar within a date range.
pub async fn ipo_calendar(from: &str, to: &str) -> Result<Vec<IpoCalendarEntry>> {
    let client = build_client()?;
    client
        .get("/api/v3/ipo_calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch stock split calendar within a date range.
pub async fn stock_split_calendar(
    from: &str,
    to: &str,
) -> Result<Vec<StockSplitCalendarEntry>> {
    let client = build_client()?;
    client
        .get(
            "/api/v3/stock_split_calendar",
            &[("from", from), ("to", to)],
        )
        .await
}

/// Fetch dividend calendar within a date range.
pub async fn dividend_calendar(from: &str, to: &str) -> Result<Vec<DividendCalendarEntry>> {
    let client = build_client()?;
    client
        .get(
            "/api/v3/stock_dividend_calendar",
            &[("from", from), ("to", to)],
        )
        .await
}

/// Fetch economic calendar within a date range.
pub async fn economic_calendar(from: &str, to: &str) -> Result<Vec<EconomicCalendarEntry>> {
    let client = build_client()?;
    client
        .get(
            "/api/v3/economic_calendar",
            &[("from", from), ("to", to)],
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_earnings_calendar_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/earning_calendar")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("from".into(), "2024-01-01".into()),
                mockito::Matcher::UrlEncoded("to".into(), "2024-01-31".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "date": "2024-01-25",
                        "symbol": "MSFT",
                        "eps": 2.93,
                        "epsEstimated": 2.78,
                        "time": "amc",
                        "revenue": 62020000000.0,
                        "revenueEstimated": 61100000000.0
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EarningsCalendarEntry> = client
            .get(
                "/api/v3/earning_calendar",
                &[("from", "2024-01-01"), ("to", "2024-01-31")],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].symbol.as_deref(), Some("MSFT"));
        assert!((resp[0].eps.unwrap() - 2.93).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_economic_calendar_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/economic_calendar")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("from".into(), "2024-01-01".into()),
                mockito::Matcher::UrlEncoded("to".into(), "2024-01-31".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "event": "CPI",
                        "date": "2024-01-11",
                        "country": "US",
                        "actual": 3.4,
                        "previous": 3.1,
                        "estimate": 3.2,
                        "impact": "High"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EconomicCalendarEntry> = client
            .get(
                "/api/v3/economic_calendar",
                &[("from", "2024-01-01"), ("to", "2024-01-31")],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].event.as_deref(), Some("CPI"));
    }
}
