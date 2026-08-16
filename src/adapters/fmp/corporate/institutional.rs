//! Institutional ownership endpoints: institutional holders, ETF holders, mutual fund holders, Form 13F.

use serde::{Deserialize, Serialize};

use crate::error::{FinanceError, Result};

use crate::adapters::fmp::build_client;

// ============================================================================
// Response types
// ============================================================================

/// Institutional holder entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InstitutionalHolderDTO {
    /// Institution name.
    #[serde(alias = "investorName", alias = "name")]
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
pub struct EtfHolderDTO {
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
pub struct MutualFundHolderDTO {
    /// Fund name.
    #[serde(alias = "investorName", alias = "name")]
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
pub struct Form13FDTO {
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
pub async fn institutional_holders(symbol: &str) -> Result<Vec<InstitutionalHolderDTO>> {
    let client = build_client()?;
    let (year, quarter) = latest_completed_quarter();
    client
        .get(
            "/stable/institutional-ownership/extract-analytics/holder",
            &[
                ("symbol", symbol),
                ("year", &year),
                ("quarter", &quarter),
                ("page", "0"),
                ("limit", "100"),
            ],
        )
        .await
}

/// Fetch ETF holders of a stock.
pub async fn etf_holders(symbol: &str) -> Result<Vec<EtfHolderDTO>> {
    let client = build_client()?;
    client
        .get("/stable/etf/holdings", &[("symbol", symbol)])
        .await
}

/// Fetch mutual fund holders of a stock.
pub async fn mutual_fund_holders(symbol: &str) -> Result<Vec<MutualFundHolderDTO>> {
    let client = build_client()?;
    client
        .get(
            "/stable/funds/disclosure-holders-latest",
            &[("symbol", symbol)],
        )
        .await
}

/// Fetch Form 13F filings for a CIK.
///
/// * `cik` - Central Index Key
/// * `date` - Filing date (YYYY-MM-DD)
pub async fn form_13f(cik: &str, date: &str) -> Result<Vec<Form13FDTO>> {
    use chrono::Datelike;

    let client = build_client()?;
    let parsed = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        FinanceError::InvalidParameter {
            param: "date".into(),
            reason: "expected YYYY-MM-DD".into(),
        }
    })?;
    let year = parsed.format("%Y").to_string();
    let quarter = (parsed.month0() / 3 + 1).to_string();
    client
        .get(
            "/stable/institutional-ownership/extract",
            &[("cik", cik), ("year", &year), ("quarter", &quarter)],
        )
        .await
}

fn latest_completed_quarter() -> (String, String) {
    use chrono::Datelike;

    let today = chrono::Utc::now().date_naive();
    let current_quarter = (today.month0() / 3) + 1;
    let (year, quarter) = if current_quarter == 1 {
        (today.year() - 1, 4)
    } else {
        (today.year(), current_quarter - 1)
    };
    (year.to_string(), quarter.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_institutional_holders_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/funds/disclosure-holders-latest")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<InstitutionalHolderDTO> = client
            .get("/stable/funds/disclosure-holders-latest", &[])
            .await
            .unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].holder.as_deref(), Some("Vanguard Group Inc"));
    }

    #[tokio::test]
    async fn test_etf_holders_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/stable/etf/holdings")
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "apikey".into(),
                "test-key".into(),
            )]))
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

        let client = crate::adapters::fmp::build_test_client(&server.url()).unwrap();
        let resp: Vec<EtfHolderDTO> = client.get("/stable/etf/holdings", &[]).await.unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].asset.as_deref(), Some("AAPL"));
        assert!((resp[0].weight_percentage.unwrap() - 7.2).abs() < 0.01);
    }
}
