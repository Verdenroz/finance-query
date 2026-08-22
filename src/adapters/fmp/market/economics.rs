// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::error::Result;

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
        let premium: MarketRiskPremiumDTO = serde_json::from_str(
            r#"{"country":"United States","countryRiskPremium":0.0,"totalEquityRiskPremium":4.5}"#,
        )
        .unwrap();
        assert_eq!(premium.total_equity_risk_premium, Some(4.5));
    }
}
