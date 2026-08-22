//! Canonical conversion for Alpha Vantage's `ETF_PROFILE`.
//!
//! The only wired source of ETF holdings — no other integrated provider serves
//! fund composition.

use crate::adapters::alphavantage::models::{EtfHoldingDTO, EtfProfileDTO};
use crate::error::Result;
use crate::models::fundamentals::{EtfHolding, EtfProfile};

pub(crate) fn to_etf_holding(dto: EtfHoldingDTO) -> EtfHolding {
    EtfHolding {
        symbol: dto.symbol,
        description: dto.description,
        weight: dto.weight,
    }
}

pub(crate) fn to_etf_profile(dto: EtfProfileDTO) -> EtfProfile {
    let mut holdings: Vec<EtfHolding> = dto.holdings.into_iter().map(to_etf_holding).collect();
    // Alpha Vantage returns holdings in portfolio order, which is usually but
    // not always weight-descending; sorting makes the contract explicit.
    holdings.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    EtfProfile {
        symbol: Some(dto.symbol),
        name: dto.name,
        asset_type: dto.asset_type,
        net_assets: dto.net_assets,
        net_expense_ratio: dto.net_expense_ratio,
        portfolio_turnover: dto.portfolio_turnover,
        dividend_yield: dto.dividend_yield,
        inception_date: dto.inception_date,
        holdings,
        // Alpha Vantage's ETF_PROFILE has no sector/country breakdown.
        sector_weightings: Vec::new(),
        country_weightings: Vec::new(),
    }
}

/// Fetch the canonical ETF profile and holdings for a symbol.
pub async fn fetch_etf_profile_response(symbol: &str) -> Result<EtfProfile> {
    let dto = super::etf_profile(symbol).await?;
    Ok(to_etf_profile(dto))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_dto() -> EtfProfileDTO {
        serde_json::from_value(serde_json::json!({
            "symbol": "QQQ",
            "name": "Invesco QQQ Trust",
            "asset_type": "ETF",
            "net_assets": 300_000_000_000.0,
            "net_expense_ratio": 0.002,
            "portfolio_turnover": 0.07,
            "dividend_yield": 0.0058,
            "inception_date": "1999-03-10",
            "holdings": [
                { "symbol": "MSFT", "description": "MICROSOFT CORP", "weight": 0.081 },
                { "symbol": "AAPL", "description": "APPLE INC", "weight": 0.089 },
                { "symbol": "NVDA", "description": "NVIDIA CORP", "weight": null }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn maps_profile_fields() {
        let out = to_etf_profile(profile_dto());
        assert_eq!(out.symbol.as_deref(), Some("QQQ"));
        assert_eq!(out.name.as_deref(), Some("Invesco QQQ Trust"));
        assert_eq!(out.net_assets, Some(300_000_000_000.0));
        assert_eq!(out.net_expense_ratio, Some(0.002));
        assert_eq!(out.inception_date.as_deref(), Some("1999-03-10"));
        assert_eq!(out.holdings.len(), 3);
    }

    #[test]
    fn sorts_holdings_by_weight_and_keeps_unweighted_ones_last() {
        let out = to_etf_profile(profile_dto());
        assert_eq!(out.holdings[0].symbol.as_deref(), Some("AAPL"));
        assert_eq!(out.holdings[0].weight, Some(0.089));
        assert_eq!(out.holdings[1].symbol.as_deref(), Some("MSFT"));
        // A `None` weight can't be ordered against a number, so it settles last
        // rather than being dropped.
        assert_eq!(out.holdings[2].symbol.as_deref(), Some("NVDA"));
        assert_eq!(out.holdings[2].weight, None);
    }

    #[test]
    fn empty_holdings_are_not_an_error() {
        let dto: EtfProfileDTO = serde_json::from_value(serde_json::json!({
            "symbol": "XYZ",
            "holdings": []
        }))
        .unwrap();
        assert!(to_etf_profile(dto).holdings.is_empty());
    }
}
