//! Earnings transcript endpoint
//!
//! Fetches earnings call transcripts from Yahoo Finance.
//! Supports fetching by symbol with optional quarter/year filters.

use crate::client::YahooClient;
use crate::error::{FinanceError, Result};
use crate::models::quote::quote_type::QuoteTypeResponse;
use crate::models::transcript::{Transcript, TranscriptWithMeta};
use crate::scrapers::yahoo_earnings::{EarningsCall, scrape_earnings_calls};

/// Fetch earnings call transcript by event ID and company ID (low-level)
///
/// Most users should use `fetch_for_symbol` instead.
pub(crate) async fn fetch(
    client: &YahooClient,
    event_id: &str,
    company_id: &str,
) -> Result<Transcript> {
    let url = "https://finance.yahoo.com/xhr/transcript";

    let params = [
        ("eventType", "earnings_call"),
        ("quartrId", company_id),
        ("eventId", event_id),
        ("lang", &client.config().lang),
        ("region", &client.config().region),
    ];

    let response = client.request_with_params(url, &params).await?;
    Ok(response.json().await?)
}

/// Get the quartr_id (company ID) for a symbol
pub(crate) async fn get_quartr_id(client: &YahooClient, symbol: &str) -> Result<String> {
    use crate::endpoints::urls::api;

    let url = api::quote_type(symbol);
    let response = client.request_with_crumb(&url).await?;
    let data: QuoteTypeResponse = response.json().await?;

    data.quote_type
        .result
        .first()
        .and_then(|r| r.quartr_id.clone())
        .ok_or_else(|| FinanceError::ResponseStructureError {
            field: "quartrId".to_string(),
            context: format!("No quartrId found for symbol {}", symbol),
        })
}

/// Fetch earnings transcript for a symbol
///
/// This high-level function handles all the complexity internally:
/// 1. Gets the quartr_id (company_id) from the quote_type endpoint
/// 2. Scrapes the earnings calls list to find available transcripts
/// 3. Fetches the requested transcript
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
/// * `quarter` - Optional fiscal quarter (Q1, Q2, Q3, Q4). If None with year=None, gets latest.
/// * `year` - Optional fiscal year. If None with quarter=None, gets latest.
pub async fn fetch_for_symbol(
    client: &YahooClient,
    symbol: &str,
    quarter: Option<&str>,
    year: Option<i32>,
) -> Result<Transcript> {
    let quartr_id = get_quartr_id(client, symbol).await?;
    let calls = scrape_earnings_calls(symbol).await?;
    let call = find_matching_call(&calls, quarter, year)?;
    fetch(client, &call.event_id, &quartr_id).await
}

/// Fetch multiple earnings transcripts for a symbol
///
/// # Arguments
///
/// * `client` - Yahoo Finance client
/// * `symbol` - Stock symbol (e.g., "AAPL", "MSFT")
/// * `limit` - Optional maximum number of transcripts. If None, fetches all.
pub async fn fetch_all_for_symbol(
    client: &YahooClient,
    symbol: &str,
    limit: Option<usize>,
) -> Result<Vec<TranscriptWithMeta>> {
    let quartr_id = get_quartr_id(client, symbol).await?;
    let calls = scrape_earnings_calls(symbol).await?;

    let calls_to_fetch: Vec<_> = match limit {
        Some(n) => calls.into_iter().take(n).collect(),
        None => calls,
    };

    // Fetch all transcripts in parallel
    let futures: Vec<_> = calls_to_fetch
        .into_iter()
        .map(|call| {
            let quartr_id = quartr_id.clone();
            async move {
                match fetch(client, &call.event_id, &quartr_id).await {
                    Ok(transcript) => Some(TranscriptWithMeta {
                        event_id: call.event_id,
                        quarter: call.quarter,
                        year: call.year,
                        title: call.title,
                        url: call.url,
                        transcript,
                    }),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to fetch transcript for Q{} {}: {}",
                            call.quarter.as_deref().unwrap_or("?"),
                            call.year.map(|y| y.to_string()).unwrap_or_default(),
                            e
                        );
                        None
                    }
                }
            }
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect();

    if results.is_empty() {
        return Err(FinanceError::ResponseStructureError {
            field: "transcripts".to_string(),
            context: format!("No transcripts could be fetched for {}", symbol),
        });
    }

    Ok(results)
}

/// Find a matching earnings call from the list
fn find_matching_call<'a>(
    calls: &'a [EarningsCall],
    quarter: Option<&str>,
    year: Option<i32>,
) -> Result<&'a EarningsCall> {
    match (quarter, year) {
        (Some(q), Some(y)) => {
            let q_upper = q.to_uppercase();
            calls
                .iter()
                .find(|c| {
                    c.quarter.as_ref().map(|cq| cq.to_uppercase()) == Some(q_upper.clone())
                        && c.year == Some(y)
                })
                .ok_or_else(|| FinanceError::ResponseStructureError {
                    field: "earnings_call".to_string(),
                    context: format!("No earnings call found for {} {}", q, y),
                })
        }
        (Some(q), None) => {
            let q_upper = q.to_uppercase();
            calls
                .iter()
                .find(|c| c.quarter.as_ref().map(|cq| cq.to_uppercase()) == Some(q_upper.clone()))
                .ok_or_else(|| FinanceError::ResponseStructureError {
                    field: "earnings_call".to_string(),
                    context: format!("No earnings call found for quarter {}", q),
                })
        }
        (None, Some(y)) => calls.iter().find(|c| c.year == Some(y)).ok_or_else(|| {
            FinanceError::ResponseStructureError {
                field: "earnings_call".to_string(),
                context: format!("No earnings call found for year {}", y),
            }
        }),
        (None, None) => calls
            .first()
            .ok_or_else(|| FinanceError::ResponseStructureError {
                field: "earnings_call".to_string(),
                context: "No earnings calls available".to_string(),
            }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_call_latest() {
        let calls = vec![
            EarningsCall {
                event_id: "1".to_string(),
                quarter: Some("Q4".to_string()),
                year: Some(2024),
                title: "Q4 2024".to_string(),
                url: "".to_string(),
            },
            EarningsCall {
                event_id: "2".to_string(),
                quarter: Some("Q3".to_string()),
                year: Some(2024),
                title: "Q3 2024".to_string(),
                url: "".to_string(),
            },
        ];

        let result = find_matching_call(&calls, None, None).unwrap();
        assert_eq!(result.event_id, "1");
    }

    #[test]
    fn test_find_matching_call_specific() {
        let calls = vec![
            EarningsCall {
                event_id: "1".to_string(),
                quarter: Some("Q4".to_string()),
                year: Some(2024),
                title: "Q4 2024".to_string(),
                url: "".to_string(),
            },
            EarningsCall {
                event_id: "2".to_string(),
                quarter: Some("Q3".to_string()),
                year: Some(2024),
                title: "Q3 2024".to_string(),
                url: "".to_string(),
            },
        ];

        let result = find_matching_call(&calls, Some("Q3"), Some(2024)).unwrap();
        assert_eq!(result.event_id, "2");
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_for_symbol_latest() {
        use crate::client::{ClientConfig, YahooClient};

        let client = YahooClient::new(ClientConfig::default()).await.unwrap();

        let result = fetch_for_symbol(&client, "AAPL", None, None).await;
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let transcript = result.unwrap();
        assert!(!transcript.text().is_empty());
        println!("Quarter: {} {}", transcript.quarter(), transcript.year());
    }
}
