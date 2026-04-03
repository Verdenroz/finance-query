//! Insider trading, congressional trading, CIK mapping, and fail-to-deliver endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Insider trading transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InsiderTrade {
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
    /// SEC form type.
    #[serde(rename = "typeOfOwner")]
    pub type_of_owner: Option<String>,
    /// Link to SEC filing.
    pub link: Option<String>,
}

/// CIK mapping entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CikMapping {
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

/// Fail-to-deliver record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FailToDeliver {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Date (YYYY-MM-DD).
    pub date: Option<String>,
    /// Quantity of fails.
    pub quantity: Option<f64>,
    /// Price.
    pub price: Option<f64>,
    /// Security name.
    pub name: Option<String>,
    /// Description.
    pub description: Option<String>,
}

/// Congressional/senate trading record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CongressionalTrade {
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
pub async fn insider_trading(symbol: &str, limit: u32) -> Result<Vec<InsiderTrade>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/api/v4/insider-trading",
            &[("symbol", symbol), ("limit", &limit_str)],
        )
        .await
}

/// Fetch the insider trading RSS feed.
pub async fn insider_trading_rss(limit: u32) -> Result<Vec<InsiderTrade>> {
    let client = build_client()?;
    let limit_str = limit.to_string();
    client
        .get(
            "/api/v4/insider-trading-rss-feed",
            &[("limit", &limit_str)],
        )
        .await
}

/// Search CIK mappings by name.
pub async fn cik_mapper(name: &str) -> Result<Vec<CikMapping>> {
    let client = build_client()?;
    client
        .get("/api/v4/mapper-cik-name", &[("name", name)])
        .await
}

/// Fetch CIK mapping by company name/identifier.
pub async fn cik_mapper_by_company(name: &str) -> Result<Vec<CikMapping>> {
    let client = build_client()?;
    let path = format!("/api/v4/mapper-cik-company/{name}");
    client.get(&path, &[]).await
}

/// Fetch fail-to-deliver data for a symbol.
pub async fn fail_to_deliver(symbol: &str) -> Result<Vec<FailToDeliver>> {
    let client = build_client()?;
    client
        .get("/api/v4/fail_to_deliver", &[("symbol", symbol)])
        .await
}

/// Fetch congressional (senate) trading data for a symbol.
pub async fn congressional_trading(symbol: &str) -> Result<Vec<CongressionalTrade>> {
    let client = build_client()?;
    client
        .get("/api/v4/senate-trading", &[("symbol", symbol)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insider_trading_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v4/insider-trading")
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
                        "typeOfOwner": "officer"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<InsiderTrade> = client
            .get(
                "/api/v4/insider-trading",
                &[("symbol", "AAPL"), ("limit", "10")],
            )
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].reporting_name.as_deref(), Some("Cook Timothy D"));
        assert!((resp[0].price.unwrap() - 185.50).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_congressional_trading_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v4/senate-trading")
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

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<CongressionalTrade> = client
            .get("/api/v4/senate-trading", &[("symbol", "AAPL")])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].last_name.as_deref(), Some("Doe"));
    }
}
