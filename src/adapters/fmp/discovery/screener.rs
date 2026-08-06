//! Stock screener, symbol search, and CIK lookup endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

/// A result from the stock screener endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScreenerResultDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Market capitalization.
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    /// Sector.
    pub sector: Option<String>,
    /// Industry.
    pub industry: Option<String>,
    /// Beta.
    pub beta: Option<f64>,
    /// Current price.
    pub price: Option<f64>,
    /// Last annual dividend.
    #[serde(rename = "lastAnnualDividend")]
    pub last_annual_dividend: Option<f64>,
    /// Trading volume.
    pub volume: Option<f64>,
    /// Exchange.
    pub exchange: Option<String>,
    /// Short exchange name.
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
    /// Country.
    pub country: Option<String>,
    /// Whether the symbol is an ETF.
    #[serde(rename = "isEtf")]
    pub is_etf: Option<bool>,
    /// Whether the symbol is a fund.
    #[serde(rename = "isFund")]
    pub is_fund: Option<bool>,
    /// Whether the symbol is actively trading.
    #[serde(rename = "isActivelyTrading")]
    pub is_actively_trading: Option<bool>,
}

/// A result from the symbol search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResultDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Security name.
    pub name: Option<String>,
    /// Currency.
    pub currency: Option<String>,
    /// Exchange name.
    #[serde(rename = "stockExchange")]
    pub stock_exchange: Option<String>,
    /// Short exchange name.
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
}

/// Screen stocks by various financial criteria.
///
/// * `params` - Query params such as `marketCapMoreThan`, `sector`, `industry`, `country`, `exchange`, `limit`, etc.
pub async fn stock_screener(params: &[(&str, &str)]) -> Result<Vec<ScreenerResultDTO>> {
    let client = build_client()?;
    client.get("/api/v3/stock-screener", params).await
}

/// Search for symbols matching a query string.
///
/// * `query` - Search query (e.g., `"apple"`)
/// * `limit` - Maximum number of results (optional)
/// * `exchange` - Filter by exchange (optional)
pub async fn symbol_search(
    query: &str,
    limit: Option<u32>,
    exchange: Option<&str>,
) -> Result<Vec<SearchResultDTO>> {
    let client = build_client()?;
    let limit_str = limit.map(|l| l.to_string());
    let mut params: Vec<(&str, &str)> = vec![("query", query)];
    if let Some(ref l) = limit_str {
        params.push(("limit", l));
    }
    if let Some(e) = exchange {
        params.push(("exchange", e));
    }
    client.get("/api/v3/search", &params).await
}

/// Search symbols and return provider-neutral matches.
pub async fn fetch_symbol_search_response(
    query: &str,
    limit: u32,
) -> Result<Vec<crate::models::discovery::reference::SymbolMatch>> {
    use crate::models::discovery::reference::SymbolMatch;
    let results = symbol_search(query, Some(limit), None).await?;
    Ok(results
        .into_iter()
        .filter_map(|r| {
            Some(SymbolMatch {
                symbol: r.symbol?,
                id: None,
                name: r.name,
                // Prefer the short code (e.g. "NASDAQ") over the long venue name.
                exchange: r.exchange_short_name.or(r.stock_exchange),
                asset_type: None,
                currency: r.currency,
                active: None,
                market_cap_rank: None,
                thumbnail: None,
                image: None,
            })
        })
        .collect())
}

/// Run a screener query and return provider-neutral matches.
pub async fn fetch_screener_response(
    filters: &crate::models::discovery::reference::ScreenerFilters,
) -> Result<Vec<crate::models::discovery::reference::ScreenerMatch>> {
    use crate::models::discovery::reference::ScreenerMatch;
    let owned = filters.to_query();
    let params: Vec<(&str, &str)> = owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let results = stock_screener(&params).await?;
    Ok(results
        .into_iter()
        .filter_map(|r| {
            Some(ScreenerMatch {
                symbol: r.symbol?,
                name: r.company_name,
                price: r.price,
                market_cap: r.market_cap,
                sector: r.sector,
                industry: r.industry,
                beta: r.beta,
                volume: r.volume,
                exchange: r.exchange_short_name.or(r.exchange),
                country: r.country,
                is_etf: r.is_etf,
                is_actively_trading: r.is_actively_trading,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_symbol_search_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("query".into(), "apple".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "5".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "name": "Apple Inc.",
                        "currency": "USD",
                        "stockExchange": "NASDAQ",
                        "exchangeShortName": "NASDAQ"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let result: Vec<SearchResultDTO> = client
            .get("/api/v3/search", &[("query", "apple"), ("limit", "5")])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].name.as_deref(), Some("Apple Inc."));
    }
}
