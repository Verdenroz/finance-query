// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::adapters::fmp::build_client;
use crate::adapters::fmp::quote::company::CompanyProfileDTO;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SecuritySearchDTO {
    pub symbol: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    pub currency: Option<String>,
    pub exchange: Option<String>,
    #[serde(rename = "exchangeFullName")]
    pub exchange_full_name: Option<String>,
    pub cik: Option<String>,
    pub cusip: Option<String>,
    pub isin: Option<String>,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
}

pub async fn name_search(query: &str) -> Result<Vec<SecuritySearchDTO>> {
    build_client()?
        .get("/stable/search-name", &[("query", query)])
        .await
}

pub async fn cik_search(cik: &str) -> Result<Vec<SecuritySearchDTO>> {
    build_client()?
        .get("/stable/search-cik", &[("cik", cik)])
        .await
}

pub async fn cusip_search(cusip: &str) -> Result<Vec<SecuritySearchDTO>> {
    build_client()?
        .get("/stable/search-cusip", &[("cusip", cusip)])
        .await
}

pub async fn isin_search(isin: &str) -> Result<Vec<SecuritySearchDTO>> {
    build_client()?
        .get("/stable/search-isin", &[("isin", isin)])
        .await
}

pub async fn exchange_variants(symbol: &str) -> Result<Vec<CompanyProfileDTO>> {
    build_client()?
        .get("/stable/search-exchange-variants", &[("symbol", symbol)])
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_payload_variants_deserialize() {
        let row: SecuritySearchDTO = serde_json::from_str(
            r#"{"symbol":"AAPL","companyName":"Apple Inc.","cusip":"037833100","marketCap":1.0}"#,
        )
        .unwrap();
        assert_eq!(row.company_name.as_deref(), Some("Apple Inc."));
        assert_eq!(row.cusip.as_deref(), Some("037833100"));
    }
}
