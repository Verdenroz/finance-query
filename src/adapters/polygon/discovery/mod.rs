//! Reference data endpoints: tickers, exchanges, conditions, market holidays, market status.

use serde::{Deserialize, Serialize};

use crate::adapters::common::encode_path_segment;
use crate::error::Result;
use crate::models::corporate::recommendation::SimilarSymbol;

use super::build_client;
use super::models::PaginatedResponseDTO;

/// Ticker reference entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerRefDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Security name.
    pub name: Option<String>,
    /// Market (e.g., `"stocks"`, `"crypto"`, `"fx"`).
    pub market: Option<String>,
    /// Locale (e.g., `"us"`).
    pub locale: Option<String>,
    /// Primary exchange.
    pub primary_exchange: Option<String>,
    /// Asset type.
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    /// Whether the ticker is active.
    pub active: Option<bool>,
    /// Currency name.
    pub currency_name: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// Composite FIGI.
    pub composite_figi: Option<String>,
    /// Share class FIGI.
    pub share_class_figi: Option<String>,
    /// Last updated date.
    pub last_updated_utc: Option<String>,
}

/// Detailed ticker overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerDetailsDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// Market.
    pub market: Option<String>,
    /// Locale.
    pub locale: Option<String>,
    /// Primary exchange.
    pub primary_exchange: Option<String>,
    /// Asset type.
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    /// Active.
    pub active: Option<bool>,
    /// Currency.
    pub currency_name: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// SIC code.
    pub sic_code: Option<String>,
    /// SIC description.
    pub sic_description: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Homepage URL.
    pub homepage_url: Option<String>,
    /// Total employees.
    pub total_employees: Option<u64>,
    /// Market cap.
    pub market_cap: Option<f64>,
    /// Phone number.
    pub phone_number: Option<String>,
    /// Address.
    pub address: Option<serde_json::Value>,
    /// Branding (logo, icon).
    pub branding: Option<serde_json::Value>,
    /// List date.
    pub list_date: Option<String>,
    /// Share class shares outstanding.
    pub share_class_shares_outstanding: Option<f64>,
    /// Weighted shares outstanding.
    pub weighted_shares_outstanding: Option<f64>,
}

/// Ticker details response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TickerDetailsResponseDTO {
    /// Request ID.
    pub request_id: Option<String>,
    /// Status.
    pub status: Option<String>,
    /// Ticker details.
    pub results: Option<TickerDetailsDTO>,
}

/// Ticker type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)] // unrouted: no capability route or consumer yet
pub struct TickerTypeDTO {
    /// Type code.
    pub code: Option<String>,
    /// Description.
    pub description: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Locale.
    pub locale: Option<String>,
}

/// Related ticker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RelatedTickerDTO {
    /// Ticker symbol.
    pub ticker: Option<String>,
}

/// Exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Exchange {
    /// Exchange ID.
    pub id: Option<i64>,
    /// Exchange type.
    #[serde(rename = "type")]
    pub exchange_type: Option<String>,
    /// Asset class.
    pub asset_class: Option<String>,
    /// Locale.
    pub locale: Option<String>,
    /// Exchange name.
    pub name: Option<String>,
    /// MIC code.
    pub mic: Option<String>,
    /// Operating MIC.
    pub operating_mic: Option<String>,
    /// Participant ID.
    pub participant_id: Option<String>,
    /// URL.
    pub url: Option<String>,
}

/// Market holiday.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketHolidayDTO {
    /// Holiday name.
    pub name: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Exchange (e.g., `"NYSE"`, `"NASDAQ"`).
    pub exchange: Option<String>,
    /// Status (e.g., `"closed"`, `"early-close"`).
    pub status: Option<String>,
    /// Open time (if early close).
    pub open: Option<String>,
    /// Close time (if early close).
    pub close: Option<String>,
}

/// Fetch all tickers.
///
/// * `params` - Query params: `type`, `market`, `exchange`, `cusip`, `cik`, `active`, `sort`, `order`, `limit`, `search`
pub async fn all_tickers(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<TickerRefDTO>> {
    let client = build_client()?;
    client.get("/v3/reference/tickers", params).await
}

/// Fetch detailed ticker information.
pub async fn ticker_details(ticker: &str) -> Result<TickerDetailsResponseDTO> {
    let client = build_client()?;
    let path = format!("/v3/reference/tickers/{}", encode_path_segment(ticker));
    client
        .get_as(&path, &[], "ticker_details", "ticker details")
        .await
}

/// Fetch ticker types.
#[allow(dead_code)] // unrouted: no capability route or consumer yet
pub async fn ticker_types(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<TickerTypeDTO>> {
    let client = build_client()?;
    client.get("/v3/reference/tickers/types", params).await
}

/// Fetch related tickers.
pub async fn related_tickers(ticker: &str) -> Result<PaginatedResponseDTO<RelatedTickerDTO>> {
    let client = build_client()?;
    let path = format!("/v1/related-companies/{}", encode_path_segment(ticker));
    client.get(&path, &[]).await
}

/// Fetch similar/recommended symbols (canonical) for a stock ticker.
pub async fn fetch_similar_symbols_response(
    symbol: &str,
    _limit: u32,
) -> Result<Vec<SimilarSymbol>> {
    let paginated = related_tickers(symbol).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|r| SimilarSymbol {
            symbol: r.ticker.unwrap_or_default(),
            score: 0.0,
        })
        .collect())
}

