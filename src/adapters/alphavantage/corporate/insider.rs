//! Insider transactions (`INSIDER_TRANSACTIONS`) — a second route for the
//! ownership surface alongside EDGAR's Form 3/4/5 parsing.
//!
//! Alpha Vantage's fields are coarser than a primary-source filing line item
//! (no accession number, form type, or derivative/non-derivative split), but
//! it populates the same canonical [`InsiderTrade`] model so callers can
//! route [`Capability::FILINGS`](crate::providers::Capability::FILINGS)
//! through Alpha Vantage as a fallback when EDGAR is unavailable or slow.

use crate::adapters::alphavantage::models::InsiderTransactionDTO;
use crate::error::{FinanceError, Result};
use crate::models::filings::InsiderTrade;

use super::super::build_client;

/// Fetch raw insider transaction records for a symbol.
pub async fn insider_transactions(symbol: &str) -> Result<Vec<InsiderTransactionDTO>> {
    let client = build_client()?;
    let json = client
        .get("INSIDER_TRANSACTIONS", &[("symbol", symbol)])
        .await?;

    let data = json.get("data").and_then(|v| v.as_array()).ok_or_else(|| {
        FinanceError::ResponseStructureError {
            field: "data".to_string(),
            context: "Missing data array in insider transactions response".to_string(),
        }
    })?;

    data.iter()
        .map(|v| {
            serde_json::from_value(v.clone()).map_err(|e| FinanceError::ResponseStructureError {
                field: "data[]".to_string(),
                context: format!("Failed to parse insider transaction record: {e}"),
            })
        })
        .collect()
}

/// Convert one raw insider transaction record into the canonical
/// [`InsiderTrade`] model. Fields Alpha Vantage doesn't report (form type,
/// accession number, filing URL, security title beyond `security_type`,
/// director/officer/10%-owner flags, shares owned after, derivative flag)
/// are left at their `None`/`false` defaults.
fn to_insider_trade(dto: InsiderTransactionDTO) -> InsiderTrade {
    InsiderTrade {
        symbol: dto.ticker,
        insider_name: dto.executive,
        officer_title: dto.executive_title,
        security_title: dto.security_type,
        transaction_date: dto.transaction_date,
        acquired_disposed: dto.acquisition_or_disposal,
        shares: dto.shares,
        price_per_share: dto.share_price,
        ..Default::default()
    }
}

/// Fetch canonical insider transactions for a symbol, newest filing first as
/// reported by Alpha Vantage. `limit` truncates the result — Alpha Vantage's
/// endpoint takes no page size, so this is applied client-side.
pub async fn fetch_insider_trades_response(symbol: &str, limit: u32) -> Result<Vec<InsiderTrade>> {
    let records = insider_transactions(symbol).await?;
    Ok(records
        .into_iter()
        .take(limit as usize)
        .map(to_insider_trade)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transaction_dto() -> InsiderTransactionDTO {
        serde_json::from_value(serde_json::json!({
            "transaction_date": "2026-07-01",
            "ticker": "IBM",
            "executive": "ROBINSON, ANNE",
            "executive_title": "Senior Vice President",
            "security_type": "Common Stock",
            "acquisition_or_disposal": "D",
            "shares": "781.0",
            "share_price": "286.725"
        }))
        .unwrap()
    }

    #[test]
    fn maps_transaction_fields() {
        let out = to_insider_trade(transaction_dto());
        assert_eq!(out.symbol.as_deref(), Some("IBM"));
        assert_eq!(out.insider_name.as_deref(), Some("ROBINSON, ANNE"));
        assert_eq!(out.officer_title.as_deref(), Some("Senior Vice President"));
        assert_eq!(out.security_title.as_deref(), Some("Common Stock"));
        assert_eq!(out.acquired_disposed.as_deref(), Some("D"));
        assert_eq!(out.shares, Some(781.0));
        assert!((out.price_per_share.unwrap() - 286.725).abs() < 0.001);
        // Unavailable-from-AV fields stay at their defaults.
        assert_eq!(out.form_type, None);
        assert_eq!(out.accession_number, None);
        assert!(!out.is_director);
        assert!(!out.is_officer);
    }

    #[tokio::test]
    async fn test_insider_transactions_mock() {
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("function".into(), "INSIDER_TRANSACTIONS".into()),
                mockito::Matcher::UrlEncoded("symbol".into(), "IBM".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                serde_json::json!({
                    "data": [
                        {
                            "transaction_date": "2026-07-01",
                            "ticker": "IBM",
                            "executive": "ROBINSON, ANNE",
                            "executive_title": "Senior Vice President",
                            "security_type": "Common Stock",
                            "acquisition_or_disposal": "D",
                            "shares": "781.0",
                            "share_price": "286.725"
                        }
                    ]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let client = super::super::super::build_test_client(&server.url()).unwrap();
        let json = client
            .get("INSIDER_TRANSACTIONS", &[("symbol", "IBM")])
            .await
            .unwrap();

        let data = json.get("data").and_then(|v| v.as_array()).unwrap();
        assert_eq!(data.len(), 1);
        let dto: InsiderTransactionDTO = serde_json::from_value(data[0].clone()).unwrap();
        assert_eq!(dto.ticker.as_deref(), Some("IBM"));
        assert_eq!(dto.shares, Some(781.0));
    }
}
