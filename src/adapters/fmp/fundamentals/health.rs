#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FinancialScoreDTO {
    pub symbol: Option<String>,
    #[serde(rename = "reportedCurrency")]
    pub reported_currency: Option<String>,
    #[serde(rename = "altmanZScore")]
    pub altman_z_score: Option<f64>,
    #[serde(rename = "piotroskiScore")]
    pub piotroski_score: Option<i32>,
    #[serde(rename = "workingCapital")]
    pub working_capital: Option<f64>,
    #[serde(rename = "totalAssets")]
    pub total_assets: Option<f64>,
    #[serde(rename = "retainedEarnings")]
    pub retained_earnings: Option<f64>,
    pub ebit: Option<f64>,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    #[serde(rename = "totalLiabilities")]
    pub total_liabilities: Option<f64>,
    pub revenue: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OwnerEarningsDTO {
    pub symbol: Option<String>,
    #[serde(rename = "reportedCurrency")]
    pub reported_currency: Option<String>,
    #[serde(rename = "fiscalYear")]
    pub fiscal_year: Option<String>,
    pub period: Option<String>,
    pub date: Option<String>,
    #[serde(rename = "averagePPE")]
    pub average_ppe: Option<f64>,
    #[serde(rename = "maintenanceCapex")]
    pub maintenance_capex: Option<f64>,
    #[serde(rename = "ownersEarnings")]
    pub owners_earnings: Option<f64>,
    #[serde(rename = "growthCapex")]
    pub growth_capex: Option<f64>,
    #[serde(rename = "ownersEarningsPerShare")]
    pub owners_earnings_per_share: Option<f64>,
}

pub async fn financial_scores(symbol: &str) -> Result<Vec<FinancialScoreDTO>> {
    build_client()?
        .get("/stable/financial-scores", &[("symbol", symbol)])
        .await
}

pub async fn owner_earnings(symbol: &str) -> Result<Vec<OwnerEarningsDTO>> {
    build_client()?
        .get("/stable/owner-earnings", &[("symbol", symbol)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_payloads_deserialize() {
        let score: FinancialScoreDTO =
            serde_json::from_str(r#"{"symbol":"AAPL","altmanZScore":12.5,"piotroskiScore":9}"#)
                .unwrap();
        assert_eq!(score.piotroski_score, Some(9));

        let earnings: OwnerEarningsDTO = serde_json::from_str(
            r#"{"symbol":"AAPL","fiscalYear":"2026","ownersEarningsPerShare":2.3}"#,
        )
        .unwrap();
        assert_eq!(earnings.owners_earnings_per_share, Some(2.3));
    }
}
