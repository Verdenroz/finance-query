//! `FILINGS` capability: House Periodic Transaction Report disclosures.

mod index;
mod transactions;

use std::io::Read;

use chrono::{Datelike, Utc};
use futures::stream::{self, StreamExt};

use crate::error::{FinanceError, Result};
use crate::models::filings::CongressionalTrade;

use super::models::PtrIndexEntry;
use index::{parse_index, parse_us_date};
use transactions::{parse_transactions, to_congressional_trade};

/// Most recent PTR filings scanned per request. There is no per-symbol index
/// upstream — every filing is its own PDF, so a symbol lookup means opening
/// filings until enough of the recent activity has been covered.
const MAX_FILINGS_SCANNED: usize = 120;

const CONCURRENT_FETCHES: usize = 8;

async fn year_ptr_entries(year: i32) -> Result<Vec<PtrIndexEntry>> {
    let bytes = super::client()?.fetch_year_archive(year).await?;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| {
        FinanceError::ResponseStructureError {
            field: "housetrades.zip".to_string(),
            context: format!("failed to open House disclosure archive: {e}"),
        }
    })?;
    let mut text = String::new();
    archive
        .by_name(&format!("{year}FD.txt"))
        .map_err(|e| FinanceError::ResponseStructureError {
            field: "housetrades.zip".to_string(),
            context: format!("archive has no {year}FD.txt: {e}"),
        })?
        .read_to_string(&mut text)
        .map_err(|e| FinanceError::ResponseStructureError {
            field: "housetrades.zip".to_string(),
            context: format!("non-UTF8 House disclosure index: {e}"),
        })?;
    Ok(parse_index(&text, year))
}

/// Open one filing's PDF and return every transaction matching `symbol`.
/// Fetch, text-extraction, or table-parsing failures all resolve to an empty
/// list — a scanned (non-text) filing is an expected gap, not an error.
async fn matching_transactions(entry: PtrIndexEntry, symbol: &str) -> Vec<CongressionalTrade> {
    let Ok(client) = super::client() else {
        return Vec::new();
    };
    let Ok(bytes) = client.fetch_filing_pdf(entry.year, &entry.doc_id).await else {
        return Vec::new();
    };
    let Ok(text) = pdf_extract::extract_text_from_mem(&bytes) else {
        return Vec::new();
    };
    let year = entry.year;
    parse_transactions(&text)
        .into_iter()
        .filter(|tx| {
            tx.symbol
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case(symbol))
        })
        .map(|tx| to_congressional_trade(&entry, year, tx))
        .collect()
}

/// Fetch congressional (House) stock-trade disclosures for a symbol.
///
/// Scans the current year's PTR filings, newest first, topping up with the
/// previous year's when the current year hasn't accumulated
/// [`MAX_FILINGS_SCANNED`] yet (early January). Every filing beyond that
/// window is not scanned — this is a bounded-cost recency window, not a full
/// historical search.
pub(crate) async fn fetch_congressional_trades_response(
    symbol: &str,
) -> Result<Vec<CongressionalTrade>> {
    let symbol = symbol.to_uppercase();
    let year = Utc::now().year();

    let mut entries = year_ptr_entries(year).await?;
    if entries.len() < MAX_FILINGS_SCANNED
        && let Ok(prior) = year_ptr_entries(year - 1).await
    {
        entries.extend(prior);
    }
    entries.sort_by(|a, b| {
        parse_us_date(&b.filing_date)
            .cmp(&parse_us_date(&a.filing_date))
            .then_with(|| b.doc_id.cmp(&a.doc_id))
    });
    entries.truncate(MAX_FILINGS_SCANNED);

    let symbol_ref = symbol.as_str();
    let trades = stream::iter(entries)
        .map(|entry| async move { matching_transactions(entry, symbol_ref).await })
        .buffer_unordered(CONCURRENT_FETCHES)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    Ok(trades)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_live_congressional_trades() {
        let trades = super::fetch_congressional_trades_response("AAPL")
            .await
            .unwrap();
        assert!(!trades.is_empty());
    }
}
