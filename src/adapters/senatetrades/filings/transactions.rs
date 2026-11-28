//! Report-page transaction table extraction.
//!
//! A born-digital filing renders `table.table-striped` with columns `#`,
//! Transaction Date, Owner, Ticker, Asset Name, Asset Type, Type, Amount,
//! Comment. Older/scanned ("paper") filings render no such table; those
//! produce no rows here rather than being specially detected, the same
//! boundary the House adapter draws around non-text-layer PDFs.

use chromiumoxide::element::Element;

use crate::models::filings::CongressionalTrade;

use super::super::client::{SENATE_BASE, SenateTradesClient, wait_for_elements};
use super::super::models::SenateFilingEntry;
use super::search::parse_us_date;

struct ParsedTransaction {
    symbol: Option<String>,
    asset_description: Option<String>,
    trade_type: Option<String>,
    transaction_date: Option<String>,
    amount: Option<String>,
}

/// Open one filing's report page and return every transaction matching
/// `symbol`. Any failure along the way (navigation, missing table, a paper
/// filing) resolves to an empty list rather than an error.
pub(super) async fn matching_transactions(
    client: &SenateTradesClient,
    entry: SenateFilingEntry,
    symbol: &str,
) -> Vec<CongressionalTrade> {
    let Ok(page) = client
        .new_page(&format!("{SENATE_BASE}{}", entry.path))
        .await
    else {
        return Vec::new();
    };
    let Ok(rows) = wait_for_elements(&page, "table.table-striped tbody tr").await else {
        page.close().await.ok();
        return Vec::new();
    };

    let mut trades = Vec::new();
    for row in &rows {
        if let Some(tx) = parse_row(row).await
            && tx
                .symbol
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case(symbol))
        {
            trades.push(to_congressional_trade(&entry, tx));
        }
    }
    page.close().await.ok();
    trades
}

async fn parse_row(row: &Element) -> Option<ParsedTransaction> {
    let cells = row.find_elements("td").await.ok()?;
    let mut texts = Vec::with_capacity(cells.len());
    for cell in &cells {
        texts.push(cell.inner_text().await.ok().flatten());
    }
    transaction_from_cells(&texts)
}

fn transaction_from_cells(cells: &[Option<String>]) -> Option<ParsedTransaction> {
    let transaction_date = cells.get(1)?.as_deref()?;
    let ticker = cells.get(3)?.as_deref().unwrap_or_default();
    let asset_name = cells.get(4)?.as_deref()?;
    let trade_type = cells.get(6)?.as_deref()?;
    let amount = cells.get(7)?.as_deref()?;

    let ticker = ticker.trim();
    Some(ParsedTransaction {
        symbol: (!ticker.is_empty() && ticker != "--").then(|| ticker.to_uppercase()),
        asset_description: normalize_whitespace(asset_name),
        trade_type: normalize_whitespace(trade_type),
        transaction_date: parse_us_date(transaction_date).map(|d| d.to_string()),
        amount: normalize_whitespace(amount),
    })
}

fn normalize_whitespace(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    (!collapsed.is_empty()).then_some(collapsed)
}

