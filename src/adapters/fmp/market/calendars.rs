//! Calendar endpoints: earnings, IPO, stock split, dividend, and economic calendars.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Earnings calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EarningsCalendarEntryDTO {
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Reported EPS, once the quarter has been announced.
    #[serde(rename = "epsActual")]
    pub eps: Option<f64>,
    /// Estimated EPS.
    #[serde(rename = "epsEstimated")]
    pub eps_estimated: Option<f64>,
    /// Reported revenue, once the quarter has been announced.
    #[serde(rename = "revenueActual")]
    pub revenue: Option<f64>,
    /// Estimated revenue.
    #[serde(rename = "revenueEstimated")]
    pub revenue_estimated: Option<f64>,
    /// Date the entry was last revised.
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

/// IPO calendar entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IpoCalendarEntryDTO {
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
pub struct StockSplitCalendarEntryDTO {
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
pub struct DividendCalendarEntryDTO {
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
pub struct EconomicCalendarEntryDTO {
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
pub async fn earnings_calendar(from: &str, to: &str) -> Result<Vec<EarningsCalendarEntryDTO>> {
    let client = build_client()?;
    client
        .get("/stable/earnings-calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch IPO calendar within a date range.
pub async fn ipo_calendar(from: &str, to: &str) -> Result<Vec<IpoCalendarEntryDTO>> {
    let client = build_client()?;
    client
        .get("/stable/ipos-calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch stock split calendar within a date range.
pub async fn stock_split_calendar(from: &str, to: &str) -> Result<Vec<StockSplitCalendarEntryDTO>> {
    let client = build_client()?;
    client
        .get("/stable/splits-calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch dividend calendar within a date range.
pub async fn dividend_calendar(from: &str, to: &str) -> Result<Vec<DividendCalendarEntryDTO>> {
    let client = build_client()?;
    client
        .get("/stable/dividends-calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch economic calendar within a date range.
pub async fn economic_calendar(from: &str, to: &str) -> Result<Vec<EconomicCalendarEntryDTO>> {
    let client = build_client()?;
    client
        .get("/stable/economic-calendar", &[("from", from), ("to", to)])
        .await
}

/// Fetch a market-wide calendar and map it to provider-neutral entries.
pub async fn fetch_market_calendar_response(
    kind: crate::models::calendar::market::CalendarKind,
    from: &str,
    to: &str,
) -> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
    use crate::models::calendar::market::{CalendarDetail, CalendarKind, MarketCalendarEntry};

    let entry = |symbol: Option<String>, date: Option<String>, detail: CalendarDetail| {
        MarketCalendarEntry {
            symbol,
            date,
            detail,
        }
    };

    Ok(match kind {
        // FMP serves neither; dispatch falls through to a provider that does
        // (Polygon for holidays, Alpha Vantage for live status).
        CalendarKind::MarketHoliday | CalendarKind::MarketStatus => {
            let operation = kind.operation();
            return Err(crate::error::FinanceError::NotSupported {
                provider: crate::Provider::Fmp,
                operation,
                candidates: operation.capability().candidate_providers(),
            });
        }
        CalendarKind::Earnings => earnings_calendar(from, to)
            .await?
            .into_iter()
            .map(|e| {
                entry(
                    e.symbol,
                    e.date,
                    // FMP's stable earnings calendar dropped the fiscal period
                    // end and the bmo/amc announcement time.
                    CalendarDetail::Earnings {
                        eps: e.eps,
                        eps_estimated: e.eps_estimated,
                        revenue: e.revenue,
                        revenue_estimated: e.revenue_estimated,
                        fiscal_date_ending: None,
                        time: None,
                    },
                )
            })
            .collect(),
        CalendarKind::Ipo => ipo_calendar(from, to)
            .await?
            .into_iter()
            .map(|e| {
                entry(
                    e.symbol,
                    e.date,
                    CalendarDetail::Ipo {
                        company: e.company,
                        exchange: e.exchange,
                        actions: e.actions,
                        shares: e.shares,
                        price_range: e.price_range,
                        market_cap: e.market_cap,
                    },
                )
            })
            .collect(),
        CalendarKind::Dividend => dividend_calendar(from, to)
            .await?
            .into_iter()
            .map(|e| {
                entry(
                    e.symbol,
                    e.date,
                    CalendarDetail::Dividend {
                        dividend: e.dividend,
                        adj_dividend: e.adj_dividend,
                        record_date: e.record_date,
                        payment_date: e.payment_date,
                        declaration_date: e.declaration_date,
                    },
                )
            })
            .collect(),
        CalendarKind::Split => stock_split_calendar(from, to)
            .await?
            .into_iter()
            .map(|e| {
                entry(
                    e.symbol,
                    e.date,
                    CalendarDetail::Split {
                        numerator: e.numerator,
                        denominator: e.denominator,
                    },
                )
            })
            .collect(),
        // Economic releases are market-wide, so they carry no symbol.
        CalendarKind::Economic => economic_calendar(from, to)
            .await?
            .into_iter()
            .map(|e| {
                entry(
                    None,
                    e.date,
                    CalendarDetail::Economic {
                        event: e.event,
                        country: e.country,
                        actual: e.actual,
                        previous: e.previous,
                        estimate: e.estimate,
                        change: e.change,
                        change_percentage: e.change_percentage,
                        impact: e.impact,
                    },
                )
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_earnings_calendar_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/earnings-calendar")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("from".into(), "2024-01-01".into()),
                mockito::Matcher::UrlEncoded("to".into(), "2024-01-31".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"[{
                    "symbol": "MSFT",
                    "date": "2024-01-25",
                    "epsActual": 2.93,
                    "epsEstimated": 2.78,
                    "revenueActual": 62020000000.0,
                    "revenueEstimated": 61100000000.0,
                    "lastUpdated": "2024-01-26"
                }]"#,
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EarningsCalendarEntryDTO> = client
            .get(
                "/stable/earnings-calendar",
                &[("from", "2024-01-01"), ("to", "2024-01-31")],
            )
            .await
            .unwrap();

        let row = &resp[0];
        assert_eq!(row.symbol.as_deref(), Some("MSFT"));
        assert_eq!(row.date.as_deref(), Some("2024-01-25"));
        assert_eq!(row.eps, Some(2.93));
        assert_eq!(row.eps_estimated, Some(2.78));
        assert_eq!(row.revenue, Some(62_020_000_000.0));
        assert_eq!(row.revenue_estimated, Some(61_100_000_000.0));
        assert_eq!(row.last_updated.as_deref(), Some("2024-01-26"));
    }

    #[tokio::test]
    async fn test_economic_calendar_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/economic-calendar")
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EconomicCalendarEntryDTO> = client
            .get(
                "/stable/economic-calendar",
                &[("from", "2024-01-01"), ("to", "2024-01-31")],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].event.as_deref(), Some("CPI"));
    }
}
