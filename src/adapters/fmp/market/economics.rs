#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TreasuryRateDTO {
    pub date: Option<String>,
    #[serde(rename = "month1")]
    pub month_1: Option<f64>,
    #[serde(rename = "month2")]
    pub month_2: Option<f64>,
    #[serde(rename = "month3")]
    pub month_3: Option<f64>,
    #[serde(rename = "month6")]
    pub month_6: Option<f64>,
    #[serde(rename = "year1")]
    pub year_1: Option<f64>,
    #[serde(rename = "year2")]
    pub year_2: Option<f64>,
    #[serde(rename = "year3")]
    pub year_3: Option<f64>,
    #[serde(rename = "year5")]
    pub year_5: Option<f64>,
    #[serde(rename = "year7")]
    pub year_7: Option<f64>,
    #[serde(rename = "year10")]
    pub year_10: Option<f64>,
    #[serde(rename = "year20")]
    pub year_20: Option<f64>,
    #[serde(rename = "year30")]
    pub year_30: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EconomicIndicatorDTO {
    pub name: Option<String>,
    pub date: Option<String>,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MarketRiskPremiumDTO {
    pub country: Option<String>,
    pub continent: Option<String>,
    #[serde(rename = "countryRiskPremium")]
    pub country_risk_premium: Option<f64>,
    #[serde(rename = "totalEquityRiskPremium")]
    pub total_equity_risk_premium: Option<f64>,
}

pub async fn treasury_rates(from: Option<&str>, to: Option<&str>) -> Result<Vec<TreasuryRateDTO>> {
    let mut params = Vec::new();
    if let Some(from) = from {
        params.push(("from", from));
    }
    if let Some(to) = to {
        params.push(("to", to));
    }
    build_client()?.get("/stable/treasury-rates", &params).await
}

pub async fn economic_indicators(name: &str) -> Result<Vec<EconomicIndicatorDTO>> {
    build_client()?
        .get("/stable/economic-indicators", &[("name", name)])
        .await
}

pub async fn market_risk_premium() -> Result<Vec<MarketRiskPremiumDTO>> {
    build_client()?
        .get("/stable/market-risk-premium", &[])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_payloads_deserialize() {
        let rate: TreasuryRateDTO =
            serde_json::from_str(r#"{"date":"2026-08-06","month1":4.3,"year10":4.2}"#).unwrap();
        assert_eq!(rate.year_10, Some(4.2));

        let premium: MarketRiskPremiumDTO = serde_json::from_str(
            r#"{"country":"United States","countryRiskPremium":0.0,"totalEquityRiskPremium":4.5}"#,
        )
        .unwrap();
        assert_eq!(premium.total_equity_risk_premium, Some(4.5));
    }
}