/// Search tickers and return provider-neutral matches.
pub async fn fetch_symbol_search_response(
    query: &str,
    limit: u32,
) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
    use crate::models::discovery::reference::SymbolMatch;
    let limit_str = limit.to_string();
    let paginated =
        all_tickers(&[("search", query), ("active", "true"), ("limit", &limit_str)]).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| {
            Some(SymbolMatch {
                symbol: t.ticker?,
                id: None,
                name: t.name,
                exchange: t.primary_exchange,
                asset_type: t.asset_type,
                currency: t.currency_name,
                active: t.active,
                market_cap_rank: None,
                thumbnail: None,
                image: None,
            })
        })
        .collect())
}

/// Fetch detailed reference data for a ticker (canonical).
pub async fn fetch_symbol_details_response(
    symbol: &str,
) -> Result<crate::models::discovery::reference::SymbolDetails> {
    use crate::models::discovery::reference::SymbolDetails;
    let resp = ticker_details(symbol).await?;
    let d = resp
        .results
        .ok_or_else(|| crate::error::FinanceError::ResponseStructureError {
            field: "results".to_string(),
            context: format!("No ticker details returned for {symbol}"),
        })?;
    Ok(SymbolDetails {
        symbol: d.ticker.unwrap_or_else(|| symbol.to_string()),
        name: d.name,
        description: d.description,
        exchange: d.primary_exchange,
        asset_type: d.asset_type,
        cik: d.cik,
        sic_code: d.sic_code,
        sic_description: d.sic_description,
        homepage_url: d.homepage_url,
        employees: d.total_employees,
        market_cap: d.market_cap,
        list_date: d.list_date,
        shares_outstanding: d
            .weighted_shares_outstanding
            .or(d.share_class_shares_outstanding),
    })
}

/// Fetch the tradable exchange listing (canonical).
pub async fn fetch_exchanges_response()
-> Result<Vec<crate::models::discovery::reference::ExchangeInfo>> {
    use crate::models::discovery::reference::ExchangeInfo;
    let paginated = exchanges(&[]).await?;
    Ok(paginated
        .results
        .unwrap_or_default()
        .into_iter()
        .map(|e| ExchangeInfo {
            id: e.id,
            name: e.name,
            mic: e.mic,
            operating_mic: e.operating_mic,
            asset_class: e.asset_class,
            locale: e.locale,
            exchange_type: e.exchange_type,
            url: e.url,
        })
        .collect())
}

/// Fetch exchanges list.
pub async fn exchanges(params: &[(&str, &str)]) -> Result<PaginatedResponseDTO<Exchange>> {
    let client = build_client()?;
    client.get("/v3/reference/exchanges", params).await
}

/// Fetch upcoming market holidays.
pub async fn market_holidays() -> Result<Vec<MarketHolidayDTO>> {
    let client = build_client()?;
    client
        .get_as(
            "/v1/marketstatus/upcoming",
            &[],
            "market_holidays",
            "market holidays",
        )
        .await
}

/// Fetch upcoming market holidays as canonical calendar entries.
pub async fn fetch_market_holidays_response()
-> Result<Vec<crate::models::calendar::market::MarketCalendarEntry>> {
    Ok(market_holidays()
        .await?
        .into_iter()
        .map(holiday_to_canonical)
        .collect())
}

/// Convert one holiday DTO into a canonical calendar entry.
fn holiday_to_canonical(
    h: MarketHolidayDTO,
) -> crate::models::calendar::market::MarketCalendarEntry {
    use crate::models::calendar::market::{CalendarDetail, MarketCalendarEntry};
    MarketCalendarEntry {
        symbol: None,
        date: h.date,
        detail: CalendarDetail::MarketHoliday {
            name: h.name,
            exchange: h.exchange,
            status: h.status,
            open: h.open,
            close: h.close,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holiday_maps_to_calendar_entry() {
        use crate::models::calendar::market::CalendarDetail;
        let entry = holiday_to_canonical(
            serde_json::from_value(serde_json::json!({
                "name": "Thanksgiving",
                "date": "2026-11-26",
                "exchange": "NYSE",
                "status": "closed"
            }))
            .unwrap(),
        );
        assert_eq!(entry.date.as_deref(), Some("2026-11-26"));
        assert!(entry.symbol.is_none());
        match entry.detail {
            CalendarDetail::MarketHoliday {
                name,
                exchange,
                status,
                open,
                close,
            } => {
                assert_eq!(name.as_deref(), Some("Thanksgiving"));
                assert_eq!(exchange.as_deref(), Some("NYSE"));
                assert_eq!(status.as_deref(), Some("closed"));
                assert!(open.is_none() && close.is_none());
            }
            other => panic!("wrong detail: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_ticker_details_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/v3/reference/tickers/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apiKey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(serde_json::json!({
                "request_id": "abc",
                "status": "OK",
                "results": {
                    "ticker": "AAPL",
                    "name": "Apple Inc.",
                    "market": "stocks",
                    "locale": "us",
                    "primary_exchange": "XNAS",
                    "type": "CS",
                    "active": true,
                    "currency_name": "usd",
                    "market_cap": 2850000000000.0,
                    "description": "Apple Inc. designs, manufactures, and markets smartphones..."
                }
            }).to_string())
            .create_async().await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get_raw("/v3/reference/tickers/AAPL", &[])
            .await
            .unwrap();
        let resp: TickerDetailsResponseDTO = serde_json::from_value(json).unwrap();
        let details = resp.results.unwrap();
        assert_eq!(details.name.as_deref(), Some("Apple Inc."));
        assert_eq!(details.ticker.as_deref(), Some("AAPL"));
        assert!((details.market_cap.unwrap() - 2850000000000.0).abs() < 1.0);
    }
}