fn to_congressional_trade(entry: &SenateFilingEntry, tx: ParsedTransaction) -> CongressionalTrade {
    CongressionalTrade {
        symbol: tx.symbol,
        first_name: Some(entry.first_name.clone()),
        last_name: Some(entry.last_name.clone()),
        office: Some("Senate".to_string()),
        district: None,
        trade_type: tx.trade_type,
        amount: tx.amount,
        asset_description: tx.asset_description,
        transaction_date: tx.transaction_date,
        disclosure_date: parse_us_date(&entry.filed_date).map(|d| d.to_string()),
        link: Some(format!("{SENATE_BASE}{}", entry.path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry() -> SenateFilingEntry {
        SenateFilingEntry {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
            filed_date: "03/14/2025".into(),
            path: "/search/view/ptr/abc/".into(),
        }
    }

    fn cells(texts: &[&str]) -> Vec<Option<String>> {
        texts.iter().map(|t| Some(t.to_string())).collect()
    }

    #[test]
    fn transaction_from_cells_reads_columns() {
        let row = cells(&[
            "1",
            "03/10/2025",
            "Self",
            " aapl ",
            "Apple  Inc.\n Common Stock",
            "Stock",
            "Purchase",
            "$1,001 - $15,000",
            "",
        ]);
        let tx = transaction_from_cells(&row).unwrap();
        assert_eq!(tx.symbol.as_deref(), Some("AAPL"));
        assert_eq!(
            tx.asset_description.as_deref(),
            Some("Apple Inc. Common Stock")
        );
        assert_eq!(tx.trade_type.as_deref(), Some("Purchase"));
        assert_eq!(tx.transaction_date.as_deref(), Some("2025-03-10"));
        assert_eq!(tx.amount.as_deref(), Some("$1,001 - $15,000"));
    }

    #[test]
    fn transaction_from_cells_treats_dashes_and_blank_ticker_as_no_symbol() {
        for ticker in ["--", "", "  "] {
            let row = cells(&[
                "1",
                "03/10/2025",
                "Self",
                ticker,
                "Bond",
                "Bond",
                "Sale",
                "$1 - $2",
            ]);
            assert_eq!(transaction_from_cells(&row).unwrap().symbol, None);
        }
    }

    #[test]
    fn transaction_from_cells_missing_ticker_text_is_no_symbol() {
        let mut row = cells(&[
            "1",
            "03/10/2025",
            "Self",
            "x",
            "Bond",
            "Bond",
            "Sale",
            "$1 - $2",
        ]);
        row[3] = None;
        assert_eq!(transaction_from_cells(&row).unwrap().symbol, None);
    }

    #[test]
    fn transaction_from_cells_bad_date_keeps_row_without_date() {
        let row = cells(&[
            "1", "soon", "Self", "AAPL", "Apple", "Stock", "Sale", "$1 - $2",
        ]);
        assert_eq!(transaction_from_cells(&row).unwrap().transaction_date, None);
    }

    #[test]
    fn transaction_from_cells_ignores_unused_cell_without_text() {
        let mut row = cells(&[
            "1",
            "03/10/2025",
            "Self",
            "AAPL",
            "Apple",
            "Stock",
            "Sale",
            "$1 - $2",
        ]);
        row[0] = None;
        row[2] = None;
        row[5] = None;
        assert!(transaction_from_cells(&row).is_some());
    }

    #[test]
    fn transaction_from_cells_rejects_short_row() {
        let row = cells(&["1", "03/10/2025", "Self", "AAPL", "Apple", "Stock"]);
        assert!(transaction_from_cells(&row).is_none());
    }

    #[test]
    fn transaction_from_cells_rejects_missing_amount_text() {
        let mut row = cells(&[
            "1",
            "03/10/2025",
            "Self",
            "AAPL",
            "Apple",
            "Stock",
            "Sale",
            "x",
        ]);
        row[7] = None;
        assert!(transaction_from_cells(&row).is_none());
    }

    #[test]
    fn normalize_whitespace_collapses_runs() {
        assert_eq!(
            normalize_whitespace("  Apple \n  Inc.\t Common "),
            Some("Apple Inc. Common".to_string())
        );
    }

    #[test]
    fn normalize_whitespace_drops_blank() {
        assert_eq!(normalize_whitespace(" \n\t"), None);
    }

    #[test]
    fn trade_carries_filing_metadata() {
        let tx = ParsedTransaction {
            symbol: Some("AAPL".into()),
            asset_description: Some("Apple Inc.".into()),
            trade_type: Some("Purchase".into()),
            transaction_date: Some("2025-03-10".into()),
            amount: Some("$1,001 - $15,000".into()),
        };
        let trade = to_congressional_trade(&entry(), tx);
        assert_eq!(trade.symbol.as_deref(), Some("AAPL"));
        assert_eq!(trade.office.as_deref(), Some("Senate"));
        assert_eq!(trade.district, None);
        assert_eq!(trade.disclosure_date.as_deref(), Some("2025-03-14"));
        assert_eq!(
            trade.link.as_deref(),
            Some("https://efdsearch.senate.gov/search/view/ptr/abc/")
        );
    }

    #[test]
    fn trade_with_unparseable_filed_date_has_no_disclosure_date() {
        let mut filing = entry();
        filing.filed_date = "not a date".into();
        let tx = ParsedTransaction {
            symbol: None,
            asset_description: None,
            trade_type: None,
            transaction_date: None,
            amount: None,
        };
        assert_eq!(to_congressional_trade(&filing, tx).disclosure_date, None);
    }
}
