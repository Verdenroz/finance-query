//! Options contract reference data.
// unrouted: awaiting a capability route; see #264.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::super::{build_client, models::PaginatedResponseDTO};

/// Listed options contract specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OptionsContractDTO {
    /// ISO 10962 Classification of Financial Instruments code.
    pub cfi: Option<String>,
    /// Contract type, such as `call` or `put`.
    pub contract_type: Option<String>,
    /// Exercise style, such as `american` or `european`.
    pub exercise_style: Option<String>,
    /// Expiration date.
    pub expiration_date: Option<String>,
    /// Primary exchange MIC.
    pub primary_exchange: Option<String>,
    /// Underlying shares represented by one contract.
    pub shares_per_contract: Option<f64>,
    /// Strike price.
    pub strike_price: Option<f64>,
    /// Options ticker.
    pub ticker: Option<String>,
    /// Underlying ticker.
    pub underlying_ticker: Option<String>,
}

/// List options contracts, optionally filtering by underlying, expiration, or activity.
pub async fn options_contracts(
    params: &[(&str, &str)],
) -> Result<PaginatedResponseDTO<OptionsContractDTO>> {
    build_client()?
        .get("/v3/reference/options/contracts", params)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_shape_deserializes() {
        let contract: OptionsContractDTO = serde_json::from_value(serde_json::json!({
            "contract_type": "call",
            "expiration_date": "2026-08-21",
            "strike_price": 200,
            "ticker": "O:AAPL260821C00200000",
            "underlying_ticker": "AAPL"
        }))
        .unwrap();
        assert_eq!(contract.contract_type.as_deref(), Some("call"));
    }
}
