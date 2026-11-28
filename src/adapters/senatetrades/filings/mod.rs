//! `FILINGS` capability: Senate Periodic Transaction Report disclosures.

mod search;
mod transactions;

use chromiumoxide::element::Element;
use futures::stream::{self, StreamExt};

use crate::error::Result;
use crate::models::filings::CongressionalTrade;

use super::client::SenateTradesClient;
use search::search_recent_ptrs;
use transactions::matching_transactions;

/// Most recent PTR filings scanned per request, matching the House
/// adapter's bounded-recency-window philosophy: there is no per-symbol
/// search here either, only per-filer/per-date.
const MAX_FILINGS_SCANNED: usize = 100;

/// Concurrent report-page fetches in flight at once, reusing one browser
/// session (separate CDP targets, not separate Chromium processes).
const CONCURRENT_FETCHES: usize = 8;

/// Fetch congressional (Senate) stock-trade disclosures for a symbol.
///
/// Launches one headless Chromium session, searches the most recent Senator
/// PTR filings, then opens each report concurrently to filter by symbol.
pub(crate) async fn fetch_congressional_trades_response(
    symbol: &str,
) -> Result<Vec<CongressionalTrade>> {
    let symbol = symbol.to_uppercase();
    let client = SenateTradesClient::launch().await?;

    let result = scan(&client, &symbol).await;
    client.close().await;
    result
}

/// Text of the `index`th cell. The outer `None` means the cell is missing or
/// the read failed, which drops the row; the inner `None` is an empty cell.
async fn cell_text(cells: &[Element], index: usize) -> Option<Option<String>> {
    cells.get(index)?.inner_text().await.ok()
}

async fn scan(client: &SenateTradesClient, symbol: &str) -> Result<Vec<CongressionalTrade>> {
    let mut entries = search_recent_ptrs(client).await?;
    entries.truncate(MAX_FILINGS_SCANNED);

    let trades = stream::iter(entries)
        .map(|entry| async move { matching_transactions(client, entry, symbol).await })
        .buffer_unordered(CONCURRENT_FETCHES)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    Ok(trades)
}

#[cfg(test)]
fn cells<const N: usize>(texts: [&str; N]) -> [Option<String>; N] {
    texts.map(|t| Some(t.to_string()))
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "requires network access and a local chromium binary"]
    async fn test_live_congressional_trades() {
        let trades = super::fetch_congressional_trades_response("AAPL")
            .await
            .unwrap();
        assert!(!trades.is_empty());
    }
}
