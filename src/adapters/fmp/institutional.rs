//! Institutional ownership endpoints: institutional holders, ETF holders, mutual fund holders, Form 13F.

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Institutional holder entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstitutionalHolder {
    /// Institution name.
    pub holder: Option<String>,
    /// Number of shares held.
    pub shares: Option<f64>,
    /// Date reported.
    #[serde(rename = "dateReported")]
    pub date_reported: Option<String>,
    /// Change in shares.
    pub change: Option<f64>,
}

/// ETF holder entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EtfHolder {
    /// Asset name / ticker.
    pub asset: Option<String>,
    /// Number of shares held.
    #[serde(rename = "sharesNumber")]
    pub shares_number: Option<f64>,
    /// Weight in ETF as a percentage.
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<f64>,
    /// Market value.
    #[serde(rename = "marketValue")]
    pub market_value: Option<f64>,
    /// Updated date.
    pub updated: Option<String>,
}

/// Mutual fund holder entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MutualFundHolder {
    /// Fund name.
    pub holder: Option<String>,
    /// Number of shares held.
    pub shares: Option<f64>,
    /// Date reported.
    #[serde(rename = "dateReported")]
    pub date_reported: Option<String>,
    /// Change in shares.
    pub change: Option<f64>,
    /// Weight percentage.
    #[serde(rename = "weightPercentage")]
    pub weight_percentage: Option<f64>,
}

/// Form 13F filing entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Form13F {
    /// Date.
    pub date: Option<String>,
    /// Filing date.
    #[serde(rename = "fillingDate")]
    pub filling_date: Option<String>,
    /// Accepted date.
    #[serde(rename = "acceptedDate")]
    pub accepted_date: Option<String>,
    /// CIK.
    pub cik: Option<String>,
    /// CUSIP.
    pub cusip: Option<String>,
    /// Ticker symbol.
    #[serde(rename = "tickercusip")]
    pub ticker_cusip: Option<String>,
    /// Company name.
    #[serde(rename = "nameOfIssuer")]
    pub name_of_issuer: Option<String>,
    /// Number of shares.
    pub shares: Option<f64>,
    /// Value of holding.
    pub value: Option<f64>,
    /// Filing link.
    pub link: Option<String>,
}

// ============================================================================
// Public API
// ============================================================================

/// Fetch institutional holders of a stock.
pub async fn institutional_holders(symbol: &str) -> Result<Vec<InstitutionalHolder>> {
    let client = build_client()?;
    let path = format!("/api/v3/institutional-holder/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch ETF holders of a stock.
pub async fn etf_holders(symbol: &str) -> Result<Vec<EtfHolder>> {
    let client = build_client()?;
    let path = format!("/api/v3/etf-holder/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch mutual fund holders of a stock.
pub async fn mutual_fund_holders(symbol: &str) -> Result<Vec<MutualFundHolder>> {
    let client = build_client()?;
    let path = format!("/api/v3/mutual-fund-holder/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch Form 13F filings for a CIK.
///
/// * `cik` - Central Index Key
/// * `date` - Filing date (YYYY-MM-DD)
pub async fn form_13f(cik: &str, date: &str) -> Result<Vec<Form13F>> {
    let client = build_client()?;
    let path = format!("/api/v3/form-thirteen/{cik}");
    client.get(&path, &[("date", date)]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_institutional_holders_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/institutional-holder/AAPL")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "holder": "Vanguard Group Inc",
                        "shares": 1300000000.0,
                        "dateReported": "2024-01-15",
                        "change": 5000000.0
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<InstitutionalHolder> = client
            .get("/api/v3/institutional-holder/AAPL", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].holder.as_deref(), Some("Vanguard Group Inc"));
    }

    #[tokio::test]
    async fn test_etf_holders_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/api/v3/etf-holder/SPY")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "asset": "AAPL",
                        "sharesNumber": 170000000.0,
                        "weightPercentage": 7.2,
                        "marketValue": 31450000000.0,
                        "updated": "2024-01-15"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfHolder> = client
            .get("/api/v3/etf-holder/SPY", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].asset.as_deref(), Some("AAPL"));
        assert!((resp[0].weight_percentage.unwrap() - 7.2).abs() < 0.01);
    }
}
