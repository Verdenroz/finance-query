//! FMP company information endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

// ============================================================================
// Response types
// ============================================================================

/// Company profile from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyProfile {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Current price.
    pub price: Option<f64>,
    /// Beta.
    pub beta: Option<f64>,
    /// Volume average.
    #[serde(rename = "volAvg")]
    pub vol_avg: Option<f64>,
    /// Market capitalization.
    #[serde(rename = "mktCap")]
    pub mkt_cap: Option<f64>,
    /// Last dividend.
    #[serde(rename = "lastDiv")]
    pub last_div: Option<f64>,
    /// 52-week range.
    pub range: Option<String>,
    /// Price changes.
    pub changes: Option<f64>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Currency.
    pub currency: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// ISIN.
    pub isin: Option<String>,
    /// CUSIP.
    pub cusip: Option<String>,
    /// Exchange name.
    pub exchange: Option<String>,
    /// Exchange short name.
    #[serde(rename = "exchangeShortName")]
    pub exchange_short_name: Option<String>,
    /// Industry.
    pub industry: Option<String>,
    /// Website.
    pub website: Option<String>,
    /// Company description.
    pub description: Option<String>,
    /// CEO.
    pub ceo: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Country.
    pub country: Option<String>,
    /// Full-time employees.
    #[serde(rename = "fullTimeEmployees")]
    pub full_time_employees: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Address.
    pub address: Option<String>,
    /// City.
    pub city: Option<String>,
    /// State.
    pub state: Option<String>,
    /// ZIP code.
    pub zip: Option<String>,
    /// DCF difference.
    #[serde(rename = "dcfDiff")]
    pub dcf_diff: Option<f64>,
    /// DCF value.
    pub dcf: Option<f64>,
    /// Image/logo URL.
    pub image: Option<String>,
    /// IPO date.
    #[serde(rename = "ipoDate")]
    pub ipo_date: Option<String>,
    /// Default image flag.
    #[serde(rename = "defaultImage")]
    pub default_image: Option<bool>,
    /// Is ETF.
    #[serde(rename = "isEtf")]
    pub is_etf: Option<bool>,
    /// Is actively trading.
    #[serde(rename = "isActivelyTrading")]
    pub is_actively_trading: Option<bool>,
    /// Is ADR.
    #[serde(rename = "isAdr")]
    pub is_adr: Option<bool>,
    /// Is fund.
    #[serde(rename = "isFund")]
    pub is_fund: Option<bool>,
}

/// Key executive from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyExecutive {
    /// Executive title.
    pub title: Option<String>,
    /// Executive name.
    pub name: Option<String>,
    /// Pay.
    pub pay: Option<f64>,
    /// Currency of pay.
    #[serde(rename = "currencyPay")]
    pub currency_pay: Option<String>,
    /// Gender.
    pub gender: Option<String>,
    /// Year born.
    #[serde(rename = "yearBorn")]
    pub year_born: Option<i32>,
    /// Title since.
    #[serde(rename = "titleSince")]
    pub title_since: Option<String>,
}

/// Market capitalization from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketCap {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date.
    pub date: Option<String>,
    /// Market capitalization.
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
}

/// Company outlook from FMP (v4 endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompanyOutlook {
    /// Profile section.
    pub profile: Option<CompanyProfile>,
    /// Metrics section.
    pub metrics: Option<serde_json::Value>,
    /// Ratios section.
    pub ratios: Option<Vec<serde_json::Value>>,
    /// Insider trading section.
    #[serde(rename = "insideTrades")]
    pub inside_trades: Option<Vec<serde_json::Value>>,
    /// Key executives.
    #[serde(rename = "keyExecutives")]
    pub key_executives: Option<Vec<KeyExecutive>>,
    /// Stock news.
    #[serde(rename = "stockNews")]
    pub stock_news: Option<Vec<serde_json::Value>>,
    /// Rating section.
    pub rating: Option<Vec<serde_json::Value>>,
}

/// Stock peer from FMP (v4 endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPeers {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// List of peer symbols.
    #[serde(rename = "peersList")]
    pub peers_list: Option<Vec<String>>,
}

