//! Advanced endpoints for Financial Modeling Prep (SIC codes, COT reports).

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::build_client;

/// Standard Industrial Classification code entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SicCode {
    /// SIC code.
    #[serde(rename = "sicCode")]
    pub sic_code: Option<String>,
    /// Industry title.
    #[serde(rename = "industryTitle")]
    pub industry_title: Option<String>,
}

/// Detailed SIC entry for a specific code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SicEntry {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Company name.
    pub name: Option<String>,
    /// CIK number.
    pub cik: Option<String>,
    /// SIC code.
    #[serde(rename = "sicCode")]
    pub sic_code: Option<String>,
    /// Industry title.
    #[serde(rename = "industryTitle")]
    pub industry_title: Option<String>,
    /// Business address.
    #[serde(rename = "businessAddress")]
    pub business_address: Option<String>,
    /// Phone number.
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
}

/// A symbol available in the Commitment of Traders report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CotSymbol {
    /// Trading symbol.
    #[serde(rename = "trading_symbol")]
    pub trading_symbol: Option<String>,
    /// Short name.
    #[serde(rename = "short_name")]
    pub short_name: Option<String>,
}

/// A Commitment of Traders report entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CotReport {
    /// Report date.
    pub date: Option<String>,
    /// Symbol.
    pub symbol: Option<String>,
    /// Short name.
    #[serde(rename = "short_name")]
    pub short_name: Option<String>,
    /// Current long (all).
    #[serde(rename = "current_long_all")]
    pub current_long_all: Option<f64>,
    /// Current short (all).
    #[serde(rename = "current_short_all")]
    pub current_short_all: Option<f64>,
    /// Change long (all).
    #[serde(rename = "change_long_all")]
    pub change_long_all: Option<f64>,
    /// Change short (all).
    #[serde(rename = "change_short_all")]
    pub change_short_all: Option<f64>,
    /// Percent long (all).
    #[serde(rename = "pct_of_oi_long_all")]
    pub pct_of_oi_long_all: Option<f64>,
    /// Percent short (all).
    #[serde(rename = "pct_of_oi_short_all")]
    pub pct_of_oi_short_all: Option<f64>,
}

/// A Commitment of Traders analysis entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CotAnalysis {
    /// Report date.
    pub date: Option<String>,
    /// Symbol.
    pub symbol: Option<String>,
    /// Short name.
    #[serde(rename = "short_name")]
    pub short_name: Option<String>,
    /// Sector.
    pub sector: Option<String>,
    /// Current net position.
    #[serde(rename = "currentNetPosition")]
    pub current_net_position: Option<f64>,
    /// Previous net position.
    #[serde(rename = "previousNetPosition")]
    pub previous_net_position: Option<f64>,
    /// Change in net position.
    #[serde(rename = "changeInNetPosition")]
    pub change_in_net_position: Option<f64>,
    /// Market sentiment.
    #[serde(rename = "marketSentiment")]
    pub market_sentiment: Option<String>,
    /// Reversal indicator.
    #[serde(rename = "reversalTrend")]
    pub reversal_trend: Option<String>,
}

/// Fetch all SIC codes.
pub async fn sic_codes() -> Result<Vec<SicCode>> {
    let client = build_client()?;
    client
        .get("/api/v4/standard_industrial_classification_list", &[])
        .await
}

/// Fetch companies by SIC code.
///
/// * `code` - SIC code (e.g., `"3674"`)
pub async fn sic_by_code(code: &str) -> Result<Vec<SicEntry>> {
    let client = build_client()?;
    client
        .get(
            "/api/v4/standard_industrial_classification",
            &[("sicCode", code)],
        )
        .await
}

/// Fetch available COT report symbols.
pub async fn cot_symbols() -> Result<Vec<CotSymbol>> {
    let client = build_client()?;
    client
        .get("/api/v4/commitment_of_traders_report/list", &[])
        .await
}

/// Fetch a Commitment of Traders report for a symbol.
///
/// * `symbol` - Trading symbol from the COT report list
pub async fn cot_report(symbol: &str) -> Result<Vec<CotReport>> {
    let client = build_client()?;
    let path = format!("/api/v4/commitment_of_traders_report/{symbol}");
    client.get(&path, &[]).await
}

/// Fetch a Commitment of Traders analysis for a symbol.
///
/// * `symbol` - Trading symbol from the COT report list
pub async fn cot_analysis(symbol: &str) -> Result<Vec<CotAnalysis>> {
    let client = build_client()?;
    let path = format!("/api/v4/commitment_of_traders_report_analysis/{symbol}");
    client.get(&path, &[]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sic_codes_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/api/v4/standard_industrial_classification_list",
            )
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("apikey".into(), "test-key".into()),
            ]))
            .with_status(200)
            .with_body(
                serde_json::json!([
                    {
                        "sicCode": "3674",
                        "industryTitle": "Semiconductors and Related Devices"
                    }
                ])
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::build_test_client(&server.url()).unwrap();
        let result: Vec<SicCode> = client
            .get("/api/v4/standard_industrial_classification_list", &[])
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sic_code.as_deref(), Some("3674"));
        assert_eq!(
            result[0].industry_title.as_deref(),
            Some("Semiconductors and Related Devices")
        );
    }
}
