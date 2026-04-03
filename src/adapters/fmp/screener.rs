//! Stock screener, symbol search, and CIK lookup endpoints for Financial Modeling Prep.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

/// A result from the stock screener endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ScreenerResult {
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
pub struct SearchResult {
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

/// A result from the CIK search endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CikResult {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company CIK number.
    #[serde(rename = "companyCik")]
    pub company_cik: Option<String>,
}

/// Screen stocks by various financial criteria.
///
/// * `params` - Query params such as `marketCapMoreThan`, `sector`, `industry`, `country`, `exchange`, `limit`, etc.
pub async fn stock_screener(params: &[(&str, &str)]) -> Result<Vec<ScreenerResult>> {
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
) -> Result<Vec<SearchResult>> {
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

/// Look up a company by CIK number.
///
/// * `cik` - CIK number (e.g., `"0000320193"`)
pub async fn cik_search(cik: &str) -> Result<Vec<CikResult>> {
    let client = build_client()?;
    let path = format!("/api/v3/cik/{cik}");
    client.get(&path, &[]).await
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<SearchResult> = client
            .get("/api/v3/search", &[("query", "apple"), ("limit", "5")])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].name.as_deref(), Some("Apple Inc."));
    }
}
