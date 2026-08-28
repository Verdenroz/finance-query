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
/// list. A scanned filing carries no text layer and is an expected gap; any
/// other extraction failure is logged before it is swallowed.
async fn matching_transactions(entry: PtrIndexEntry, symbol: &str) -> Vec<CongressionalTrade> {
    let Ok(client) = super::client() else {
        return Vec::new();
    };
    let Ok(bytes) = client.fetch_filing_pdf(entry.year, &entry.doc_id).await else {
        return Vec::new();
    };
    let lines = match super::pdf::extract_lines(bytes) {
        Ok(lines) => lines,
        Err(super::pdf::PdfError::NoTextLayer) => return Vec::new(),
        Err(e) => {
            tracing::debug!("House PTR {} is unreadable: {e}", entry.doc_id);
            return Vec::new();
        }
    };
    let year = entry.year;
    parse_transactions(&lines)
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
    use super::*;

    /// Values in these assertions were read off the filings themselves, not
    /// captured from this extractor.
    fn rows(fixture: &str) -> Vec<transactions::ParsedTransaction> {
        let path = format!(
            "{}/tests/fixtures/housetrades/{fixture}",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).expect("fixture");
        let lines = super::super::pdf::extract_lines(bytes).expect("text layer");
        parse_transactions(&lines)
    }

    #[test]
    fn reads_every_row_of_a_real_filing() {
        let rows = rows("ptr_20023717.pdf");
        assert_eq!(rows.len(), 6);
        let symbols: Vec<_> = rows.iter().filter_map(|r| r.symbol.as_deref()).collect();
        assert_eq!(symbols, ["CCK", "DHR", "FIS", "FIS", "GPN", "JPM"]);

        assert_eq!(rows[0].asset_description, "Crown Holdings, Inc.");
        assert_eq!(rows[0].trade_type.as_deref(), Some("Purchase"));
        assert_eq!(rows[0].transaction_date.as_deref(), Some("2023-04-19"));
        assert_eq!(rows[0].amount.as_deref(), Some("$1,001 - $15,000"));
    }

    #[test]
    fn rejoins_a_description_that_wrapped_in_the_rendered_table() {
        let rows = rows("ptr_20023717.pdf");
        assert_eq!(
            rows[2].asset_description,
            "Fidelity National Information Services, Inc."
        );
        assert_eq!(rows[2].trade_type.as_deref(), Some("Sale"));
        assert_eq!(rows[2].transaction_date.as_deref(), Some("2023-05-04"));
    }

    #[test]
    fn rejoins_an_amount_that_wrapped_in_the_rendered_table() {
        let rows = rows("ptr_20026736.pdf");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].asset_description, "MAI Managed");
        assert_eq!(rows[0].symbol, None);
        assert_eq!(rows[0].trade_type.as_deref(), Some("Sale"));
        assert_eq!(rows[0].transaction_date.as_deref(), Some("2025-01-10"));
        assert_eq!(rows[0].amount.as_deref(), Some("$50,001 - $100,000"));
    }

    #[test]
    fn a_scanned_filing_reports_no_text_layer() {
        let path = format!(
            "{}/tests/fixtures/housetrades/ptr_scanned_8217876.pdf",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).expect("fixture");
        assert_eq!(
            super::super::pdf::extract_lines(bytes).unwrap_err(),
            super::super::pdf::PdfError::NoTextLayer
        );
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_live_congressional_trades() {
        let trades = super::fetch_congressional_trades_response("AAPL")
            .await
            .unwrap();
        assert!(!trades.is_empty());
    }
}
