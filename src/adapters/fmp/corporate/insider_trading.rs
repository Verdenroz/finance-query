//! Insider trading, congressional trading, and fail-to-deliver endpoints.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::filings::{CongressionalTrade, FailToDeliver, InsiderTrade};

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
    /// Reporting owner's relationship to the issuer (e.g. "officer", "director").
    #[serde(rename = "typeOfOwner")]
    pub type_of_owner: Option<String>,
    /// Link to SEC filing.
    pub link: Option<String>,
}

/// Fail-to-deliver record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FailToDeliverDTO {
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
            &[("symbol", symbol), ("limit", &limit_str)],
        )
        .await
}

/// Fetch fail-to-deliver data for a symbol. FMP has not published a
/// `/stable` replacement for this endpoint.
pub async fn fail_to_deliver(symbol: &str) -> Result<Vec<FailToDeliverDTO>> {
    let client = build_client()?;
    client
        .get("/api/v4/fail_to_deliver", &[("symbol", symbol)])
        .await
}

/// Fetch congressional (senate) trading data for a symbol.
pub async fn congressional_trading(symbol: &str) -> Result<Vec<CongressionalTradeDTO>> {
    let client = build_client()?;
    client
        .get("/stable/senate-trades", &[("symbol", symbol)])
        .await
}

/// FMP has no form type, accession number, or ownership flags, so those
/// default; `transaction_code` is `transaction_type`'s letter prefix.
fn to_insider_trade(dto: InsiderTradeDTO) -> InsiderTrade {
    InsiderTrade {
        symbol: dto.symbol,
        insider_name: dto.reporting_name,
        insider_cik: dto.reporting_cik,
        officer_title: dto.type_of_owner,
        transaction_code: dto
            .transaction_type
            .as_deref()
            .and_then(|t| t.split('-').next())
            .map(str::to_string),
        url: dto.link,
        transaction_date: dto.transaction_date,
        shares: dto.securities_transacted,
        price_per_share: dto.price,
        shares_owned_after: dto.securities_owned,
        ..Default::default()
    }
}

/// Fetch canonical insider transactions for a symbol.
pub async fn fetch_insider_trades_response(symbol: &str, limit: u32) -> Result<Vec<InsiderTrade>> {
    Ok(insider_trading(symbol, limit)
        .await?
        .into_iter()
        .map(to_insider_trade)
        .collect())
}

fn to_congressional_trade(dto: CongressionalTradeDTO) -> CongressionalTrade {
    CongressionalTrade {
        symbol: dto.symbol,
        first_name: dto.first_name,
        last_name: dto.last_name,
        office: dto.office,
        district: dto.district,
        trade_type: dto.trade_type,
        amount: dto.amount,
        asset_description: dto.asset_description,
        transaction_date: dto.transaction_date,
        disclosure_date: dto.disclosure_date,
        link: dto.link,
    }
}

/// Fetch canonical congressional trades for a symbol.
pub async fn fetch_congressional_trades_response(symbol: &str) -> Result<Vec<CongressionalTrade>> {
    Ok(congressional_trading(symbol)
        .await?
        .into_iter()
        .map(to_congressional_trade)
        .collect())
}

fn to_fail_to_deliver(dto: FailToDeliverDTO) -> FailToDeliver {
    FailToDeliver {
        symbol: dto.symbol,
        date: dto.date,
        quantity: dto.quantity,
        price: dto.price,
        name: dto.name,
        description: dto.description,
    }
}

/// Fetch canonical fails-to-deliver data for a symbol.
pub async fn fetch_fails_to_deliver_response(symbol: &str) -> Result<Vec<FailToDeliver>> {
    Ok(fail_to_deliver(symbol)
        .await?
        .into_iter()
        .map(to_fail_to_deliver)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_insider_trade_fields() {
        let dto: InsiderTradeDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "transactionDate": "2024-01-12",
            "reportingCik": "0001234567",
            "reportingName": "Cook Timothy D",
            "transactionType": "S-Sale",
            "securitiesTransacted": 50000.0,
            "price": 185.50,
            "securitiesOwned": 3200000.0,
            "link": "https://sec.gov/example",
            "typeOfOwner": "officer"
        }))
        .unwrap();

        let out = to_insider_trade(dto);
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.insider_name.as_deref(), Some("Cook Timothy D"));
        assert_eq!(out.transaction_code.as_deref(), Some("S"));
        assert_eq!(out.shares, Some(50000.0));
        assert_eq!(out.shares_owned_after, Some(3200000.0));
        assert_eq!(out.officer_title.as_deref(), Some("officer"));
        assert_eq!(out.form_type, None);
        assert!(!out.is_director);
    }

    #[test]
    fn maps_congressional_trade_fields() {
        let dto: CongressionalTradeDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "transactionDate": "2024-01-10",
            "disclosureDate": "2024-01-20",
            "firstName": "John",
            "lastName": "Doe",
            "office": "Senate",
            "type": "Purchase",
            "amount": "$1,001 - $15,000"
        }))
        .unwrap();

        let out = to_congressional_trade(dto);
        assert_eq!(out.last_name.as_deref(), Some("Doe"));
        assert_eq!(out.trade_type.as_deref(), Some("Purchase"));
        assert_eq!(out.amount.as_deref(), Some("$1,001 - $15,000"));
    }

    #[test]
    fn maps_fail_to_deliver_fields() {
        let dto: FailToDeliverDTO = serde_json::from_value(serde_json::json!({
            "symbol": "AAPL",
            "date": "2024-01-15",
            "quantity": 1200.0,
            "price": 185.50
        }))
        .unwrap();

        let out = to_fail_to_deliver(dto);
        assert_eq!(out.symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.quantity, Some(1200.0));
    }

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
                        "typeOfOwner": "officer"
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
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].reporting_name.as_deref(), Some("Cook Timothy D"));
        assert!((resp[0].price.unwrap() - 185.50).abs() < 0.01);
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