/// Delisted company from FMP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DelistedCompany {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    /// Exchange.
    pub exchange: Option<String>,
    /// IPO date.
    #[serde(rename = "ipoDate")]
    pub ipo_date: Option<String>,
    /// Delisted date.
    #[serde(rename = "delistedDate")]
    pub delisted_date: Option<String>,
}

// ============================================================================
// Query functions
// ============================================================================

/// Fetch company profile for a symbol.
pub async fn company_profile(symbol: &str) -> Result<Vec<CompanyProfile>> {
    let client = super::build_client()?;
    client
        .get(&format!("/api/v3/profile/{symbol}"), &[])
        .await
}

/// Fetch key executives for a symbol.
pub async fn key_executives(symbol: &str) -> Result<Vec<KeyExecutive>> {
    let client = super::build_client()?;
    client
        .get(&format!("/api/v3/key-executives/{symbol}"), &[])
        .await
}

/// Fetch market capitalization for a symbol.
pub async fn market_cap(symbol: &str) -> Result<Vec<MarketCap>> {
    let client = super::build_client()?;
    client
        .get(&format!("/api/v3/market-capitalization/{symbol}"), &[])
        .await
}

/// Fetch historical market capitalization for a symbol.
pub async fn historical_market_cap(symbol: &str, limit: Option<u32>) -> Result<Vec<MarketCap>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(100).to_string();
    client
        .get(
            &format!("/api/v3/historical-market-capitalization/{symbol}"),
            &[("limit", &limit_str)],
        )
        .await
}

/// Fetch company outlook for a symbol (v4 endpoint).
pub async fn company_outlook(symbol: &str) -> Result<CompanyOutlook> {
    let client = super::build_client()?;
    client
        .get("/api/v4/company-outlook", &[("symbol", symbol)])
        .await
}

/// Fetch stock peers for a symbol (v4 endpoint).
pub async fn stock_peers(symbol: &str) -> Result<Vec<StockPeers>> {
    let client = super::build_client()?;
    client
        .get("/api/v4/stock_peers", &[("symbol", symbol)])
        .await
}

/// Fetch delisted companies.
pub async fn delisted_companies(limit: Option<u32>) -> Result<Vec<DelistedCompany>> {
    let client = super::build_client()?;
    let limit_str = limit.unwrap_or(100).to_string();
    client
        .get("/api/v3/delisted-companies", &[("limit", &limit_str)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_company_profile_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/profile/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([{
                    "symbol": "AAPL",
                    "price": 178.72,
                    "beta": 1.286,
                    "volAvg": 58405568,
                    "mktCap": 2794000000000_f64,
                    "companyName": "Apple Inc.",
                    "currency": "USD",
                    "exchange": "NASDAQ Global Select",
                    "exchangeShortName": "NASDAQ",
                    "industry": "Consumer Electronics",
                    "sector": "Technology",
                    "country": "US",
                    "ceo": "Mr. Timothy D. Cook",
                    "isEtf": false,
                    "isActivelyTrading": true
                }])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<CompanyProfile> = client
            .get("/api/v3/profile/AAPL", &[])
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(result[0].company_name.as_deref(), Some("Apple Inc."));
        assert_eq!(result[0].sector.as_deref(), Some("Technology"));
        assert_eq!(result[0].is_etf, Some(false));
    }

    #[tokio::test]
    async fn test_key_executives_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/key-executives/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "title": "Chief Executive Officer",
                        "name": "Mr. Timothy D. Cook",
                        "pay": 16425933,
                        "currencyPay": "USD",
                        "gender": "male",
                        "yearBorn": 1960
                    },
                    {
                        "title": "Chief Financial Officer",
                        "name": "Mr. Luca Maestri",
                        "pay": 5019783,
                        "currencyPay": "USD",
                        "gender": "male",
                        "yearBorn": 1963
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<KeyExecutive> = client
            .get("/api/v3/key-executives/AAPL", &[])
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name.as_deref(), Some("Mr. Timothy D. Cook"));
        assert_eq!(result[0].pay, Some(16425933.0));
    }
}
