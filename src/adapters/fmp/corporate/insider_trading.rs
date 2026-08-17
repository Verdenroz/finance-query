//! Insider trading, congressional trading, and CIK mapping endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Insider trading transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InsiderTradeDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Filing date.
    #[serde(rename = "filingDate")]
    pub filing_date: Option<String>,
    /// Transaction date.
    #[serde(rename = "transactionDate")]
    pub transaction_date: Option<String>,
    /// Reporting CIK.
    #[serde(rename = "reportingCik")]
    pub reporting_cik: Option<String>,
    /// Reporting person name.
    #[serde(rename = "reportingName")]
    pub reporting_name: Option<String>,
    /// Transaction type (e.g., "P-Purchase", "S-Sale").
    #[serde(rename = "transactionType")]
    pub transaction_type: Option<String>,
    /// Number of securities transacted.
    #[serde(rename = "securitiesTransacted")]
    pub securities_transacted: Option<f64>,
    /// Price per share.
    pub price: Option<f64>,
    /// Securities owned after transaction.
    #[serde(rename = "securitiesOwned")]
    pub securities_owned: Option<f64>,
    /// Relationship of the reporting person to the issuer (e.g. `"officer"`).
    #[serde(rename = "typeOfOwner")]
    pub type_of_owner: Option<String>,
    /// Link to SEC filing.
    #[serde(rename = "url")]
    pub link: Option<String>,
}

/// CIK mapping entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)] // unrouted: insider-transaction surface lands with #243/#264
pub struct CikMappingDTO {
    /// Reporting CIK.
    #[serde(rename = "reportingCik")]
    pub reporting_cik: Option<String>,
    /// Reporting name.
    #[serde(rename = "reportingName")]
    pub reporting_name: Option<String>,
    /// Company CIK.
    #[serde(rename = "companyCik")]
    pub company_cik: Option<String>,
    /// Company name.
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
}

/// Congressional/senate trading record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[allow(dead_code)] // unrouted: insider-transaction surface lands with #243/#264
pub struct CongressionalTradeDTO {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Transaction date.
    #[serde(rename = "transactionDate")]
    pub transaction_date: Option<String>,
    /// Disclosure date.
    #[serde(rename = "disclosureDate")]
    pub disclosure_date: Option<String>,
    /// First name.
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    /// Last name.
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    /// Office.
    pub office: Option<String>,
    /// District.
    pub district: Option<String>,
    /// Transaction type.
    #[serde(rename = "type")]
    pub trade_type: Option<String>,
    /// Amount range.
    pub amount: Option<String>,
    /// Asset description.
    #[serde(rename = "assetDescription")]
    pub asset_description: Option<String>,
    /// Link to filing.
    pub link: Option<String>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch insider trading transactions for a symbol.
pub async fn insider_trading(symbol: &str, limit: u32) -> Result<Vec<InsiderTradeDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/insider-trading/search",
            &[("symbol", symbol), ("page", "0"), ("limit", &limit_str)],
        )
        .await
}

/// Fetch the insider trading RSS feed.
#[allow(dead_code)] // unrouted: insider-transaction surface lands with #243/#264
pub async fn insider_trading_rss(limit: u32) -> Result<Vec<InsiderTradeDTO>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/stable/insider-trading/latest",
            &[("page", "0"), ("limit", &limit_str)],
        )
        .await
}

/// Search CIK mappings by name.
#[allow(dead_code)] // unrouted: insider-transaction surface lands with #243/#264
pub async fn cik_mapper(name: &str) -> Result<Vec<CikMappingDTO>> {
    let client = build_client()?;
    client
        .get(
            "/stable/sec-filings-company-search/name",
            &[("company", name)],
        )
        .await
}

/// Fetch congressional (senate) trading data for a symbol.
#[allow(dead_code)] // unrouted: insider-transaction surface lands with #243/#264
pub async fn congressional_trading(symbol: &str) -> Result<Vec<CongressionalTradeDTO>> {
    let client = build_client()?;
    client
        .get("/stable/senate-trades", &[("symbol", symbol)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insider_trading_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/insider-trading/search")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "10".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "filingDate": "2024-01-15",
                        "transactionDate": "2024-01-12",
                        "reportingCik": "0001234567",
                        "reportingName": "Cook Timothy D",
                        "transactionType": "S-Sale",
                        "securitiesTransacted": 50000.0,
                        "price": 185.50,
                        "securitiesOwned": 3200000.0,
                        "typeOfOwner": "officer",
                        "url": "https://www.sec.gov/Archives/edgar/data/320193/x.htm"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<InsiderTradeDTO> = client
            .get(
                "/stable/insider-trading/search",
                &[("symbol", "AAPL"), ("limit", "10")],
            )
            .await
            .unwrap();

        let row = &resp[0];
        assert_eq!(row.symbol.as_deref(), Some("AAPL"));
        assert_eq!(row.filing_date.as_deref(), Some("2024-01-15"));
        assert_eq!(row.transaction_date.as_deref(), Some("2024-01-12"));
        assert_eq!(row.reporting_cik.as_deref(), Some("0001234567"));
        assert_eq!(row.reporting_name.as_deref(), Some("Cook Timothy D"));
        assert_eq!(row.transaction_type.as_deref(), Some("S-Sale"));
        assert_eq!(row.securities_transacted, Some(50_000.0));
        assert_eq!(row.price, Some(185.50));
        assert_eq!(row.securities_owned, Some(3_200_000.0));
        assert_eq!(row.type_of_owner.as_deref(), Some("officer"));
        assert_eq!(
            row.link.as_deref(),
            Some("https://www.sec.gov/Archives/edgar/data/320193/x.htm")
        );
    }

    #[tokio::test]
    async fn test_congressional_trading_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/senate-trades")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "symbol": "AAPL",
                        "transactionDate": "2024-01-10",
                        "disclosureDate": "2024-01-20",
                        "firstName": "John",
                        "lastName": "Doe",
                        "office": "Senate",
                        "type": "Purchase",
                        "amount": "$1,001 - $15,000"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<CongressionalTradeDTO> = client
            .get("/stable/senate-trades", &[("symbol", "AAPL")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].last_name.as_deref(), Some("Doe"));
    }
}
